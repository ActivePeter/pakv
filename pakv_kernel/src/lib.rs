// pub mod file;
// pub mod channel_caller;
pub mod file;
pub mod serial;
pub mod meta;
mod compress;
mod test;
// mod client_match_msg;

use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use serial::KvOpe;
use file::DbFileHandle;
use file::FilePos;
use std::fs;
use crate::compress::Compresser;
use std::sync::atomic::Ordering;
use std::time::UNIX_EPOCH;
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
pub(crate) struct PaKVCtxLock{
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
        Compresser::get().update_threshold(self.file.get_r_offset());

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

    fn wait_for_comp(&self,hold:&mut PaKVCtxLock,comp:&Compresser){
        {
            //等压缩让出
            let mut res =comp.compact_res.lock();
            if !comp.compacting.load(Ordering::Relaxed)&&res.is_some(){
                let mut none=None;
                std::mem::swap(&mut none,&mut res);
                let res=none.unwrap();
                hold.store.map=res.map;
                hold.file=res.fhandle;
            }
        }
    }

    //return old value
    pub fn set(&self, k: String, v: String) -> Option<String> {
        let comp=Compresser::get();
        comp.user_reading.store(true,Ordering::Relaxed);
        // println!("lock3");
        let mut hold=self.locked.write();
        self.wait_for_comp(&mut hold,comp);

        let ope=KvOpe::create(serial::KvOpeE::KvOpeSet { k:k.clone(), v });
        let appendedpos=hold.file.append_log(ope.to_str()+"\n");
        // println!("set pos{}",appendedpos.offset);
        hold.store.set(k.clone(), appendedpos.clone());

        comp.user_readtime.store(std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64, Ordering::Relaxed);
        comp.start_compress_if_need(hold.file.get_w_offset(), &hold.store, self.path.clone(),Some((k,appendedpos)),true);
        comp.user_reading.store(false,Ordering::Relaxed);
        // println!("lock3_");

        // println!("lock3__");
        return None;
    }

    //return old value 
    pub fn del(&self, k: String) -> Option<String> {

        let mut hold=self.locked.write();
        let comp=Compresser::get();
        self.wait_for_comp(&mut hold,comp);
        if hold.store.get(&k).is_some(){
            //等压缩发现并让出
            comp.user_reading.store(true,Ordering::Relaxed);
            // println!("lock1");


            let ope=KvOpe::create(serial::KvOpeE::KvOpeDel { k:k.clone() });
            hold.file.append_log(ope.to_str()+"\n");
            hold.store.del(&k);

            comp.user_readtime.store(std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64, Ordering::Relaxed);
            comp.start_compress_if_need(hold.file.get_w_offset(),&hold.store,self.path.clone(),Some((k,FilePos{offset:0})),false);
            comp.user_reading.store(false,Ordering::Relaxed);

            return None;
        }
        return None;
        // hold.store.set(k, hold.file.append_log(ope.to_str()+"\n"));
    }

    //return value
    pub fn get(&self, k: String) -> Option<String> {
        let comp=Compresser::get();
        comp.user_reading.store(true,Ordering::Relaxed);
        // println!("lock2");

        let mut hold=self.locked.write();
        self.wait_for_comp(&mut hold,comp);

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
        comp.user_readtime.store(std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64, Ordering::Relaxed);
        comp.user_reading.store(false,Ordering::Relaxed);
        match ope.ope {
            serial::KvOpeE::KvOpeSet { k, v } => Some(v),
            serial::KvOpeE::KvOpeDel { k:_ } => panic!(),
        }
    }

}