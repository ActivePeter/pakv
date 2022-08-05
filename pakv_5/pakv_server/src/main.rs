#[macro_use]
extern crate log;

use log::LevelFilter;

pub mod server_listen;
pub mod pakv;
pub mod server_rw_eachclient;
pub mod msg_parse;
pub mod server_mainloop;
pub mod client2server;
pub mod server2client;
pub mod msg_gen;
pub mod msg_match;

#[tokio::main]
async fn main() {
    env_logger::init();//remember to set RUST_LOG=INFO
    info!("starting up");
    // let tp=paco::threadpool::ThreadPool::new(1);

    // let pakv_chan_handler=pakv::start_kernel().await;

    server_listen::start().await;
}