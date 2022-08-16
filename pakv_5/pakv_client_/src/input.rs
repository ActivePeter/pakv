// use std::io;
// use std::io::{Write, BufReader};
use crate::chan::{App2ConnSend, Conn2AppMsg};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::Receiver;
use std::io::Write;

pub async fn input_wait(send: App2ConnSend, mut rec: Receiver<Conn2AppMsg>) {
    println!("input_wait");
    // loop {
    fn newinput() {
        print!("> ");
        std::io::stdout().flush().expect("Couldn't flush stdout");
    }
    // Print command prompt and get command
    newinput();
    // println!("input_wait");
    // let mut input = String::new();

    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    loop {
        tokio::select! {
                line_= lines.next_line()=>{
                    // match line_{/
                    if let Ok(Some(line))=line_{

                        println!("line input {}",line);
                        send.send_cmd(line).await;
                    }else{
                        println!("input end");
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
                                    newinput();
                                }
                            }
                        }
                        None=>{
                            println!("net no");
                            break;
                        }
                    }
                }
            }
    }

    println!("loop end");
    // }
}
