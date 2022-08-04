use std::io::{Read, Write};
use std::borrow::Cow;
use crate::pakv::channel_caller::App2KernelSender;
use tokio::net::TcpListener as TokioTcpListener;
use crate::server_rw_eachclient;
use crate::server_rw_eachclient::ClientId;
use crate::server_mainloop::{mainloop, PaKVServerApp};
use crate::client2server::Client2ServerSender;

const ADDRESS: &str = "127.0.0.1:7878";

pub struct PaKVServer{
    pakvchan: App2KernelSender
}

pub async fn start(){
    let mut nextcid=0 as ClientId;
    info!("start server at {}", ADDRESS);
    let listener = TokioTcpListener::bind(ADDRESS).await?;
    // let listener = TcpListener::bind(ADDRESS).unwrap();

    let (a,b)= Client2ServerSender::create_with_chan();
    //服务端接收循环
    tokio::spawn(async move{
        loop{
            let (stream,_addr)=listener.accept().await.unwrap();

            let s2csender=server_rw_eachclient::handle_stream(nextcid,&a, stream).await;
            a.client_in(nextcid,s2csender);
            nextcid+=1;
        }
    });
    let pakvserverapp= PaKVServerApp::new();
    mainloop(pakvserverapp,b).await;
}
