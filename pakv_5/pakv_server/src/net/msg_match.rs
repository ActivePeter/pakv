// use crate::client2server::Client2ServerSender;
// use crate::server_rw_eachclient::ClientId;

use crate::net::server_rw_eachclient::ClientId;
use crate::net::msg2app_sender::NetMsg2AppSender;

pub async fn match_msg_from_client(
    fromcid:ClientId,
    c2ssender: &NetMsg2AppSender,
    slice: &[u8],
) {
    let to_str=std::str::from_utf8(slice).unwrap();
    println!("recv client msg: {}",to_str);
    let div: Vec<&str> = to_str.split_whitespace().collect::<Vec<&str>>();
    // println!("recv:{}",s);
    if div.len() == 2 && div[0] == "get" {
        c2ssender.get(fromcid,div[1].to_string()).await;
    }
    if div.len() == 3 && div[0] == "set" {
        c2ssender.set(fromcid,div[1].to_string(),div[2].to_string()).await;
        // self.cmdhandle_set(div[1], div[2]);
    }
    if div.len() == 2 && div[0] == "del" {
        c2ssender.del(fromcid,div[1].to_string()).await;
        //self.cmdhandle_del(div[1]);
    }
}