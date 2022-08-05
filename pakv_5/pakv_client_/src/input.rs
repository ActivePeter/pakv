// use std::io;
// use std::io::{Write, BufReader};
use crate::chan::{App2ConnSend, Conn2AppMsg};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::Receiver;

pub async fn input_wait(send:App2ConnSend,mut rec:Receiver<Conn2AppMsg>){

    // loop {
    async fn newinput(){
        print!("> ");
        tokio::io::stdout().flush().await.expect("Couldn't flush stdout");
    }
        // Print command prompt and get command
    newinput().await;
        // let mut input = String::new();

        let mut lines = BufReader::new(tokio::io::stdin()).lines();
        loop{
            tokio::select! {
                line_= lines.next_line()=>{
                    // match line_{/
                        if let Ok(Some(line))=line_{
                            send.send_cmd(line).await;
                        }else{
                            break;
                        }
                    // }
                }
                msg_=rec.recv()=>{
                    match msg_{
                        Some(msg)=>{
                            match msg{
                                Conn2AppMsg::End=>{
                                    break;
                                }
                                Conn2AppMsg::Common{
                                    print2user
                                }=>{
                                    println!("  {}",print2user);
                                    newinput().await;
                                }
                            }
                        }
                        None=>{
                            break;
                        }
                    }
                }
            }
            // if let Some(line) = lines.next_line().await {
            //     println!("length = {}", line.len())
            // }
        }

    // }
}
