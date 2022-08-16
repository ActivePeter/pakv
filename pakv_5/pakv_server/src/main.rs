#[macro_use]
extern crate log;

use log::LevelFilter;
use crate::net::server_listen;

// pub mod server_listen;
pub mod pakv;
// pub mod server_rw_eachclient;
// pub mod msg_parse;
// pub mod client2server;
pub mod server2client;
pub mod net;
pub mod server_app;
pub mod server_app_consume;
// pub mod msg_gen;
// pub mod msg_match;

#[tokio::main]
async fn main() {
    env_logger::init();//remember to set RUST_LOG=INFO
    println!("starting up");

    // tokio::spawn(async move{
        //处理内核运行以及，客户端消息
    // });


    // tokio::spawn(async move{
        //监听客户端
        let recv=
            server_listen::start_wait_client().await;


    server_app::PaKVServerApp::new().await
        .hold(recv).await;
    // });

}