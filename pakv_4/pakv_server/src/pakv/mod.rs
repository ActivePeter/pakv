
pub mod file;
pub mod channel_caller;

use std::collections::{HashMap, HashSet};
use std::{ thread};
use std::sync;
use file::{ LogFileId, FilePos};
use std::sync::mpsc::{ Sender, Receiver};

use file::meta::MetaFileOpe;
use file::serial::{KvOpe, KvOpeE};
use file::compact::CompactCtx;
use crate::pakv::channel_caller::{ PaKVCtxChanCallerForUser, PaKVCtxChanCallerForSys};
// use crate::r#mod;


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
        // self.map.get_mut()
        self.map.entry(k).and_modify(| v1|{
            *v1= (*v).clone();
        }).or_insert( (*v).clone());
    }
    pub fn get(&self, k:&String) -> Option<&FilePos> {
        return self.map.get(k);
    }
    pub fn del(&mut self, k:&String) -> Option<FilePos> {
        self.map.remove(k)
    }
}

pub enum UserKvOpe{
    KvOpeSet{k:String,v:String,
        resp:sync::mpsc::Sender<bool>
    },
    KvOpeDel{k:String,
        resp:sync::mpsc::Sender<bool>
    },
    KvOpeGet{k:String,
        resp:sync::mpsc::Sender<Option<String>>},
    SysKvOpeBatchUpdate{
        fid:LogFileId,map_k2pos:HashMap<String,u64>,
        resp:Sender<bool>
    },
    SysKvOpeCompactEnd{

    }
}
impl UserKvOpe{
    pub fn create_set_chan() -> (Sender<bool>, Receiver<bool>) {
        UserKvOpe::create_del_chan()
    }
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
    pub sys_chan_caller:PaKVCtxChanCallerForSys,
    pub user_chan_caller:PaKVCtxChanCallerForUser,
    pub compacting:bool,//标记compact，compact期间，需要将操作存入特殊位置
    pub user_opek_whencompact:HashSet<String>,
    pub meta_file_ope:MetaFileOpe
}
impl PaKVCtx{
    pub fn create(sendin_chan:sync::mpsc::Sender<UserKvOpe>) -> PaKVCtx {
        return PaKVCtx{
            store: KVStore::create(),
            tarfid: LogFileId{ id: 1 },
            sys_chan_caller:PaKVCtxChanCallerForSys::new(sendin_chan.clone()),
            user_chan_caller:PaKVCtxChanCallerForUser::new(sendin_chan),
            compacting: false,
            user_opek_whencompact: Default::default(),
            meta_file_ope:MetaFileOpe::create()
        }
    }

    pub fn set(&mut self, k:String, v:String) -> u64 {

        let ope=KvOpe{
            ope: KvOpeE::KvOpeSet {k:k.clone(),v:v.clone()}
        };
            //1.log
            let pos= file::file_append_log(&self.tarfid.get_pathbuf(), ope.to_line_str().unwrap()).unwrap();
            //2.mem
            self.store.set(k.clone(), &FilePos {
                file_id: self.tarfid.id,
                pos
            });

        if self.compacting{
            self.user_opek_whencompact.insert(k);
        }else{
            CompactCtx::compact_ifneed(self, pos);
        }
        // self.append_log(ope.to_line_str().unwrap());
        pos
    }
    pub fn del(&mut self, k:&String) -> Option<FilePos> {
        //1.log
        let ope=KvOpe{
            ope: KvOpeE::KvOpeDel {k:k.clone()}
        };
        let pos= file::file_append_log(&self.tarfid.get_pathbuf(), ope.to_line_str().unwrap()).unwrap();
        // self.append_log(ope.to_line_str().unwrap());
        let ret=self.store.del(k);

        if self.compacting{
            self.user_opek_whencompact.insert(k.clone());
        }else{
            CompactCtx::compact_ifneed(self, pos);
        }

        ret
    }
    pub fn get(&self, k:&String) -> Option<String> {
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
                            KvOpeE::KvOpeSet { k:_,  v } => {
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
pub fn start_kernel() -> PaKVCtxChanCallerForUser {
    let (tx,rx)
        :(sync::mpsc::Sender<UserKvOpe>,
          sync::mpsc::Receiver<UserKvOpe>)
        =sync::mpsc::channel();
    let mut ctx=PaKVCtx::create(tx.clone());
    file::file_check(&mut ctx);
    fn handle_ope(ctx:&mut PaKVCtx, ope:UserKvOpe){
        match ope{
            UserKvOpe::KvOpeSet {
                k,v, resp
            } => {
                ctx.set(k,v);
                resp.send(true).unwrap();
            }
            UserKvOpe::KvOpeDel { k,resp } => {
                match ctx.del(&k){
                    None => {
                        resp.send(false).unwrap();
                    }
                    Some(_) => {
                        resp.send(true).unwrap();
                    }
                }
            }
            UserKvOpe::KvOpeGet {
                k,resp } => {
                match ctx.get(&k){
                    None => {
                        resp.send(None).unwrap();
                    }
                    Some(v) => {
                        resp.send(Some(v)).unwrap();
                    }
                }
            }
            UserKvOpe::SysKvOpeBatchUpdate {fid,map_k2pos,resp  } => {
                ctx.compacting=true;
                for (k,pos) in map_k2pos{
                    if !ctx.user_opek_whencompact.contains(&k) {
                        ctx.store.set(k, &FilePos {
                            file_id: fid.id,
                            pos
                        })
                    }
                }
                resp.send(true).unwrap();
            }
            UserKvOpe::SysKvOpeCompactEnd { .. } => {
                ctx.compacting=false;
                ctx.user_opek_whencompact.clear();
            }
        }
    }
    let ret=ctx.user_chan_caller.clone();
    let _handle = thread::spawn(move || {
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

    ret
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