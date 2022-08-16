use std::io::{Read, Write};
use std::borrow::Cow;
use tokio::net::TcpListener as TokioTcpListener;
use crate::net::msg2app_sender::{NetMsg2AppSender, NetMsg2App};
use tokio::sync::mpsc::Receiver;
use crate::net::server_rw_eachclient;
use crate::net::server_rw_eachclient::ClientId;
use tokio::io::AsyncReadExt;

const ADDRESS: &str = "127.0.0.1:7878";

// pub struct PaKVServer{
//     pakvchan: App2KernelSender
// }

pub async fn start_wait_client() -> Receiver<NetMsg2App> {
    let mut nextcid = 0 as ClientId;
    info!("start server at {}", ADDRESS);

    let (a, b) = NetMsg2AppSender::create_with_chan();
    //服务端接收循环
    tokio::spawn(async move {
        let listener = TokioTcpListener::bind(ADDRESS).await.unwrap();
        loop {
            let (mut stream, _addr) = listener.accept().await.unwrap();
            // tokio::spawn(async move {
            //stream.read_f32().await;
            let s2csender = server_rw_eachclient::handle_stream(
                nextcid, &a, stream).await;
            a.client_in(nextcid, s2csender).await;
            println!("loop end");
            nextcid += 1;
        }
    });

    b
}
