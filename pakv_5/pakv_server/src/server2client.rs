use tokio::sync::mpsc::Sender;
use pakv_server_lib::pakv::KernelToAppMsg;
use crate::msg2client;

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
pub struct Server2ClientSender {
    _sender:Sender<Server2ClientMsg>
}
impl Server2ClientSender {
    pub fn new(sender:Sender<Server2ClientMsg>) -> Server2ClientSender {
        Server2ClientSender {
            _sender:sender
        }
    }
    pub async fn set_rpl(&self, succ:bool){
        self._sender.send(Server2ClientMsg::new(msg2client::genmsg_setrpl(succ))).await.unwrap()
    }
    pub async fn get_rpl(&self, res:Option<String>){
        self._sender.send(Server2ClientMsg::new(
            msg2client::genmsg_getrpl(res))).await.unwrap()
    }
    pub async fn del_rpl(&self, succ:bool) {
        self._sender.send(Server2ClientMsg::new(
            msg2client::genmsg_delrpl(succ))).await.unwrap()
    }
}
