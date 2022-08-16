
use tokio::sync::mpsc::{Sender, Receiver};
use crate::server2client::Server2ClientSender;
use crate::net::server_rw_eachclient::ClientId;

#[derive(Clone,Debug)]
pub enum NetMsg2App {
    ClientIn{cid:ClientId,s2csender:Server2ClientSender},
    ClientOut{cid:ClientId},
    Set{cid:ClientId,k:String,v:String},
    Get{cid:ClientId,k:String},
    Del{cid:ClientId,k:String}
}
#[derive(Clone)]
pub struct NetMsg2AppSender {
    sender:Sender<NetMsg2App>
}

impl NetMsg2AppSender {
    pub fn create_with_chan() -> (NetMsg2AppSender, Receiver<NetMsg2App>) {
        let (tx,rx)
            :(Sender<NetMsg2App>, Receiver<NetMsg2App>)
            =tokio::sync::mpsc::channel(10);
        (NetMsg2AppSender {
            sender:tx
        }, rx)
    }
    pub async fn client_in(&self,cid:ClientId,s2csender:Server2ClientSender){
        self.sender.send(NetMsg2App::ClientIn {
            cid,
            s2csender
        }).await.unwrap();
    }
    pub async fn client_out(&self,cid:ClientId){
        self.sender.send(NetMsg2App::ClientOut {
            cid: cid
        }).await.unwrap()
    }
    pub async fn set(&self,cid:ClientId,k:String,v:String){
        self.sender.send(NetMsg2App::Set {
            cid,
            k,
            v
        }).await.unwrap()
    }
    pub async fn get(&self,cid:ClientId,k:String){
        self.sender.send(NetMsg2App::Get {
            cid,
            k
        }).await.unwrap()
    }
    pub async fn del(&self,cid:ClientId,k:String){
        self.sender.send(NetMsg2App::Del {
            cid,
            k
        }).await.unwrap()
    }
}