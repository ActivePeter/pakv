use std::net::TcpStream;
use std::io::{Write, Read};

const ADDRESS: &str = "127.0.0.1:7878";
pub struct PakVClient{

}
impl PakVClient{
    pub fn new() -> PakVClient {
        PakVClient{}
    }
    pub fn oneshot(&self, mut content:String){
        let mut client = TcpStream::connect(ADDRESS).expect("连接失败！");
        client.write(content.as_bytes()).expect("发送失败");
        content.clear();
        client.read_to_string(&mut content).expect("读取失败");
        // println!("  recv {}",content);
        if let Some(_v)= content.find("s:"){
            println!("  succ: {}",&content[2..]);
        }
        else if let Some(_v)= content.find("f:"){
            println!("  fail: {}",&content[2..]);
        }
        else{
            println!("  invalid request!");
        }
    }
}