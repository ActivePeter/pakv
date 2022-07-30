use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, read_dir, File, DirEntry, Metadata};
use std::io::{BufReader, BufRead, Write, Seek, SeekFrom};
use crate::pakv::{PaKVCtx, PaKVCtxChannelCaller};
use serde::{Serialize, Deserialize};
use serde_json::Error;
use std::char::MAX;
use std::{fs, slice};
use std::cmp::max;
use std::collections::{HashMap, BTreeMap};
use std::ops::Deref;
use std::borrow::BorrowMut;
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


#[derive(Serialize, Deserialize, Debug)]
pub enum KvOpeE {
    KvOpeSet { k: String, v: String },
    KvOpeDel { k: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KvOpe {
    // pub opetype:u8,
    // pub time: u32,
    pub ope: KvOpeE,
}

impl KvOpe {
    pub fn from_str(str_: &str) -> serde_json::Result<KvOpe> {
        serde_json::from_str(str_)
    }
    pub fn to_line_str(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

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
                reader.seek(SeekFrom::Start(self.pos));
                let mut s = String::new();
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
        let mut i_ = s.find(".kv");
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
        let mut i_ = s.find(".kv");
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
        OpenOptions::new().create(true).write(true).open(self.get_pathbuf());
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
    let mut filer = OpenOptions::new()
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
        Err(_) => {
            panic!("open tar file failed")
        }
    }

    // }
    None
}

const LOG_FILE_MAX:u64 =1 << 20;

//这里存入id前必须保证有效性
pub struct CompactCtx {
    compactfrom_fids: Vec<LogFileId>,
    compact2_fids: Vec<LogFileId>,
    //若1个装满了，就弄个新的来装
    tarfid: LogFileId,
    pakvcaller: PaKVCtxChannelCaller,
}

impl CompactCtx {
    fn create(compactfrom: Vec<LogFileId>, pakvcaller: PaKVCtxChannelCaller) -> CompactCtx {
        //从compact from中选出tarfid
        let mut max=1;
        for fid in &compactfrom{
            if fid.id>max{
                max=fid.id;
            }
        }
        CompactCtx {
            compactfrom_fids: compactfrom,
            compact2_fids: vec![],
            tarfid: LogFileId{id:max+1},
            pakvcaller,
        }
    }
    pub fn compact_ifneed(ctx: &mut PaKVCtx, curpos: u64) {
        if curpos < LOG_FILE_MAX {
            return;
        }
        ctx.compacting=true;
        let fromids = scan_log_files();
        // ctx.tarfid.id += 1;
        // ctx.tarfid.touch_if_not_exist();
        // META_FILE_OPE.set_usertar_fid(ctx.tarfid.id);
        let compact = CompactCtx::create(
            fromids,
            ctx.channel_caller.clone(),
            );
        //更新目标id
        // META_FILE_OPE.with(|f|{
        //     .set_usertar_fid(compact.tarfid.id);
        // });
        ctx.meta_file_ope.set_usertar_fid(compact.tarfid.id);
        ctx.tarfid=compact.tarfid.clone();
        ctx.tarfid.touch_if_not_exist();

        std::thread::spawn(move || {
            compact.start_compact();
        });
    }
    fn start_compact(mut self) {
        let mut map_k2_opestr: HashMap<String, String> = HashMap::new();
        //1.压缩操作
        for f_ in &self.compactfrom_fids {
            if let Ok(f) = OpenOptions::new()
                .read(true)
                .open(f_.get_pathbuf()) {
                logfile_gothroughlogs(&f, |off, line, kvope| {
                    match &(kvope.ope) {
                        KvOpeE::KvOpeSet { k, v } => {
                            map_k2_opestr.entry(k.clone()).and_modify(|ss| {
                                let mut l = line.clone();
                                std::mem::swap(&mut l, ss);
                            }).or_insert(line.clone());
                        }
                        KvOpeE::KvOpeDel { k } => {
                            map_k2_opestr.remove(k);
                        }
                    }
                });
            }
        }

        //2.创建新的文件，并一条条写入
        self.add_new_compact2_fid();
        let mut fid = self.compact2_fids.last_mut().unwrap();
        let mut f = OpenOptions::new().write(true).open(&fid.get_pathbuf()).unwrap();

        let mut len = 0;
        let mut map_k2pos = HashMap::new();
        let mut wait =None;
        for (k, opestr) in map_k2_opestr {
            f.write(opestr.as_bytes());
            len += opestr.len();
            map_k2pos.insert(k, len as u64);
            if len> LOG_FILE_MAX as usize {
                let mut sendmap=HashMap::new();
                std::mem::swap(&mut sendmap,&mut map_k2pos);
                // 完成后将map交给主循环，更新hash里的数据
                wait=Some(self.pakvcaller.update_k_positions((fid).clone(),sendmap));
                len=0;

                self.add_new_compact2_fid();
                fid = self.compact2_fids.last_mut().unwrap();
                f = OpenOptions::new().write(true).open(&fid.get_pathbuf()).unwrap();
            }
        }
        if let Some(haswait)=wait{
            println!("waiting update");
            haswait.recv();//等待索引修改完毕
            //删除所有旧文件
            for ffid in self.compactfrom_fids{
                fs::remove_file(ffid.get_pathbuf()).unwrap();
                println!("remove {}",ffid.id);
            }
        }
        self.pakvcaller.end_compact();
    }
    pub fn add_new_compact2_fid(&mut self) {
        let mut max = 0;
        for fid in &self.compactfrom_fids {
            if (fid.id > max) {
                max = fid.id;
            }
        }
        for fid in &self.compact2_fids {
            if (fid.id > max) {
                max = fid.id;
            }
        }
        if (self.tarfid.id > max) {
            max = self.tarfid.id;
        }
        let add = LogFileId {
            id: max + 1
        };
        add.touch_if_not_exist();
        self.compact2_fids.push(add);
    }
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaFileStore{
    pub usertarfid:u64
}
impl MetaFileStore{
    pub fn default() -> MetaFileStore {
        MetaFileStore{
            usertarfid:1
        }
    }
}
pub struct MetaFileOpe{
    store:Option<MetaFileStore>
}
impl MetaFileOpe{
    pub fn create() -> MetaFileOpe {
        MetaFileOpe{
            store:None
        }
    }
    fn metafile_path() -> &'static str {
        return "./store/meta"
    }
    pub fn update2file(store:&MetaFileStore){
        let v=serde_json::to_string(store).unwrap();
        let mut f =OpenOptions::new().write(true).open(Path::new(MetaFileOpe::metafile_path())).unwrap();
        f.write(v.as_bytes());
    }

    pub fn makesure_exist(&self){
        OpenOptions::new().create(true).write(true).open(Path::new(MetaFileOpe::metafile_path()));
    }

    //在更新ctx里的tarfid时要跟着变
    pub fn set_usertar_fid(&mut self,id:u64){
        match &mut self.store{
            None => {
                self.store=Some(MetaFileStore::default());
            }
            Some( v) => {
                v.usertarfid=id;
            }
        }

        MetaFileOpe::update2file(self.store.as_ref().unwrap());
    }
    pub fn get_usertar_fid(&mut self)->u64{
        if let Some(v)=&self.store{
            return v.usertarfid;
        }else{
            //读取并解析成功，则之前有，否则设为默认值
            let f=OpenOptions::new().write(true).open(Path::new(MetaFileOpe::metafile_path())).unwrap();
            let mut reader =BufReader::new(f);
            let mut line=String::new();
            reader.read_line(&mut line);
            let r:serde_json::Result<MetaFileStore>=serde_json::from_str(&line);
            match r {
                Ok(v) => {
                    self.store=Some(v);
                    return self.store.as_ref().unwrap().usertarfid;
                }
                Err(_) => {
                    self.set_usertar_fid(1);
                    return 1;
                }
            }
        }
    }
}
// thread_local! (
//   pub  static   META_FILE_OPE:MetaFileOpe=MetaFileOpe{store:None});

// pub fn touch_file_ifnot_exist()
pub fn file_check(ctx: &mut PaKVCtx) {
    let path = Path::new(pathstr_of_logfile());
    //1.缓存文件文件夹
    fs::create_dir(path);
    ctx.meta_file_ope.makesure_exist();
    let mut tarfid =ctx.meta_file_ope.get_usertar_fid();
    // META_FILE_OPE.with(|mut f|{
    //     tarfid=f.get_usertar_fid();
    // });
    // OpenOptions::new().create(true).write(true).open(path).unwrap();

    // let meta=dir.metadata().unwrap();
    // if !meta.is_dir() {
    //     // panic!("store应该为文件夹");
    // }
    //3.遍历文件夹下的文件,恢复所有操作到内存，选定一个最小文件为后续持久化文件，
    let r = read_dir(path).unwrap();
    // let mut minfilelen = u64::MAX;
    // let mut minfilep = None;
    let mut rank_by_edit_time=BTreeMap::new();
    for dir_ in r {
        match dir_ {
            Ok(dir) => {
                println!("scanning log {}", dir.path().as_os_str().to_str().unwrap());
                let p = dir.path();
                let mut file = File::open(&p).unwrap();
                let mut fileid = LogFileId { id: 0 };
                if !fileid.set_by_logfile_path(&p){
                    println!("  skip");
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
                println!("{}", e);
            }
        }
    }
    fn file_readlogs(ctx:&mut PaKVCtx,fid:LogFileId,file:&mut File){
        logfile_gothroughlogs(file, |off, line, kvope| {
            // println!("recover ope {} {}",line_str,off);
            // println!("seek relative {}");
            //恢复操作
            match &kvope.ope {
                KvOpeE::KvOpeSet { k, v } => {
                    ctx.store.set(k.clone(), &FilePos {
                        file_id: fid.id,
                        pos: off,
                    })
                }
                KvOpeE::KvOpeDel { k } => {
                    ctx.store.del(k.clone());
                }
            }
        });
    }
    let mut tarfile =None;
    let mut latest_fid=0;
    //最新编辑的最后操作
    for (t,(id, mut file)) in rank_by_edit_time{
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
                ctx.tarfid.id=latest_fid;
                ctx.meta_file_ope.set_usertar_fid(latest_fid);
                // META_FILE_OPE.with(|mut f|{
                //     f.borrow_mut().set_usertar_fid(latest_fid);
                // })
            }else{
                ctx.tarfid.id=1;
                ctx.tarfid.touch_if_not_exist();
            }
        }
        Some(mut f) => {
            //找到了匹配的
            ctx.tarfid.id=tarfid;
            file_readlogs(ctx,ctx.tarfid.clone(),&mut f);
        }
    }
}