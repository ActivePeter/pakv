use pakv_server_lib::pakv::{PaKVCtx, KernelToAppMsg, PaKVOpeId};
use std::collections::HashMap;
use crate::server_rw_eachclient::{ClientId};
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use crate::client2server::Client2ServerMsg;
use crate::server2client::Server2ClientSender;


pub async fn mainloop(
    mut serverapp: PaKVServerApp,
    mut client2server_recv:Receiver<Client2ServerMsg>){

    loop{
        // let rr=server_ctx.kernel2apprecv.recv().await;
        // if let Some(msg)=rr{
        //
        // }
        // let r=client2server_recv.recv().await;
        tokio::select! {
            msg = serverapp.kernel2apprecv => {
                match msg{
                    KernelToAppMsg::SetResult { opeid,succ } => {
                        if let Some(handle)=serverapp.get_server2clientsender_of_opeid(opeid){
                            handle.set_rpl(succ).await;
                        }
                        // server_ctx.kernelopeid_2_cid.get(op)
                    }
                    KernelToAppMsg::DelResult { opeid,succ } => {
                        if let Some(handle)=serverapp.get_server2clientsender_of_opeid(opeid){
                            handle.del_rpl(succ).await;
                        }
                    }
                    KernelToAppMsg::GetResult { opeid,v } => {
                        if let Some(handle)=serverapp.get_server2clientsender_of_opeid(opeid){
                            handle.get_rpl(v).await;
                        }
                    }
                }
            }
            val =  client2server_recv=> {
                match val {
                    Client2ServerMsg::ClientIn { cid,chandle } => {
                        serverapp.client_in(cid,chandle);
                    }
                    Client2ServerMsg::ClientOut { cid} => {
                        serverapp.client_out(cid);
                    }
                    Client2ServerMsg::Set { cid,k,v} => {
                        //此处有两种考虑，
                        // 一种是await调用结果，这样后面的消息就得在这个之后处理
                        // 另一种考虑到io操作有时间，也可以先调用，但是不等待结果，等有了执行结果再返回给客户端
                        //  要使用这个模式需要用select处理两个通道，
                        //   一个是当前接收来自客户端的消息，
                        //   另一个是接收来自内核的消息

                        //采取第二种方法需要标记需要异步处理的任务，
                        // 等处理结束后，返回结果携带这个标记，这样就可以在上下文中找到对应的客户端id，然后,将消息返回给客户端
                        let opeid= serverapp.kernel.set(k,v);
                        serverapp.kernelopeid_2_cid.insert(opeid,cid);
                    }
                    Client2ServerMsg::Get { cid,k } => {
                        let opeid= serverapp.kernel.get(&k);
                        serverapp.kernelopeid_2_cid.insert(opeid,cid);
                    }
                    Client2ServerMsg::Del { cid,k } => {
                        let opeid= serverapp.kernel.del(&k);
                        serverapp.kernelopeid_2_cid.insert(opeid,cid);
                    }
                }
            }
        }
    }
}
//持有pakv内核以及服务端状态，
pub struct PaKVServerApp {
    kernel:PaKVCtx,//内核运行上下文
    clients_cid2sender:HashMap<ClientId,Server2ClientSender>,//用于服务端向客户端发消息
    kernelopeid_2_cid:HashMap<PaKVOpeId,ClientId>,//用于处理内核的执行结果

    kernel2apprecv:Receiver<KernelToAppMsg>
}
impl PaKVServerApp {
    pub fn new() -> PaKVServerApp {
        let (kernel,r)=PaKVCtx::create();
        PaKVServerApp {
            kernel,
            clients_cid2sender: Default::default(),
            kernelopeid_2_cid: Default::default(),
            kernel2apprecv:r
        }
    }
    pub fn get_server2clientsender_of_opeid(&self,opeid:PaKVOpeId)->Option<&Server2ClientSender>{
        if let Some(cid)=self.kernelopeid_2_cid.get(&opeid){
            if let Some(csender)=self.clients_cid2sender.get(cid){
                return Some(csender);
            }
            return None;
        }
        return None;
    }
    pub fn client_in(&mut self,cid:ClientId,chandle:Server2ClientSender){
        self.clients_cid2sender.insert(cid,chandle);
    }
    pub fn client_out(&mut self,cid:ClientId){
        self.clients_cid2sender.remove(&cid);
    }
}
