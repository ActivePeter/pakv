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
mod conccurent_bench;
// pub mod msg_gen;
// pub mod msg_match;

#[tokio::main]
async fn main() {
    env_logger::init();//remember to set RUST_LOG=INFO
    println!("starting up");

    conccurent_bench::conccurent_bench();

    // //监听客户端
    // let recv=
    //     server_listen::start_wait_client().await;
    //
    // server_app::PaKVServerApp::new().await
    //     .hold(recv).await;
}