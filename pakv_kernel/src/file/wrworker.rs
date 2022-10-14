//通过读写worker进行写入操作，完成后

use crate::pakv::{ PaKVOpeId, KernelOtherWorkerSend2SelfMain};
use tokio::sync::mpsc::{Sender, Receiver};
use crate::pakv::file::serial::{KvOpe, KvOpeE};
use crate::pakv::file::wrworker::WRWorkerTask::{SetAppend, DelAppend, GetRead, TarFileSet};
use crate::pakv::file::{fileio, LogFileId, FilePos};
use std::fs::{OpenOptions, File};
use std::io::{SeekFrom, Seek};
use crate::pakv::file::compact::Compactor;
use std::collections::HashMap;
use crate::pakv::file::meta::MetaFileOpe;

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
    disactived_files:HashMap<u64,()>
}

impl PaKvFileWorker {
    pub fn new() -> PaKvFileWorker {
        return PaKvFileWorker { disactived_files: Default::default() };
    }
    fn pre_collect_dir(&mut self){
        let files=fileio::get_dirfiles_rank_by_time(fileio::get_folder_path());
        for (_a,b) in files{
            self.disactived_files.insert(b.0.id,());
        }
    }
    pub fn hold(&mut self, mut r: Receiver<WRWorkerTask>,send2main: KernelOtherWorkerSend2SelfMain) {
        struct CurFileStates{
            pub file:Option<File>,
            pub curpos:u64,
            pub fid:LogFileId,
            metafile_ope: MetaFileOpe,
        }
        // let mut appendcnt=0;
        let mut curf_states=CurFileStates{
            metafile_ope:MetaFileOpe::create(),
            file: None,
            curpos: 0,
            fid: LogFileId { id: 0 }
        };
        let mut compactor=None;
        fn tarfileset(fstate:&mut CurFileStates,tarfid:LogFileId){
            println!("tar file set in worker {}",tarfid.id);
            fstate.file.replace(OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(tarfid.get_pathbuf()).unwrap());
            fstate.curpos=fstate.file.as_mut().unwrap().seek(SeekFrom::End(0)).unwrap();
            fstate.metafile_ope.set_usertar_fid(tarfid.id);
            fstate.fid=tarfid;
        }

        loop {
            if let Some(rr) = r.blocking_recv() {
                match rr {
                    WRWorkerTask::SetAppend {
                        opeid, k, v
                    } => {
                        // appendcnt+=1;
                        let ope = KvOpe {
                            ope: KvOpeE::KvOpeSet { k, v }
                        };
                        if let Some(f)=&mut curf_states.file{
                            let p=fileio::file_append_log(
                                f,
                                serde_json::to_string(&ope).unwrap());
                            match ope.ope{
                                KvOpeE::KvOpeSet { k,..} => {
                                    send2main.after_set_append(opeid,FilePos{
                                        file_id: (&curf_states.fid).id,
                                        pos: (&curf_states).curpos
                                    },k);
                                    curf_states.curpos+=p;
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
                        if let Some(f)=&mut curf_states.file{
                            let p=fileio::file_append_log(
                                f,
                                serde_json::to_string(&ope).unwrap());
                            match ope.ope{
                                KvOpeE::KvOpeDel { k,.. } => {
                                    send2main.after_del_append(opeid,k);
                                }
                                _ => {}
                            }
                            curf_states.curpos+=p;
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
                            KvOpeE::KvOpeSet { k:_, v } => {
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
                        //从未激活文件移出
                        self.disactived_files.remove(&tarfid.id);
                        tarfileset(&mut curf_states,tarfid);
                        self.pre_collect_dir();
                    }
                }
                //没有正在压缩的任务
                if (&compactor).is_none(){
                    if Compactor::if_need_compact(curf_states.curpos){
                        // println!("append cnt {} {}",appendcnt,curf_states.fid.id);
                        // appendcnt=0;
                        compactor=Some(Compactor::new());
                        if let Some(comp)=&mut compactor{
                            let mut fid =curf_states.fid.clone();
                            {
                                // {//1.从state取出当前文件句柄File
                                //     let mut f = None;
                                //     std::mem::swap(&mut f, &mut curf_states.file);
                                //     comp.disactived_files.insert(fid.id, ());
                                // }
                                //2.获取kv数据并设置要压缩的kv数据
                                comp.kv = send2main.clone_kv_hash();
                                //3.设置当前目标写入的文件，压缩目标文件需要避开
                                comp.curfid = fid.id+1;
                            }
                            fid.id+=1;
                            tarfileset(&mut curf_states,
                                       fid);
                            comp.calc_kvranked();
                            comp.startpact(&send2main);
                            std::mem::swap(&mut comp.disactived_files,&mut self.disactived_files);
                            compactor=None;
                        }
                    }
                }
            } else {
                break;
            }
        }
    }
}

pub async fn start_worker(send2main: KernelOtherWorkerSend2SelfMain) -> KernelMain2WorkerSend {
    let (s,  r) =
        KernelMain2WorkerSend::new();

    tokio::task::spawn_blocking(
    // std::thread::spawn(
        move ||{
        PaKvFileWorker::new().hold(r,send2main);
    });
    //批量处理写入任务，
    // tokio::task::spawn_blocking(move || {
    // });

    s
}