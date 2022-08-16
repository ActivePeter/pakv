pub mod input;
pub mod conn;
pub mod chan;
pub mod msg_parse;
pub mod msg_gen;

#[tokio::main]
async fn main() {
    if let Some((a2ssend, b))=conn::conn2server().await{
        input::input_wait(a2ssend,b).await;
    }else{
        println!("failed to conn");
    }
    println!("main end");
}