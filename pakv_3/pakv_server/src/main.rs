
mod server;
mod pakv;


fn main() {
    let pakv_chan_handler=pakv::start_kernel();
    server::PaKVServer::new(pakv_chan_handler).start();
}