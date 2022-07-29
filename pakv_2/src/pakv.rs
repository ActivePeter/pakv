use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File, read_dir};
use std::{fs, thread};
use std::io::{Error, Read, Write, Seek, Cursor, SeekFrom};
use std::sync;
use crate::file::{file_check, KvOpe, KvOpeE, LogFileId, FilePos};
use std::sync::mpsc::{RecvError, Sender, Receiver};
use crate::file;


pub struct KVStore{
    map:HashMap<String,FilePos>
}
impl KVStore{
    pub fn create() -> KVStore {
        return KVStore{
            map:HashMap::new()
        }
    }
    // pub fn do_ope(&mut self,ope:&KvOpe){
    //     match &ope.ope{
    //         KvOpeE::KvOpeSet {k,v } => {
    //             self.set(k.clone(), v);
    //         }
    //         KvOpeE::KvOpeDel { k} => {
    //             self.del(k.clone());
    //         }
    //     }
    // }
    pub fn set(&mut self,k:String,v:&FilePos){
        self.map.entry(k).and_modify(|mut v1|{
            *v1= (*v).clone();
        }).or_insert( (*v).clone());
    }
    pub fn get(&self, k:String) -> Option<&FilePos> {
        return self.map.get(&k);
    }
    pub fn del(&mut self, k:String) -> Option<FilePos> {
        self.map.remove(&k)
    }
}

pub enum UserKvOpe{
    KvOpeSet{k:String,v:String,
        // resp:sync::mpsc::Sender<bool>
    },
    KvOpeDel{k:String,
        resp:sync::mpsc::Sender<bool>
    },
    KvOpeGet{k:String,
        resp:sync::mpsc::Sender<Option<String>>},
}
impl UserKvOpe{
    pub fn create_get_chan() -> (Sender<Option<String>>, Receiver<Option<String>>) {
        let c:(
        sync::mpsc::Sender<Option<String>>,
        sync::mpsc::Receiver<Option<String>>
        )=sync::mpsc::channel();

        c
    }
    pub fn create_del_chan() -> (Sender<bool>,
                                 Receiver<bool>) {
        let c:(
            sync::mpsc::Sender<bool>,
            sync::mpsc::Receiver<bool>
        )=sync::mpsc::channel();

        c
    }
}
pub struct PaKVCtx{
    pub store:KVStore,
    pub tarfid:LogFileId,//使用前已经确保文件可写
}
impl PaKVCtx{
    pub fn create() -> PaKVCtx {
        return PaKVCtx{
            store: KVStore::create(),
            tarfid: LogFileId{ id: 1 }
        }
    }

    pub fn set(&mut self, k:String, v:String){

        let ope=KvOpe{
            ope: KvOpeE::KvOpeSet {k:k.clone(),v:v.clone()}
        };
            //1.log
            let pos=file::file_append_log(&self.tarfid.get_pathbuf(),ope.to_line_str().unwrap()).unwrap();
            //2.mem
            self.store.set(k, &FilePos {
                file_id: self.tarfid.id,
                pos
            });



        // self.append_log(ope.to_line_str().unwrap());

    }
    pub fn del(&mut self, k:String) -> Option<FilePos> {
        //1.log
        let ope=KvOpe{
            ope: KvOpeE::KvOpeDel {k:k.clone()}
        };
        let pos=file::file_append_log(&self.tarfid.get_pathbuf(),ope.to_line_str().unwrap()).unwrap();
        // self.append_log(ope.to_line_str().unwrap());
        self.store.del(k)
    }
    pub fn get(&self, k:String) -> Option<String> {
        let res=self.store.get(k);
        match res{
            None => {
                None
            }
            Some(pos) => {
                let line=pos.readline();
                // if let Some(l)=line.clone(){
                //     println!("get {}",l);
                // }
                if let Some(l)=line{
                    let ope=KvOpe::from_str(&*l);
                    if let Ok(v)=ope{
                        match v.ope{
                            KvOpeE::KvOpeSet { k,  v } => {
                                return Some(v);
                            }
                            _=>{
                                return None
                            }
                        }
                    }
                }
                None
            }
        }
    }
}
pub fn start_kernel() -> Sender<UserKvOpe> {
    let mut ctx=PaKVCtx::create();
    file::file_check(&mut ctx);
    let (tx,rx)
        :(sync::mpsc::Sender<UserKvOpe>,
          sync::mpsc::Receiver<UserKvOpe>)
        =sync::mpsc::channel();
    fn handle_ope(ctx:&mut PaKVCtx, ope:UserKvOpe){
        match ope{
            UserKvOpe::KvOpeSet {
                k,v } => {
                ctx.set(k,v);
            }
            UserKvOpe::KvOpeDel { k,resp } => {
                match ctx.del(k){
                    None => {
                        resp.send(false);
                    }
                    Some(_) => {
                        resp.send(true);
                    }
                }
            }
            UserKvOpe::KvOpeGet {
                k,resp } => {
                match ctx.get(k){
                    None => {
                        resp.send(None);
                    }
                    Some(v) => {
                        resp.send(Some(v.clone()));
                    }
                }
            }
        }
    }
    let handle = thread::spawn(move || {
        loop {
            let r=rx.recv();
            match r{
                Ok(rr) => {
                    handle_ope(&mut ctx,rr);
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    tx
}

// #[cfg(test)]
// mod tests {
//     // Note this useful idiom: importing names from outer (for mod tests) scope.
//     use super::*;
//
//     #[test]
//     fn test_get_none() {
//         let mut kvs=KVStore::create();
//         // This assert would fire and test will fail.
//         // Please note, that private functions can be tested too!
//         assert_eq!(kvs.get(("1").to_owned()), None);
//         assert_eq!(kvs.get("2".to_owned()), None);
//     }
//
//     #[test]
//     fn test_add_get() {
//         let mut kvs=KVStore::create();
//         kvs.set("1".to_owned(),"111".to_owned());
//         kvs.set("2".to_owned(),"222".to_owned());
//         // This assert would fire and test will fail.
//         // Please note, that private functions can be tested too!
//         assert_eq!(kvs.get("1".to_owned()).unwrap(), &"111".to_owned());
//         assert_eq!(kvs.get("2".to_owned()).unwrap(), &"222".to_owned());
//     }
//
//     #[test]
//     fn test_del() {
//         let mut kvs=KVStore::create();
//         kvs.set("1".to_owned(),"111".to_owned());
//         kvs.set("2".to_owned(),"222".to_owned());
//         kvs.del("1".to_owned());
//         kvs.del("2".to_owned());
//         // This assert would fire and test will fail.
//         // Please note, that private functions can be tested too!
//         assert_eq!(kvs.get("1".to_owned()), None);
//         assert_eq!(kvs.get("2".to_owned()), None);
//     }
// }