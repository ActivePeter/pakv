use crate::server_rw_eachclient::{ClientId};
use tokio::sync::mpsc::{Sender, Receiver};
use crate::server2client::Server2ClientSender;

#[derive(Clone)]
pub enum Client2ServerMsg{
    ClientIn{cid:ClientId,s2csender:Server2ClientSender},
    ClientOut{cid:ClientId},
    Set{cid:ClientId,k:String,v:String},
    Get{cid:ClientId,k:String},
    Del{cid:ClientId,k:String}
}
#[derive(Clone)]
pub struct Client2ServerSender {
    sender:Sender<Client2ServerMsg>
}

impl Client2ServerSender {
    pub fn create_with_chan() -> (Client2ServerSender, Receiver<Client2ServerMsg>) {
        let (tx,rx)
            :(Sender<Client2ServerMsg>,Receiver<Client2ServerMsg>)
            =tokio::sync::mpsc::channel(10);
        (Client2ServerSender {
            sender:tx
        }, rx)
    }
    pub async fn client_in(&self,cid:ClientId,s2csender:Server2ClientSender){
        self.sender.send(Client2ServerMsg::ClientIn {
            cid,
            s2csender
        }).await.unwrap();
    }
    pub async fn client_out(&self,cid:ClientId){
        self.sender.send(Client2ServerMsg::ClientOut {
            cid: cid
        }).await.unwrap()
    }
    pub async fn set(&self,cid:ClientId,k:String,v:String){
        self.sender.send(Client2ServerMsg::Set {
            cid,
            k,
            v
        }).await.unwrap()
    }
    pub async fn get(&self,cid:ClientId,k:String){
        self.sender.send(Client2ServerMsg::Get {
            cid,
            k
        }).await.unwrap()
    }
    pub async fn del(&self,cid:ClientId,k:String){
        self.sender.send(Client2ServerMsg::Del {
            cid,
            k
        }).await.unwrap()
    }
}