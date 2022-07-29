use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, read_dir, File, DirEntry, Metadata};
use std::io::{BufReader, BufRead, Write, Seek, SeekFrom};
use crate::pakv::{PaKVCtx};
use serde::{Serialize, Deserialize};
use serde_json::Error;
use std::char::MAX;
use std::{fs, slice};
use std::cmp::max;

// use std::str;
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

#[derive(Clone)]
pub struct FilePos{
    pub file_id:u64,
    pub pos:u64,
}
impl FilePos{
    pub fn readline(&self)->Option<String>{
        let logfid=LogFileId{
            id: self.file_id
        };
        let p=logfid.get_pathbuf();
        let f=OpenOptions::new()
            .read(true)
            .open(p);
        match f{
            Ok(ff) => {
                let mut reader =BufReader::new(ff);
                reader.seek(SeekFrom::Start(self.pos));
                let mut s=String::new();
                reader.read_line(&mut s);
                return Some(s);
            }
            Err(e) => {
                println!("readline open fail {} {}",
                         logfid.get_pathbuf().to_str().unwrap(),
                    e
                );
                return None;
            }
        }
    }
}



pub struct LogFileId{
    pub(crate) id:u64
}
impl LogFileId{
    // pub fn get_path(&self) -> Path {
    //     let s=format!("./store/{}.kv",self.id);
    //     Path::new(&*s)
    // }
    pub fn get_pathbuf(&self) -> PathBuf {
        let s=format!("./store/{}.kv",self.id);
        PathBuf::from(s)
    }

    pub fn set_by_logfile_path(&mut self,path:&PathBuf){
        let mut s =path.file_name().unwrap().to_str().unwrap();
        // println!("parsing {}",s);
        let mut i=s.find(".kv").unwrap();

        let ptr = s.as_ptr();
        // i = max(i,0);
        // We can re-build a str out of ptr and len. This is all unsafe because
        // we are responsible for making sure the two components are valid:
        s = unsafe {
            // First, we build a &[u8]...
            let slice = slice::from_raw_parts(ptr, i);

            // ... and then convert that slice into a string slice
            std::str::from_utf8(slice).unwrap()
        };
        self.id=s.parse().unwrap();
    }
}

pub fn file_append_log(filepath: &PathBuf, mut str:String)->Option<u64>{
    // match ctx.tarfpath.clone(){
    //     None => {
    //         panic!("tar path is not chosen");
    //     }
    //     Some(p) => {
            let mut filer = OpenOptions::new()
                .write(true)
                .append(true)
                .open(filepath);
            match filer{
                Ok(mut file) => {
                    // file.stream_position()
                    let o=file.seek( SeekFrom::End(0)).unwrap();
                    println!("cur pos {}",o);
                    str.push('\n');
                    // file.
                    file.write(str.as_bytes()).unwrap();
                    return Some(o);
                    // // let v=["\n"];
                    // file.write("\n".as_bytes()).unwrap();

                    // let s=&*str;
                    // if let Err(e) = writeln!(file,s) {
                    //     eprintln!("Couldn't write to file: {}", e);
                    // }
                }
                Err(_) => {

                    panic!("open tar file failed")
                }
            }

    // }
    None

}
pub fn file_check(ctx:&mut PaKVCtx){
    let path=Path::new("./store/");
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
    for dir_ in r{
        match dir_ {
            Ok(dir) => {
                println!("scanning log {}",dir.path().as_os_str().to_str().unwrap());
                let p=dir.path();
                let mut file = File::open(&p).unwrap();
                let mut fileid =LogFileId{ id: 0 };
                fileid.set_by_logfile_path(&p);
                match file.metadata(){
                    Ok(meta) => {
                        if meta.len()<minfilelen {
                            minfilelen=meta.len();
                            let a=
                                minfilep=Some(dir.path().clone());
                        }
                        let mut reader = BufReader::new(file);
                        let mut off:u64=0;
                        // let lines=reader.lines();
                        loop{
                            let mut line_str=String::new();

                            let line =reader.read_line(&mut line_str);
                            // reader.lines();
                            // reader.pop();
                            // println!("reader pos {}",reader.seek(SeekFrom::Current(0)).unwrap());

                            // let mut fail=false;
                            match line{
                                Ok(l) => {
                                    if l==0 {
                                        break;
                                    }

                                    let ope=KvOpe::from_str(&line_str);
                                    // println!("read line {}",line_str);
                                    match ope {
                                        Ok(ope) => {
                                            println!("recover ope {} {}",line_str,off);
                                            // println!("seek relative {}");
                                            //恢复操作
                                            match ope.ope{
                                                KvOpeE::KvOpeSet {k,v  } => {
                                                    ctx.store.set(k, &FilePos {
                                                        file_id: fileid.id,
                                                        pos: off
                                                    })
                                                }
                                                KvOpeE::KvOpeDel { k } => {
                                                    ctx.store.del(k);
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            println!("parse file failed");
                                            // fail=true;
                                        }
                                    }
                                    off+=l as u64;
                                    // if fail{
                                    //     println!("parse file failed");
                                    // }
                                }
                                Err(e) => {
                                    println!("{}",e);
                                    // fail=true;
                                    break;
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

    }
    //此时数据已经恢复，并选出了最小的文件
    match minfilep{
        None => {
            //没有选出最小文件
            println!("no min log file");
            // ctx.tarfpath=Some(PathBuf::from("./store/1.kv"));
            OpenOptions::new().create(true).write(true).open(ctx.tarfid.get_pathbuf());
        }
        Some(p) => {
            ctx.tarfid.set_by_logfile_path(&p);
        }
    }
    //3.启动定时收缩进程

}