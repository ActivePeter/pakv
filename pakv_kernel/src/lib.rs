// pub mod file;
// pub mod channel_caller;
pub mod file;
pub mod serial;
pub mod meta;
// mod client_match_msg;

use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use serial::KvOpe;
use file::DbFileHandle;
use file::FilePos;
use std::fs;
// use std::sync::RwLock;

// use file::{LogFileId, FilePos};
// use std::sync::mpsc::{ Sender, Receiver};

// use file::meta::MetaFileOpe;


pub struct KVIndexStore {
    map: HashMap<String, FilePos>,}

impl KVIndexStore {
    pub fn create() -> KVIndexStore {
        return KVIndexStore {
            map: HashMap::new()
        };
    }
    pub fn set(&mut self, k: String, v: FilePos) {
        // self.map.get_mut()
        self.map.entry(k).and_modify(|v1| {
            *v1 = (v).clone();
        }).or_insert(v);
    }
    pub fn get(&self, k: &String) -> Option<&FilePos> {
        return self.map.get(k);
    }
    pub fn del(&mut self, k: &String) -> Option<FilePos> {
        self.map.remove(k)
    }
}

//重新设计我的kv数据库
// 文件需要存的信息，
// 只有一个文件，
// 读写使用阻塞接口
// 压缩到另一个文件后，删除原文件
struct PaKVCtxLock{
    pub store: KVIndexStore,
    pub file:DbFileHandle
}
impl PaKVCtxLock {
    pub fn readall(&mut self){
        self.file.iter_start();
        while let Some((v,pos))=self.file.iter_readline() {
            match v.ope{
                serial::KvOpeE::KvOpeSet { k, v } =>{
                     self.store.set(k, FilePos { offset:pos });},
                serial::KvOpeE::KvOpeDel { k } =>{
                     self.store.del(&k);
                    },
            }
        }
    }
}

//提供外部调用的接口
pub struct PaKVCtx {
    locked:RwLock<PaKVCtxLock>,
    path: String,
}

impl PaKVCtx {
    pub fn create() -> PaKVCtx {
        let path= "./default".to_string();
        fs::create_dir_all(&*path).unwrap();
        let kvctx=PaKVCtx{
            locked:RwLock::new(PaKVCtxLock {
                store:KVIndexStore::create(),
                file:DbFileHandle::create("./default/db".to_string()).unwrap()
            }), 
            path
        };
        kvctx.locked.write().readall();
        return kvctx;
    }
    pub fn create_with_name(name:String)->PaKVCtx{
        let path= format!("./{}",name);
        fs::create_dir_all(&*path).unwrap();
        let kvctx=PaKVCtx{
            locked:RwLock::new(PaKVCtxLock {
                store:KVIndexStore::create(),
                file:DbFileHandle::create(format!("./{}/db",name)).unwrap()
            }), 
            path
        };
        return kvctx;
    }

    //return old value
    pub fn set(&self, k: String, v: String) -> Option<String> {
        let ope=KvOpe::create(serial::KvOpeE::KvOpeSet { k:k.clone(), v });
        let mut hold=self.locked.write();
        let appendedpos=hold.file.append_log(ope.to_str()+"\n");
        println!("set pos{}",appendedpos.offset);
        hold.store.set(k, appendedpos);
        return None;
    }

    //return old value 
    pub fn del(&self, k: String) -> Option<String> {
        let mut hold=self.locked.write();
        if hold.store.get(&k).is_some(){
            let ope=KvOpe::create(serial::KvOpeE::KvOpeDel { k:k.clone() });
            hold.file.append_log(ope.to_str()+"\n");
            hold.store.del(&k);
            return None;
        }
        return None;
        // hold.store.set(k, hold.file.append_log(ope.to_str()+"\n"));
    }

    //return value
    pub fn get(&self, k: String) -> Option<String> {
        let mut hold=self.locked.write();
        let mut fp:Option<FilePos>=None;
        match hold.store.get(&k){
            Some(v) => {//有记录，读取
                fp=Some(v.clone());
            },
            None => {//没记录，
                return None;
            },
        }

        let ope=hold.file.get_log_by_pos(&fp.unwrap());
        match ope.ope {
            serial::KvOpeE::KvOpeSet { k, v } => Some(v),
            serial::KvOpeE::KvOpeDel { k:_ } => panic!(),
        }
    }

}