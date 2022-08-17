
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
// use crate::client2server::Client2ServerMsg;
use crate::server2client::Server2ClientSender;
use crate::net::msg2app_sender::NetMsg2App;
use crate::net::server_rw_eachclient::ClientId;
use crate::pakv::{PaKVCtx, PaKVOpeId, KernelWorker2Main, PaKvOpeResult};

pub enum Requester{
    NetClient(ClientId),
    ChannelSendBack(tokio::sync::oneshot::Sender<PaKvOpeResult>)
}
//持有pakv内核以及服务端状态，
pub struct PaKVServerApp {
    pub(crate) kernel:PaKVCtx,//内核运行上下文
    pub(crate) clients_cid2sender:HashMap<ClientId,Server2ClientSender>,//用于服务端向客户端发消息
    // pub(crate) kernelopeid_2_cid:HashMap<PaKVOpeId,ClientId>,//用于处理内核的执行结果
pub(crate) kernelopeid_2_requester:HashMap<PaKVOpeId,Requester>,
    kernel2apprecv:Receiver<KernelWorker2Main>
}
impl PaKVServerApp {
    pub async fn new() -> PaKVServerApp {
        let (kernel,r)=PaKVCtx::create().await;
        PaKVServerApp {
            kernel,
            clients_cid2sender: Default::default(),
            // kernelopeid_2_cid: Default::default(),
            kernelopeid_2_requester: Default::default(),
            kernel2apprecv:r
        }
    }
    pub fn consume_requester_of_opeid(&mut self,opeid:PaKVOpeId)->Requester{
        let requster=self.kernelopeid_2_requester.remove(&opeid);
        requster.unwrap()
    }
    pub fn client_in(&mut self,cid:ClientId,s2csender:Server2ClientSender){
        self.clients_cid2sender.insert(cid,s2csender);
    }
    pub fn client_out(&mut self,cid:ClientId){
        self.clients_cid2sender.remove(&cid);
    }

    pub async fn hold(
        mut self,
        mut net_recv:Receiver<NetMsg2App>){
        println!("app start wait for net msg");
        loop{
            // let rr=server_ctx.kernel2apprecv.recv().await;
            // if let Some(msg)=rr{
            //
            // }
            // let r=client2server_recv.recv().await;
            tokio::select! {
                msg = self.kernel2apprecv.recv() => {
                    self.consume_kernel(msg.unwrap()).await;
                }
                msg =  net_recv.recv()=> {

                    self.consume_net(msg.unwrap()).await;
                }
            }
        }
    }
}



