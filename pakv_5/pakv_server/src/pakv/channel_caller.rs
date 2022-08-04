use crate::pakv::{KvOpe, KvOpeCmd};
use tokio::sync;
use tokio::sync::mpsc::Receiver as TokioMpscReceiver;
use tokio::sync::mpsc::Sender as TokioMpscSender;
use crate::pakv::file::LogFileId;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PaKVCtxChanCallerForSys{
    worker_sendin_chan:sync::mpsc::Sender<KvOpeCmd>
}

impl PaKVCtxChanCallerForSys{
    pub fn new(chan:sync::mpsc::Sender<KvOpeCmd>) -> PaKVCtxChanCallerForSys {
        PaKVCtxChanCallerForSys{
            worker_sendin_chan:chan
        }
    }
    pub async fn update_k_positions(&self, fid:LogFileId, map_k2pos:HashMap<String,u64>) -> TokioMpscReceiver<bool> {
        //receive end and notify from;
        let (tx,rx):(TokioMpscSender<bool>,TokioMpscReceiver<bool>)=sync::mpsc::channel(10);

        self.worker_sendin_chan.send(KvOpeCmd::SysKvOpeBatchUpdate { fid, map_k2pos ,
            resp:tx
        }).await;

        rx
    }
    pub async fn end_compact(&self){
        self.worker_sendin_chan.send(KvOpeCmd::SysKvOpeCompactEnd {}).unwrap();
    }
}

#[derive(Clone)]
pub struct App2KernelSender {
    sendin_chan:sync::mpsc::Sender<KvOpeCmd>
}
impl App2KernelSender {
    pub fn new(chan:sync::mpsc::Sender<KvOpeCmd>) -> App2KernelSender {
        App2KernelSender {
            sendin_chan:chan
        }
    }
    pub async fn get(&self,s:String){
        // let (tx,rx)=UserKvOpe::create_get_chan();
        self.sendin_chan.send(KvOpeCmd::KvOpeGet {
            k: s,
        }).await.unwrap();
    }
    pub async fn set(&self,k:String,v:String){
        self.sendin_chan.send(KvOpeCmd::KvOpeSet {
            k,
            v,
        }).await.unwrap();
    }
    pub async fn del(&self, k:String){
        self.sendin_chan.send(KvOpeCmd::KvOpeDel {
            k,
            // resp: tx
        }).await.unwrap();
        // return rx.recv().unwrap();
    }
}