//通过读写worker进行写入操作，完成后

use crate::pakv::{PaKVCtx, PaKVOpeId, KernelOtherWorkerSend2SelfMain};
use std::future::Future;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::pakv::file::serial::{KvOpe, KvOpeE};
use crate::pakv::file::wrworker::WRWorkerTask::{SetAppend, DelAppend, GetRead, TarFileSet};
use crate::pakv::file::{fileio, LogFileId, FilePos};
use std::fs::{OpenOptions, File};
use std::io::{SeekFrom, Seek};

#[derive(Debug)]
pub enum WRWorkerTask {
    SetAppend {
        opeid: PaKVOpeId,
        k: String,
        v: String,
    },
    DelAppend {
        opeid: PaKVOpeId,
        k: String,
    },
    GetRead {
        opeid: PaKVOpeId,
        pos: FilePos,
    },
    TarFileSet {
        tarfid: LogFileId
    },
}

pub struct KernelMain2WorkerSend {
    sender: Sender<WRWorkerTask>,
}

impl KernelMain2WorkerSend {
    pub fn new() -> (KernelMain2WorkerSend, Receiver<WRWorkerTask>) {
        let (s, r)
            : (Sender<WRWorkerTask>, Receiver<WRWorkerTask>)
            = tokio::sync::mpsc::channel(10);
        (KernelMain2WorkerSend {
            sender: s
        }, r)
    }
    pub async fn set_append(&self, opeid: PaKVOpeId, k: String, v: String) {
        self.sender.send(SetAppend {
            opeid,
            k,
            v,
        }).await.unwrap()
    }
    pub async fn del_append(&self, opeid: PaKVOpeId, k: String) {
        self.sender.send(DelAppend {
            opeid,
            k,
        }).await.unwrap()
    }
    pub async fn get_read(&self, opeid: PaKVOpeId, pos: FilePos) {
        self.sender.send(GetRead {
            opeid,
            pos,
        }).await.unwrap();
    }
    pub async fn tarfile_set(&self, tarfid: LogFileId) {
        self.sender.send(TarFileSet {
            tarfid
        }).await.unwrap();
    }
}

struct PaKvFileWorker {
    // pub
}

impl PaKvFileWorker {
    pub fn new() -> PaKvFileWorker {
        return PaKvFileWorker {};
    }

    pub fn hold(&mut self, mut r: Receiver<WRWorkerTask>,send2main: KernelOtherWorkerSend2SelfMain) {


        let mut file1=None;
        let mut curpos:u64=0;
        let mut fid=LogFileId{
            id: 0
        };
        loop {
            if let Some(rr) = r.blocking_recv() {
                match rr {
                    WRWorkerTask::SetAppend {
                        opeid, k, v
                    } => {
                        let ope = KvOpe {
                            ope: KvOpeE::KvOpeSet { k, v }
                        };
                        if let Some(f)=&mut file1{
                            let p=fileio::file_append_log(
                                f,
                                serde_json::to_string(&ope).unwrap());
                            match ope.ope{
                                KvOpeE::KvOpeSet { k,..} => {
                                    send2main.after_set_append(opeid,FilePos{
                                        file_id: fid.id,
                                        pos: curpos
                                    },k);
                                    curpos+=p;
                                }
                                _ => {panic!("impossible")}
                            }
                        }else{
                            panic!("f not exist");
                        }
                    }
                    WRWorkerTask::DelAppend {
                        opeid, k
                    } => {
                        let ope = KvOpe {
                            ope: KvOpeE::KvOpeDel { k }
                        };
                        if let Some(f)=&mut file1{
                            let p=fileio::file_append_log(
                                f,
                                serde_json::to_string(&ope).unwrap());
                            match ope.ope{
                                KvOpeE::KvOpeDel { k,.. } => {
                                    send2main.after_del_append(opeid,k);
                                }
                                _ => {}
                            }
                            curpos+=p;
                        }else{
                            panic!("f not exist");
                        }
                    }
                    WRWorkerTask::GetRead {
                        opeid, pos
                    } => {
                        let l = pos.readline().unwrap();
                        // if let Some(l)=line

                        let ope = KvOpe::from_str(&*l).unwrap();
                        match ope.ope {
                            //为set记录，正确
                            KvOpeE::KvOpeSet { k, v } => {
                                send2main.after_get_read(opeid,v);
                            }
                            _ => {
                                panic!("hash record delete? imposible!")
                            }
                        }
                    }
                    WRWorkerTask::TarFileSet {
                        tarfid
                    } => {
                        println!("tar file set in worker");
                        file1 = Some(OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(tarfid.get_pathbuf()).unwrap());
                        curpos=file1.as_mut().unwrap().seek(SeekFrom::End(0)).unwrap();
                        fid=tarfid;
                    }
                }
            } else {
                break;
            }
        }
    }
}

pub async fn start_worker(send2main: KernelOtherWorkerSend2SelfMain) -> KernelMain2WorkerSend {
    let (s, mut r) =
        KernelMain2WorkerSend::new();

    // tokio::task::spawn_blocking(
    std::thread::spawn(
        move ||{
        PaKvFileWorker::new().hold(r,send2main);
    });
    //批量处理写入任务，
    // tokio::task::spawn_blocking(move || {
    // });

    s
}