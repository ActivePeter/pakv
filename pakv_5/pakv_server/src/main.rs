#[macro_use]
extern crate log;

use log::LevelFilter;

mod server;
mod pakv;


fn main() {
    env_logger::init();//remember to set RUST_LOG=INFO
    info!("starting up");
    let tp=paco::threadpool::ThreadPool::new(1);

    let pakv_chan_handler=pakv::start_kernel();

    server::PaKVServer::new(pakv_chan_handler).start(tp.get_handle());
}