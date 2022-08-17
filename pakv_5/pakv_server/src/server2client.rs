use tokio::sync::mpsc::{Sender, Receiver};
use crate::net::msg_gen;

#[derive(Debug)]
pub struct Server2ClientMsg {
    pub serilized_vec:Vec<u8>
}
impl Server2ClientMsg{
    pub fn new(vec:Vec<u8>) -> Server2ClientMsg {
        Server2ClientMsg{
            serilized_vec:vec
        }
    }
}

/// 持有管道，向client的发送进程发送数据
#[derive(Clone,Debug)]
pub struct Server2ClientSender {
    _sender:Sender<Server2ClientMsg>
}
impl Server2ClientSender {
    pub fn new()
        -> (Server2ClientSender, Receiver<Server2ClientMsg>) {
        let (t, r): (Sender<Server2ClientMsg>, Receiver<Server2ClientMsg>)
            = tokio::sync::mpsc::channel(10);
        (Server2ClientSender {
            _sender:t
        },r)
    }
    pub async fn set_rpl(&self, succ:bool){
        self._sender.send(
            Server2ClientMsg::new(
                msg_gen::genmsg_setrpl(succ))).await.unwrap()
    }
    pub async fn get_rpl(&self, res:Option<String>){
        self._sender.send(Server2ClientMsg::new(
            msg_gen::genmsg_getrpl(res))).await.unwrap()
    }
    pub async fn del_rpl(&self, succ:bool) {
        self._sender.send(Server2ClientMsg::new(
            msg_gen::genmsg_delrpl(succ))).await.unwrap()
    }
}
