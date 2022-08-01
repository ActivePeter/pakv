use crate::pakv::UserKvOpe;
use std::sync;
use std::sync::mpsc::{Sender, Receiver};
use crate::pakv::file::LogFileId;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PaKVCtxChanCallerForSys{
    worker_sendin_chan:sync::mpsc::Sender<UserKvOpe>
}

impl PaKVCtxChanCallerForSys{
    pub fn new(chan:sync::mpsc::Sender<UserKvOpe>) -> PaKVCtxChanCallerForSys {
        PaKVCtxChanCallerForSys{
            worker_sendin_chan:chan
        }
    }
    pub fn update_k_positions(&self, fid:LogFileId, map_k2pos:HashMap<String,u64>) -> Receiver<bool> {
        //receive end and notify from;
        let (tx,rx):(Sender<bool>,Receiver<bool>)=sync::mpsc::channel();

        self.worker_sendin_chan.send(UserKvOpe::SysKvOpeBatchUpdate { fid, map_k2pos ,
            resp:tx
        }).unwrap();

        rx
    }
    pub fn end_compact(&self){
        self.worker_sendin_chan.send(UserKvOpe::SysKvOpeCompactEnd {}).unwrap();
    }
}

#[derive(Clone)]
pub struct PaKVCtxChanCallerForUser{
    sendin_chan:sync::mpsc::Sender<UserKvOpe>
}
impl PaKVCtxChanCallerForUser{
    pub fn new(chan:sync::mpsc::Sender<UserKvOpe>) -> PaKVCtxChanCallerForUser {
        PaKVCtxChanCallerForUser{
            sendin_chan:chan
        }
    }
    pub fn get(&self,s:String)->Option<String>{
        let (tx,rx)=UserKvOpe::create_get_chan();
        self.sendin_chan.send(UserKvOpe::KvOpeGet {
            k: s,
            resp: tx
        }).unwrap();
        let r_=rx.recv();
        match r_{
            Ok(s) => {s}
            Err(e) => {
                eprintln!("{}",e);
                None
            }
        }
    }
    pub fn set(&self,k:String,v:String){
        let (tx,rx)=UserKvOpe::create_set_chan();
        self.sendin_chan.send(UserKvOpe::KvOpeSet {
            k,
            v,
            resp: tx
        }).unwrap();
        rx.recv().unwrap();
    }
    pub fn del(&self, k:String) -> bool {
        let (tx,rx)=UserKvOpe::create_del_chan();
        self.sendin_chan.send(UserKvOpe::KvOpeDel {
            k,
            resp: tx
        }).unwrap();

        return rx.recv().unwrap();
    }
}