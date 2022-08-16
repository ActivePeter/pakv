use tokio::sync::mpsc as TMpsc;
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum Conn2AppMsg{
    End,
    Common{
        print2user:String
    }
}
pub struct Conn2AppSend{
    send:TMpsc::Sender<Conn2AppMsg>
}
impl Conn2AppSend{
    pub fn new() -> (Conn2AppSend, Receiver<Conn2AppMsg>) {
        let (t, r)
            :(TMpsc::Sender<Conn2AppMsg>,TMpsc::Receiver<Conn2AppMsg>)
            =tokio::sync::mpsc::channel(10);

        (Conn2AppSend{
            send: t
        },r)

    }
    pub async fn send_print2usr(&self,str:String){
        self.send.send(Conn2AppMsg::Common {
            print2user:str
        }).await.unwrap();
    }
    pub async fn end_app(&self){
        self.send.send(Conn2AppMsg::End).await.unwrap()
    }
}
#[derive(Debug)]
pub struct App2ConnMsg {
    pub vec:String
}
impl App2ConnMsg {
    pub fn new(vec:String) -> App2ConnMsg {
        App2ConnMsg {
            vec
        }
    }
}

pub struct App2ConnSend{
    send:TMpsc::Sender<App2ConnMsg>
}
impl App2ConnSend{
    pub fn new() -> (App2ConnSend, Receiver<App2ConnMsg>) {
        let (t,r)
            :(TMpsc::Sender<App2ConnMsg>, TMpsc::Receiver<App2ConnMsg>)
            =tokio::sync::mpsc::channel(10);
        (App2ConnSend{
            send: t
        },r)
    }
    pub async fn send_cmd(&self,cmd:String){
        self.send.send(App2ConnMsg::new(cmd)).await.unwrap();
    }
}