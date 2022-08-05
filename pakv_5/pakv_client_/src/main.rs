pub mod input;
pub mod conn;
pub mod chan;
pub mod msg_parse;

#[tokio::main]
async fn main() {
    if let Some((a2ssend, b))=conn::conn2server().await{
        input::input_wait(a2ssend,b).await;
    }else{
        println!("failed to conn");
    }
}