use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::borrow::Cow;
use crate::pakv::channel_caller::PaKVCtxChanCallerForUser;

const ADDRESS: &str = "127.0.0.1:7878";

pub struct PaKVServer{
    pakvchan:PaKVCtxChanCallerForUser
}
fn reply_succ_str(s:String) -> String {
    format!("s:{}", s)
}
fn reply_fail_str(s:String) -> String {
    format!("f:{}", s)
}

impl PaKVServer{
    pub fn new(chan:PaKVCtxChanCallerForUser) -> PaKVServer{
        PaKVServer{
            pakvchan:chan
        }
    }

    pub fn start(&mut self){

        info!("start server at {}", ADDRESS);
        let listener = TcpListener::bind(ADDRESS).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            self.handle_connection(stream);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream){
        info!("handle connection");
        let mut buffer = [0; 2048];

        let len=stream.read(&mut buffer).unwrap();
        let s=String::from_utf8_lossy(&buffer[..len]);
        let rep=self.cmdmatch(s);
        // let response = "HTTP/1.1 200 OK\r\n\r\n";

        stream.write(rep.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    
}
