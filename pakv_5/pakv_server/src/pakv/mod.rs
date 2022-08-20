pub mod file;
pub mod channel_caller;
mod kernel2app_msg;
// mod client_match_msg;

use std::collections::{HashMap, HashSet};
use file::{LogFileId, FilePos};
// use std::sync::mpsc::{ Sender, Receiver};

use file::meta::MetaFileOpe;
// use file::compact::CompactCtx;
// use crate::pakv::channel_caller::{App2KernelSender, PaKVCtxChanCallerForSys};
use tokio::sync::mpsc::{Sender, Receiver};
use crate::pakv::file::wrworker::{ KernelMain2WorkerSend};
// use crate::r#mod;


pub struct KVStore {
    map: HashMap<String, FilePos>,
}

impl KVStore {
    pub fn create() -> KVStore {
        return KVStore {
            map: HashMap::new()
        };
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
    pub fn set(&mut self, k: String, v: &FilePos) {
        // self.map.get_mut()
        self.map.entry(k).and_modify(|v1| {
            *v1 = (*v).clone();
        }).or_insert((*v).clone());
    }
    pub fn get(&self, k: &String) -> Option<&FilePos> {
        return self.map.get(k);
    }
    pub fn del(&mut self, k: &String) -> Option<FilePos> {
        self.map.remove(k)
    }
}


//在wrworker执行完后传递给主循环，
// 主循环在交给内核selfconsume，进行后续的set
// consume->一个结果给app，app发给客户端，
#[derive(Debug)]
pub enum KernelWorker2Main {
    AfterSetAppend{
        opeid:PaKVOpeId,
        k:String,
        pos:FilePos
    },
    AfterGetRead{opeid:PaKVOpeId, v:String},
    AfterDelAppend{
        opeid:PaKVOpeId,
        k:String,
    },
    GetKVHashClone{
        resp:tokio::sync::oneshot::Sender<HashMap<String, FilePos>>
    },
    BunchUpdate{
        fileid:u64,
        k2off:Vec<(String,usize)>,
    }
}

#[derive(Clone,Debug)]
pub enum PaKvOpeResult {
    SetResult {
    },
    DelResult {
        succ: bool,
    },
    GetResult {
        v: Option<String>,
    },
}
pub struct  KernelToAppMsg {
    pub opeid: PaKVOpeId,
    pub res:PaKvOpeResult
}

//内核其他协程向主协程发送
pub struct KernelOtherWorkerSend2SelfMain {
    sender: Sender<KernelWorker2Main>,
}

impl KernelOtherWorkerSend2SelfMain {
    pub fn new(sender: Sender<KernelWorker2Main>) -> KernelOtherWorkerSend2SelfMain {
        KernelOtherWorkerSend2SelfMain {
            sender
        }
    }
    pub fn clone_kv_hash(&self) -> HashMap<String, FilePos> {
        let (t,r)=tokio::sync::oneshot::channel();
        self.sender.blocking_send(KernelWorker2Main::GetKVHashClone {
            resp:t
        }).unwrap();
        let res=r.blocking_recv().unwrap();

        res
    }
    pub fn after_set_append(
        &self, opeid:PaKVOpeId,pos:FilePos,k:String) {
        self.sender.blocking_send(
            KernelWorker2Main::AfterSetAppend {
                opeid,
                pos,k
            }
        ).unwrap();
    }
    pub fn after_del_append(
        &self, opeid:PaKVOpeId,k:String) {
        self.sender.blocking_send(
            KernelWorker2Main::AfterDelAppend {
                opeid,
                k
            }
        ).unwrap();
    }
    pub fn after_get_read(
        &self, opeid:PaKVOpeId,v:String) {
        self.sender.blocking_send(
            KernelWorker2Main::AfterGetRead {
                opeid,
                v
            }
        ).unwrap();
    }
}

pub type PaKVOpeId = u64;

pub struct PaKVCtx {
    pub store: KVStore,
    pub tarfid: LogFileId,
    //使用前已经确保文件可写
    // pub sys_chan_caller:PaKVCtxChanCallerForSys,
    // pub user_chan_caller:PaKVCtxChanCallerForUser,
    pub compacting: bool,
    //标记compact，compact期间，需要将操作存入特殊位置
    pub user_opek_whencompact: HashSet<String>,
    pub meta_file_ope: MetaFileOpe,
    // kernel2app_sender: Sender<KernelToAppMsg>,
    // kernel2app_receiver:Receiver<KernelToAppMsg>,
    opeid: PaKVOpeId,
    main_2_fileworker: KernelMain2WorkerSend,
}

impl PaKVCtx {
    pub async fn create() -> (PaKVCtx, Receiver<KernelWorker2Main>) {
        let (t, r):
            (Sender<KernelWorker2Main>, Receiver<KernelWorker2Main>)
            = tokio::sync::mpsc::channel(10);

        let worker_2_main =
            KernelOtherWorkerSend2SelfMain::new(t.clone());
        let main_2_worker =
            file::wrworker::start_worker(worker_2_main).await;

        let mut kvctx = PaKVCtx {
            store: KVStore::create(),
            tarfid: LogFileId { id: 1 },
            // sys_chan_caller:PaKVCtxChanCallerForSys::new(sendin_chan.clone()),
            // user_chan_caller:PaKVCtxChanCallerForUser::new(sendin_chan),
            compacting: false,
            user_opek_whencompact: Default::default(),
            meta_file_ope: MetaFileOpe::create(),
            opeid: 0,
            // kernel2app_sender: t,
            main_2_fileworker: main_2_worker,
        };
        file::fileio::file_check(&mut kvctx).await;
        kvctx.main_2_fileworker.tarfile_set(kvctx.tarfid.clone()).await;

        return (kvctx, r);
    }
    fn get_opeid(&mut self) -> PaKVOpeId {
        let ret = self.opeid;
        self.opeid += 1;
        ret
    }
    pub async fn set(&mut self, k: String, v: String) -> PaKVOpeId {
        let opeid = self.get_opeid();
        self.main_2_fileworker.set_append(opeid, k, v).await;
        // let ope=KvOpe{
        //     ope: KvOpeE::KvOpeSet {k:k.clone(),v:v.clone()}
        // };
        //     //1.log
        //     let pos= file::file_append_log(&self.tarfid.get_pathbuf(), ope.to_line_str().unwrap()).unwrap();
        //     //2.mem
        // self.store.set(k.clone(), &FilePos {
        //     file_id: self.tarfid.id,
        //     pos
        // });
        //
        // if self.compacting{
        //     self.user_opek_whencompact.insert(k);
        // }else{
        //     CompactCtx::compact_ifneed(self, pos);
        // }
        // self.append_log(ope.to_line_str().unwrap());
        opeid
    }

    pub async fn del(&mut self, k: String) -> PaKVOpeId {
        let opeid = self.get_opeid();
        self.main_2_fileworker.del_append(opeid, k).await;
        //
        // //1.log
        // let ope=KvOpe{
        //     ope: KvOpeE::KvOpeDel {k:k.clone()}
        // };
        // let pos= file::file_append_log(&self.tarfid.get_pathbuf(), ope.to_line_str().unwrap()).unwrap();
        // // self.append_log(ope.to_line_str().unwrap());
        // let ret=self.store.del(k);
        //
        // if self.compacting{
        //     self.user_opek_whencompact.insert(k.clone());
        // }else{
        //     CompactCtx::compact_ifneed(self, pos);
        // }

        opeid
    }
    pub async fn get(&mut self, k: String) -> Option<PaKVOpeId> {
        let r = self.store.get(&k);
        if let Some(pos) = r {
            let pos=pos.clone();
            let opeid = self.get_opeid();
            self.main_2_fileworker.get_read(
                opeid,
                pos).await;

            return Some(opeid);
        }
        // let res=self.store.get(k);
        // match res{
        //     None => {
        //         None
        //     }
        //     Some(pos) => {
        //         let line=pos.readline();
        //         // if let Some(l)=line.clone(){
        //         //     println!("get {}",l);
        //         // }
        //         if let Some(l)=line{
        //             let ope=KvOpe::from_str(&*l);
        //             if let Ok(v)=ope{
        //                 match v.ope{
        //                     KvOpeE::KvOpeSet { k:_,  v } => {
        //                         return Some(v);
        //                     }
        //                     _=>{
        //                         return None
        //                     }
        //                 }
        //             }
        //         }
        //         None
        //     }
        // }

        None
    }
    //在操作完文件后，操作内存数据
    pub async fn consume_selfmsg(&mut self, selfmsg: KernelWorker2Main)
        ->Option<KernelToAppMsg>{
        match selfmsg {
            KernelWorker2Main::AfterSetAppend {
                opeid,k,pos
            } => {
                self.store.set(k,&pos);
                return Some(KernelToAppMsg {
                    opeid,
                    res:PaKvOpeResult::SetResult {}
                });
            }
            KernelWorker2Main::AfterGetRead {
                opeid,v
            } => {
                return Some(KernelToAppMsg {
                    opeid,res:PaKvOpeResult::GetResult {
                        v:Some(v)
                    }
                });
            }
            KernelWorker2Main::AfterDelAppend {
                opeid,k
            } => {
                let r=self.store.del(&k);

                return Some(KernelToAppMsg {
                    opeid,res:PaKvOpeResult::DelResult {
                        succ:r.is_some()
                    }
                });
            }
            KernelWorker2Main::GetKVHashClone {
                resp}=>{
                resp.send(self.store.map.clone()).unwrap();
                None
            }
            KernelWorker2Main::BunchUpdate {
                fileid,k2off
            }=>{
                // let mut ks=vec![];
                for(k,off) in &k2off{
                    let v=self.store.map
                        .get_mut(k).unwrap();
                    v.file_id=fileid;
                    v.pos=*off as u64;
                }

                None
            }
        }
    }
}
// pub async fn start_kernel() -> App2KernelSender {
//     let (tx,rx)
//         :(sync::mpsc::Sender<KvOpe>,
//           sync::mpsc::Receiver<KvOpe>)//kv内核循环，用于接收用户和来自内核的数据库操作
//         =sync::mpsc::channel(10);
//     let mut ctx=PaKVCtx::create(tx.clone());
//     file::file_check(&mut ctx);
//     fn handle_ope(ctx:&mut PaKVCtx, ope: KvOpe){
//         match ope{
//             KvOpeCmd::KvOpeSet {
//                 k,v
//             } => {
//                 ctx.set(k,v);
//
//             }
//             KvOpeCmd::KvOpeDel { k } => {
//                 match ctx.del(&k){
//                     None => {
//                         resp.send(false).unwrap();
//                     }
//                     Some(_) => {
//                         resp.send(true).unwrap();
//                     }
//                 }
//             }
//             KvOpeCmd::KvOpeGet {
//                 k } => {
//                 match ctx.get(&k){
//                     None => {
//                         resp.send(None).unwrap();
//                     }
//                     Some(v) => {
//                         resp.send(Some(v)).unwrap();
//                     }
//                 }
//             }
//             KvOpe::SysKvOpeBatchUpdate {fid,map_k2pos,resp  } => {
//                 ctx.compacting=true;
//                 for (k,pos) in map_k2pos{
//                     if !ctx.user_opek_whencompact.contains(&k) {
//                         ctx.store.set(k, &FilePos {
//                             file_id: fid.id,
//                             pos
//                         })
//                     }
//                 }
//                 resp.send(true).unwrap();
//             }
//             KvOpe::SysKvOpeCompactEnd { .. } => {
//                 ctx.compacting=false;
//                 ctx.user_opek_whencompact.clear();
//             }
//         }
//     }
//     let ret=ctx.user_chan_caller.clone();
//     tokio::spawn(move || {
//         loop {
//             let r=rx.recv();
//             match r{
//                 Ok(rr) => {
//                     handle_ope(&mut ctx,rr);
//                 }
//                 Err(_) => {
//                     break;
//                 }
//             }
//         }
//     });
//
//     ret
// }

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