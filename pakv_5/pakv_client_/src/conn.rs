use crate::chan::{App2ConnSend, Conn2AppSend, Conn2AppMsg, App2ConnMsg};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::Receiver;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use crate::msg_parse::MsgParser;
use crate::msg_gen;

const ADDRESS: &str = "127.0.0.1:7878";
// pub struct Connection{
// }
// impl Connection{
//     pub fn new() -> Connection {
//         Connection{}
//     }
    async fn readloop(mut rh:OwnedReadHalf, send2app:Conn2AppSend){
        tokio::spawn(async move{
            let mut buf = [0; 1024];
            let mut packmaker= MsgParser::create();
            loop {
                let n = match rh.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => {
                        println!("socket  end ");
                        break; },
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        break;
                    }
                };
                //处理粘包半包
                packmaker.before_handle();
                loop{
                    match packmaker.handle_a_buffset(&buf,n).await{
                        None => {break;}
                        Some(slice) => {
                            send2app.send_print2usr(String::from_utf8(slice.to_vec()).unwrap()).await;
                        }
                    }
                }
            }
            println!("readloop end")
        });
    }
    async fn writeloop(mut wh:OwnedWriteHalf, mut r:Receiver<App2ConnMsg>){
        tokio::spawn(async move{
            loop{
                let rec=r.recv().await;
                match rec{
                    None => {
                        break;
                    }
                    Some(msg) => {
                        println!("writeloop send");
                        //发送头
                        let head=msg_gen::headlen_bytes(&msg.vec);
                        let r=wh.write_all(&head).await;
                        match r{
                            Ok(_) => {}
                            Err(e) => {
                                println!("send err {}",e);
                                break;
                            }
                        }
                        //发送体
                        let r=wh.write_all(msg.vec.as_bytes()).await;
                        match r{
                            Ok(_) => {}
                            Err(e) => {
                                println!("send err {}",e);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
    pub async fn conn2server()->Option<(App2ConnSend,Receiver<Conn2AppMsg>)>{
        let res=tokio::net::TcpStream::connect(ADDRESS).await;
        if let Ok( st)=res{
            println!("connected");
            let (s,r)=App2ConnSend::new();
            let (c2a_s, c2a_r)=Conn2AppSend::new();

            let ( rh,wh)=st.into_split();
            readloop(rh,c2a_s).await;

            writeloop(wh,r).await;

            Some((s,c2a_r))
        }else{
            res.unwrap();
            None
        }
    }
// }