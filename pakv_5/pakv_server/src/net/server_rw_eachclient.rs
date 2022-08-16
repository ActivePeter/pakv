use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use std::io::Error;
use crate::server2client::{Server2ClientSender, Server2ClientMsg};
use crate::*;
use crate::net::{msg_parse, msg_match};
use crate::net::msg_parse::MsgParser;
use crate::net::msg2app_sender::NetMsg2AppSender;

pub type ClientId = u64;

fn write_loop(cid: ClientId, mut writer: OwnedWriteHalf) -> Server2ClientSender {
    let (t, mut r): (Sender<Server2ClientMsg>, Receiver<Server2ClientMsg>) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        loop {
            let to_client = r.recv().await.unwrap();
            // Some(to_client) => {
            match writer.write_all(&to_client.serilized_vec).await {
                Ok(_) => {}
                Err(e) => {
                    error!("write failed {}",e);
                    break;
                }
            }
            // }
            // }
        }
        info!("writeloop end");
    });

    Server2ClientSender::new(t)
}

async fn read_loop(cid: ClientId, c2s: NetMsg2AppSender, mut r: OwnedReadHalf) {
    tokio::spawn(async move {
        let mut buf = [0; 1024];
        let mut packmaker = MsgParser::create();
        loop {
            let n = match r.read(&mut buf).await {
                // socket closed
                Ok(n) if n == 0 => break,
                Ok(n) => n,
                Err(e) => {
                    eprintln!("failed to read from socket; err = {:?}", e);
                    break;
                }
            };
            println!("server recv some");
            //处理粘包半包
            packmaker.before_handle();
            loop {
                match packmaker.handle_a_buffset(&buf, n).await {
                    None => { break; }
                    Some(slice) => {
                        msg_match::match_msg_from_client(
                            cid, &c2s, slice).await;
                    }
                }
            }
        }
        c2s.client_out(cid);
        info!("readloop end");
    });
    // });
}

pub async fn handle_stream(cid: ClientId, c2s: &NetMsg2AppSender, mut stream: TcpStream)
                           -> Server2ClientSender {
    let (mut rx, tx) = stream.into_split();
    let c2s = c2s.clone();

    read_loop(cid, c2s.clone(), rx).await;

    let t = write_loop(cid, tx);
    t
}
