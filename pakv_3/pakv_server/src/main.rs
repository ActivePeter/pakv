#[macro_use]
extern crate log;

use log::LevelFilter;

mod server;
mod pakv;


fn main() {
    env_logger::init();
    info!("starting up");
    let pakv_chan_handler=pakv::start_kernel();
    server::PaKVServer::new(pakv_chan_handler).start();
}