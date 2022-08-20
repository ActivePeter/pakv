use std::path::{ Path};
use std::fs::{ File};
use std::fs;
use std::io::{  BufReader, Write, BufRead};
use crate::pakv::PaKVCtx;
use std::collections::BTreeMap;
use crate::pakv::file::{LogFileId, FilePos};
use crate::pakv::file::serial::{KvOpe, KvOpeE};
use std::time::SystemTime;

static FDIR: &str = "./store/";

//返回写入长度
pub fn file_append_log(file: &mut File, mut str: String) -> u64 {
    str.push('\n');

    let bytes=str.as_bytes();
    // file.
    file.write_all(bytes).unwrap();

    return bytes.len() as u64;
}


//遍历文件中的操作
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
                let ope = KvOpe::from_str(&line_str).unwrap();

                handle_one_ope(off, &line_str, &ope);
                off += l as u64;
            }
            Err(e) => {//读取行失败
                println!("{}", e);
                break;
            }
        }
    }
}

pub fn get_folder_path() -> &'static Path {
     Path::new(FDIR)
}

pub fn get_dirfiles_rank_by_time<P: AsRef<Path>>(
    path: P
) -> BTreeMap<SystemTime, (LogFileId, File)> {
    let r = fs::read_dir(path).unwrap();
    let mut rank_by_edit_time = BTreeMap::new();
    for dir_ in r {
        match dir_ {
            Ok(dir) => {
                // println!("scanning log {}", dir.path().as_os_str().to_str().unwrap());
                let p = dir.path();
                let file = File::open(&p).unwrap();
                let mut fileid = LogFileId { id: 0 };
                if !fileid.set_by_logfile_path(&p) {
                    // println!("  skip");
                    continue;
                }
                match file.metadata() {
                    Ok(meta) => {
                        let t = meta.modified().unwrap();
                        rank_by_edit_time.insert(t, (fileid, file));
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

    rank_by_edit_time
}

pub async fn file_check(ctx: &mut PaKVCtx) {
    let path = Path::new(FDIR);
    //1.缓存文件文件夹
    if let Err(e) = fs::create_dir(path) {
        // eprintln!("{}",e);
        info!("make sure store folder exist {}",e);
    }
    ctx.meta_file_ope.makesure_exist();
    let tarfid = ctx.meta_file_ope.get_usertar_fid();

    //3.遍历文件夹下的文件,对文件编辑时间进行排序
    let rank_by_edit_time=get_dirfiles_rank_by_time(path);

    //从文件中读取记录到内存中
    fn file_readlogs(ctx: &mut PaKVCtx, fid: LogFileId, file: &mut File) {
        info!("recovering from log file {}",fid.id);
        logfile_gothroughlogs(file, |off, _line, kvope| {
            // println!("recover ope {} {}",line_str,off);
            // println!("seek relative {}");
            //恢复操作
            match &kvope.ope {
                KvOpeE::KvOpeSet { k, v: _ } => {
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
    let mut tarfile = None;
    let mut latest_fid = 0;
    info!("begin to recover hash index from logs，and make sure tarfile {} exist",tarfid);
    //最新编辑的最后操作
    for (_t, (id, mut file)) in rank_by_edit_time {
        // println!("cur file id {} tarfid {}",id.id,tarfid);
        if id.id == tarfid {//meta中标记的tarfile是否存在
            tarfile = Some(file);
            continue;
        }
        file_readlogs(ctx, id.clone(), &mut file);
        latest_fid = id.id;
    }
    match tarfile {//meta中标记的tarfile是否存在
        None => {
            //没有找到目标存储文件，则选定一个序号作为目标文件
            if latest_fid != 0 {
                info!("didnt find tarfile,the latest edit file {} would be chosen for tarfile",latest_fid);
                ctx.tarfid.id = latest_fid;
                ctx.meta_file_ope.set_usertar_fid(latest_fid);
                // META_FILE_OPE.with(|mut file|{
                //     file.borrow_mut().set_usertar_fid(latest_fid);
                // })
            } else {
                info!("no log file exists,tarfile would be 1");
                ctx.tarfid.id = 1;
                ctx.tarfid.touch_if_not_exist();
            }
        }
        Some(mut f) => {
            info!("the tarfile id {} in meta is valid, would be set as current tarfile",tarfid);
            //找到了匹配的
            ctx.tarfid.id = tarfid;
            file_readlogs(ctx, ctx.tarfid.clone(), &mut f);
        }
    }
    info!("index recovering finished!");
}
