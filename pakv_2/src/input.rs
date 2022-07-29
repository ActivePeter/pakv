use std::io;
use std::io::Write;
use std::sync;
use crate::pakv::UserKvOpe;
use clap::{App, ArgMatches, Arg, Command,arg};
use std::ops::Index;
use std::sync::mpsc::RecvError;

pub fn input_wait(sender:sync::mpsc::Sender<UserKvOpe>){
    let app = App::new("MyApp")
        // .subcommand_required(true)
        // .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("set")
                .arg(arg!([K]))
                .arg(arg!([V]))
        )
        .subcommand(
            Command::new("get")
                .arg(arg!([K]))
        )
        .subcommand(
            Command::new("del")
                .arg(arg!([K]))
        )
        ;

    loop {
        // Print command prompt and get command
        print!("> ");
        io::stdout().flush().expect("Couldn't flush stdout");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error reading input.");
        let mut div:Vec<&str>=(*input).split_whitespace().collect::<Vec<&str>>();
        div.insert(0,"s");
        // let match_ =;
        let matches= app.clone().get_matches_from(div);
        match matches.subcommand() {
            Some(("set", _matches)) => {
                let k_=_matches.get_one::<String>("K");
                let v_=_matches.get_one::<String>("V");
                if let (Some(k), Some(v)) = (k_,v_) {
                    sender.send(UserKvOpe::KvOpeSet {
                        k:k.clone(),
                        v:v.clone()
                    });
                    // Ok(Card::new(face, suit))
                } else {
                    println!("wrong set arg");
                    // Err(())
                }
                // write!(std::io::stdout(), "Pong").map_err(|e| e.to_string())?;
                // std::io::stdout().flush().map_err(|e| e.to_string())?;
            }
            Some(("get", _matches)) => {
                let k_=_matches.get_one::<String>("K");

                if let (Some(k)) = (k_) {
                    let (tx,
                        rx)
                        =UserKvOpe::create_get_chan();
                    sender.send(UserKvOpe::KvOpeGet {
                        k:k.clone(),
                        resp:tx
                    });
                    let r=rx.recv();
                    if let Ok(Some(get))=r{
                        println!("found {}",get)
                    }else{
                        println!("not found");
                    }
                    // Ok(Card::new(face, suit))
                } else {
                    println!("wrong set arg");
                    // Err(())
                }
                // write!(std::io::stdout(), "Pong").map_err(|e| e.to_string())?;
                // std::io::stdout().flush().map_err(|e| e.to_string())?;
            }
            Some(("del", _matches)) => {
                let k_=_matches.get_one::<String>("K");

                if let (Some(k)) = k_ {
                    let (tx,rx)
                        =UserKvOpe::create_del_chan();
                    sender.send(UserKvOpe::KvOpeDel {
                        k:k.clone(),
                        // v:v.clone()
                        resp: tx
                    });
                    if let Ok(true)=rx.recv(){
                        println!("remove succ");
                    }else{
                        println!("not found k {}",k);
                    }
                    // Ok(Card::new(face, suit))
                } else {
                    println!("wrong set arg");
                    // Err(())
                }
                // write!(std::io::stdout(), "Pong").map_err(|e| e.to_string())?;
                // std::io::stdout().flush().map_err(|e| e.to_string())?;
            }
            _ => {println!("no cmd matches");}

        }
        // if let Some(deleted) = matches.value_of("set") {
        //     // info!("Removing db item: {}", &deleted);
        //     // remove_db_item(&mongo_client, collection, deleted)?;
        // }
        // let args = WORD.captures_iter(&input)
        //     .map(|cap| cap.get(1).or(cap.get(2)).unwrap().as_str())
        //     .collect::<Vec<&str>>();
    }
}
