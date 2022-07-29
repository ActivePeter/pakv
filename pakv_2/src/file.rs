use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, read_dir, File, DirEntry, Metadata};
use std::io::{BufReader, BufRead};
use crate::pakv::PaKVCtx;
use serde::{Serialize, Deserialize};
use serde_json::Error;
use std::char::MAX;
use std::fs;

// struct KvOpeSet{
//     k:String,
//     v:String,
// }
// struct KvOpeDel{
//     k:String,
// }
#[derive(Serialize, Deserialize, Debug)]
pub enum KvOpeE{
    KvOpeSet{k:String,v:String},
    KvOpeDel{k:String},
}
#[derive(Serialize, Deserialize, Debug)]
pub struct KvOpe {
    // pub opetype:u8,
    // pub time: u32,
    pub ope:KvOpeE,
}
impl KvOpe{
    pub fn from_str(str_:&str) -> serde_json::Result<KvOpe> {
        serde_json::from_str(str_)
    }
    pub fn to_line_str(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}
pub fn file_check(ctx:&mut PaKVCtx){
    let path=Path::new("./store");
    //1.缓存文件文件夹
    fs::create_dir(path);
    // OpenOptions::new().create(true).write(true).open(path).unwrap();

    // let meta=dir.metadata().unwrap();
    // if !meta.is_dir() {
    //     // panic!("store应该为文件夹");
    // }
    //3.遍历文件夹下的文件,恢复所有操作到内存，选定一个最小文件为后续持久化文件，
    let r=read_dir(path).unwrap();
    let mut minfilelen =u64::MAX;
    let mut minfilep =None;
    r.map(|v|{
        match v {
            Ok(dir) => {
                let mut file = File::open(dir.path()).unwrap();
                match file.metadata(){
                    Ok(meta) => {
                        if meta.len()<minfilelen {
                            minfilelen=meta.len();
                            let a=
                            minfilep=Some(dir.path().clone());
                        }
                        let reader = BufReader::new(file);

                        for line in reader.lines() {
                            let mut fail=false;
                            match line{
                                Ok(l) => {

                                    let ope=KvOpe::from_str(&l);
                                    match ope {
                                        Ok(ope) => {
                                            //恢复操作
                                            ctx.store.do_ope(&ope);
                                        }
                                        Err(_) => {

                                            fail=true;
                                        }
                                    }
                                    if fail{
                                        println!("parse file failed");
                                    }
                                }
                                Err(e) => {
                                    println!("{}",e);
                                    fail=true;
                                }
                            }

                            // println!("{}", line?);
                        }
                    }
                    Err(_) => {}
                }

            }
            Err(e) => {
                println!("{}",e);
            }
        }

        // fs::read_to_string()
    });
    //此时数据已经恢复，并选出了最小的文件
    match minfilep{
        None => {
            //没有选出最小文件
            println!("no min log file");
            ctx.tarfpath=Some(PathBuf::from("./store/1.kv"));
            OpenOptions::new().create(true).write(true).open("./store/1.kv");
        }
        Some(p) => {
            ctx.tarfpath=Some(p);
        }
    }
    //3.启动定时收缩进程

}