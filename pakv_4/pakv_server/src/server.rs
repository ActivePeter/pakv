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

    fn cmdhandle_get(&self, k:&str)->String{
        match
        self.pakvchan.get(k.to_string()){
            None => {
                reply_fail_str("didn't find".to_string())
            }
            Some(s) => {
                format!("s:find {}", s)
            }
        }

    }
    fn cmdhandle_set(&self,k:&str,v:&str)->String{
        self.pakvchan.set(k.to_string(),v.to_string());
        reply_succ_str("setted".to_string())
    }
    fn cmdhandle_del(&self,k:&str)->String{
        match self.pakvchan.del(k.to_string()){
            true => {
                reply_succ_str("found and delete".to_string())
            }
            false => {
                reply_fail_str("didnt find".to_string())
            }
        }
    }

    fn cmdmatch(&self, s:Cow<str>) -> String {
        let div:Vec<&str>=s.as_ref().split_whitespace().collect::<Vec<&str>>();
        // println!("recv:{}",s);
        if div.len()==2 && div[0]=="get"{
            return self.cmdhandle_get(div[1]);

        }
        if div.len()==3 && div[0]=="set" {
            return self.cmdhandle_set(div[1],div[2]);

        }
        if div.len()==2 && div[0]=="del" {
            return self.cmdhandle_del(div[1]);
        }
        return "Invalid Cmd".to_string();
    }
}
