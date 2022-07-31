pub mod serial;
pub mod compact;
pub mod meta;
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, read_dir, File};
use std::io::{BufReader, BufRead, Write, Seek, SeekFrom};
use crate::pakv::{PaKVCtx};
// use serde_json::Error;
use std::{fs, slice};
use std::collections::{ BTreeMap};
use serial::{KvOpeE, KvOpe};
// use std::error::Error;
// use std::intrinsics::add_with_overflow;

// use std::str;
// struct KvOpeSet{
//     k:String,
//     v:String,
// }
// struct KvOpeDel{
//     k:String,
// }
// pub trait FileOpe{
//     fn compact_ifneed(&mut self,curpos:u64);
// }
// impl FileOpe for PaKVCtx{
//
// }


#[derive(Clone)]
pub struct FilePos {
    pub file_id: u64,
    pub pos: u64,
}

impl FilePos {
    pub fn readline(&self) -> Option<String> {
        let logfid = LogFileId {
            id: self.file_id
        };
        let p = logfid.get_pathbuf();
        let f = OpenOptions::new()
            .read(true)
            .open(p);
        match f {
            Ok(ff) => {
                let mut reader = BufReader::new(ff);
                reader.seek(SeekFrom::Start(self.pos)).unwrap();
                let mut s = String::new();
                reader.read_line(&mut s).unwrap();
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


#[derive(Clone)]
pub struct LogFileId {
    pub(crate) id: u64,
}

impl LogFileId {
    // pub fn get_path(&self) -> Path {
    //     let s=format!("./store/{}.kv",self.id);
    //     Path::new(&*s)
    // }
    pub fn from_path(p: PathBuf) -> Option<LogFileId> {
        let mut s = p.file_name().unwrap().to_str().unwrap();
        // println!("parsing {}",s);
        let i_ = s.find(".kv");
        match i_ {
            Some(i) => {
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
                if let Ok(i) = s.parse::<u64>() {
                    return Some(LogFileId { id: i });
                } else {
                    return None;
                }
            }
            None => { return None; }
        }
    }
    pub fn get_pathbuf(&self) -> PathBuf {
        let s = format!("./store/{}.kv", self.id);
        PathBuf::from(s)
    }

    pub fn set_by_logfile_path(&mut self, path: &PathBuf) -> bool {
        let mut s = path.file_name().unwrap().to_str().unwrap();
        // println!("parsing {}",s);
        let i_ = s.find(".kv");
        if let Some(i)=i_{
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
            self.id = s.parse().unwrap();

            return true;
        }
        return false;
    }

    pub fn touch_if_not_exist(&self) {
        if let Err(e)=OpenOptions::new().create(true).write(true).open(self.get_pathbuf()){
            eprintln!("{}",e);
        }
    }
    // pub fn begin_read(&self){
    //     let file=OpenOptions::new().read(true).open(self.get_pathbuf());
    //     let mut reader = BufReader::new(file);
    // }
}

pub fn logfile_gothroughlogs(file: &File, mut handle_one_ope: impl FnMut(u64, &String, &KvOpe)) {
    let mut reader = BufReader::new(file);
    let mut off: u64 = 0;
    loop {
        let mut line_str = String::new();
        let line = reader.read_line(&mut line_str);
        match line {
            Ok(l) => {
                if l == 0 {
                    break;
                }
                let ope = KvOpe::from_str(&line_str);
                // println!("read line {}",line_str);
                match ope {
                    Ok(ope) => {
                        handle_one_ope(off, &line_str, &ope);
                    }
                    Err(_) => {
                        println!("parse file failed");
                    }
                }
                off += l as u64;
            }
            Err(e) => {//读取行失败
                println!("{}", e);
                break;
            }
        }
    }
}

pub fn file_append_log(filepath: &PathBuf, mut str: String) -> Option<u64> {
    // match ctx.tarfpath.clone(){
    //     None => {
    //         panic!("tar path is not chosen");
    //     }
    //     Some(p) => {
    let filer = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filepath);
    match filer {
        Ok(mut file) => {
            // file.stream_position()
            let o = file.seek(SeekFrom::End(0)).unwrap();
            println!("cur pos {}", o);
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
        Err(e) => {
            panic!("open tar file failed {}",e);
        }
    }

}

const LOG_FILE_MAX:u64 =1 << 20;


fn scan_log_files() -> Vec<LogFileId> {
    let path = Path::new(pathstr_of_logfile());
    let mut v = vec![];
    let r = read_dir(path).unwrap();
    for dir_ in r {
        let dir = dir_.unwrap();
        let meta = dir.metadata().unwrap();
        if meta.is_file() {
            let fid_ = LogFileId::from_path(dir.path());
            if let Some(fid) = fid_ {
                v.push(fid);
            }
        }
    }
    return v;
}
// fn collect_log_files(except:Vec<LogFileId>){
//     let path=Path::new(path_logfile());
//     let r=read_dir(path).unwrap();
// }

fn pathstr_of_logfile() -> &'static str {
    return "./store/";
}

//   pub  static   META_FILE_OPE:MetaFileOpe=MetaFileOpe{store:None});

// pub fn touch_file_ifnot_exist()
pub fn file_check(ctx: &mut PaKVCtx) {
    let path = Path::new(pathstr_of_logfile());
    //1.缓存文件文件夹
    if let Err(e)= fs::create_dir(path){
        // eprintln!("{}",e);
        info!("make sure store folder exist {}",e);
    }
    ctx.meta_file_ope.makesure_exist();
    let tarfid =ctx.meta_file_ope.get_usertar_fid();

    //3.遍历文件夹下的文件,恢复所有操作到内存，选定一个最小文件为后续持久化文件，
    let r = read_dir(path).unwrap();
    // let mut minfilelen = u64::MAX;
    // let mut minfilep = None;
    let mut rank_by_edit_time=BTreeMap::new();
    for dir_ in r {
        match dir_ {
            Ok(dir) => {
                // println!("scanning log {}", dir.path().as_os_str().to_str().unwrap());
                let p = dir.path();
                let file = File::open(&p).unwrap();
                let mut fileid = LogFileId { id: 0 };
                if !fileid.set_by_logfile_path(&p){
                    // println!("  skip");
                    continue;
                }
                match file.metadata() {
                    Ok(meta) => {
                        let t= meta.modified().unwrap();
                        rank_by_edit_time.insert(t,(fileid,file));
                        // if meta.len() < minfilelen {
                        //     minfilelen = meta.len();
                        //     let a =
                        //         minfilep = Some(dir.path().clone());
                        // }
                    }
                    Err(_) => {}
                }
            }
            Err(e) => {
                error!("err when go through store folder {}", e);
            }
        }
    }
    fn file_readlogs(ctx:&mut PaKVCtx,fid:LogFileId,file:&mut File){
        info!("recovering from log file {}",fid.id);
        logfile_gothroughlogs(file, |off, _line, kvope| {
            // println!("recover ope {} {}",line_str,off);
            // println!("seek relative {}");
            //恢复操作
            match &kvope.ope {
                KvOpeE::KvOpeSet { k,v:_ } => {
                    ctx.store.set(k.clone(), &FilePos {
                        file_id: fid.id,
                        pos: off,
                    })
                }
                KvOpeE::KvOpeDel { k } => {
                    ctx.store.del(k);
                }
            }
        });
    }
    let mut tarfile =None;
    let mut latest_fid=0;
    info!("begin to recover hash index from logs，and make sure tarfile {} exist",tarfid);
    //最新编辑的最后操作
    for (_t,(id, mut file)) in rank_by_edit_time{
        // println!("cur file id {} tarfid {}",id.id,tarfid);
        if id.id==tarfid{//meta中标记的tarfile是否存在
            tarfile=Some(file);
            continue;
        }
        file_readlogs(ctx,id.clone(),&mut file);
        latest_fid=id.id;
    }
    match tarfile {//meta中标记的tarfile是否存在
        None => {
            //没有找到目标存储文件，则选定一个序号作为目标文件
            if latest_fid!=0{
                info!("didnt find tarfile,the latest edit file {} would be chosen for tarfile",latest_fid);
                ctx.tarfid.id=latest_fid;
                ctx.meta_file_ope.set_usertar_fid(latest_fid);
                // META_FILE_OPE.with(|mut file|{
                //     file.borrow_mut().set_usertar_fid(latest_fid);
                // })
            }else{
                info!("no log file exists,tarfile would be 1");
                ctx.tarfid.id=1;
                ctx.tarfid.touch_if_not_exist();
            }
        }
        Some(mut f) => {
            info!("the tarfile id {} in meta is valid, would be set as current tarfile",tarfid);
            //找到了匹配的
            ctx.tarfid.id=tarfid;
            file_readlogs(ctx,ctx.tarfid.clone(),&mut f);
        }
    }
    info!("index recovering finished!");
}