// use crate::pakv;
//
// use std::collections::HashMap;
// use std::fs::OpenOptions;
// use pakv::file::{logfile_gothroughlogs, LOG_FILE_MAX, LogFileId, scan_log_files};
// use pakv::file::serial::KvOpeE;
// use std::io::Write;
// use std::fs;
// // use pakv::channel_caller::PaKVCtxChanCallerForSys;
// use pakv::PaKVCtx;
//
// //这里存入id前必须保证有效性
// pub struct CompactCtx {
//     compactfrom_fids: Vec<LogFileId>,
//     compact2_fids: Vec<LogFileId>,
//     //若1个装满了，就弄个新的来装
//     tarfid: LogFileId,
//     pakvcaller: PaKVCtxChanCallerForSys,
// }
//
// impl CompactCtx {
//     fn create(compactfrom: Vec<LogFileId>, pakvcaller: PaKVCtxChanCallerForSys) -> CompactCtx {
//         //从compact from中选出tarfid
//         let mut max=1;
//         for fid in &compactfrom{
//             if fid.id>max{
//                 max=fid.id;
//             }
//         }
//         CompactCtx {
//             compactfrom_fids: compactfrom,
//             compact2_fids: vec![],
//             tarfid: LogFileId{id:max+1},
//             pakvcaller,
//         }
//     }
//     pub fn compact_ifneed(ctx: &mut PaKVCtx, curpos: u64) {
//         if curpos < LOG_FILE_MAX {
//             return;
//         }
//         info!("compact start");
//         ctx.compacting=true;
//
//         let fromids = scan_log_files();
//         // ctx.tarfid.id += 1;
//         // ctx.tarfid.touch_if_not_exist();
//         // META_FILE_OPE.set_usertar_fid(ctx.tarfid.id);
//         let compact = CompactCtx::create(
//             fromids,
//             ctx.sys_chan_caller.clone(),
//         );
//
//         info!("change tarfile, compact old files");
//         //更新目标id
//         ctx.meta_file_ope.set_usertar_fid(compact.tarfid.id);
//         ctx.tarfid=compact.tarfid.clone();
//         ctx.tarfid.touch_if_not_exist();
//
//         std::thread::spawn(move || {
//             info!("compact task begin");
//             compact.start_compact();
//         });
//     }
//     fn start_compact(mut self) {
//         let mut map_k2_opestr: HashMap<String, String> = HashMap::new();
//         //1.压缩操作
//         info!("collect compacted info");
//         for f_ in &self.compactfrom_fids {
//             if let Ok(f) = OpenOptions::new()
//                 .read(true)
//                 .open(f_.get_pathbuf()) {
//                 logfile_gothroughlogs(&f, |_, line, kvope| {
//                     match &(kvope.ope) {
//                         KvOpeE::KvOpeSet { k, v:_ } => {
//                             map_k2_opestr.entry(k.clone()).and_modify(|ss| {
//                                 ss.clone_from(line);
//                             }).or_insert(line.clone());
//                         }
//                         KvOpeE::KvOpeDel { k } => {
//                             map_k2_opestr.remove(k);
//                         }
//                     }
//                 });
//             }
//         }
//
//         //2.创建新的文件，并一条条写入
//         self.add_new_compact2_fid();
//         let mut fid = self.compact2_fids.last_mut().unwrap();
//         info!("create file for compact {}",fid.id);
//         let mut f = OpenOptions::new().write(true).open(&fid.get_pathbuf()).unwrap();
//
//         let mut len = 0;
//         let mut map_k2pos = HashMap::new();
//         let mut wait =None;
//         for (k, opestr) in map_k2_opestr {
//             f.write(opestr.as_bytes()).unwrap();
//             len += opestr.len();
//             map_k2pos.insert(k, len as u64);
//             if len> LOG_FILE_MAX as usize {
//                 let mut sendmap=HashMap::new();
//                 std::mem::swap(&mut sendmap,&mut map_k2pos);
//                 // 完成后将map交给主循环，更新hash里的数据
//                 wait=Some(self.pakvcaller.update_k_positions((fid).clone(),sendmap));
//                 len=0;
//
//
//                 self.add_new_compact2_fid();
//                 fid = self.compact2_fids.last_mut().unwrap();
//                 info!("create file for compact {}",fid.id);
//                 f = OpenOptions::new().write(true).open(&fid.get_pathbuf()).unwrap();
//             }
//         }
//         if let Some(haswait)=wait{
//             // println!("waiting update");
//             haswait.recv().unwrap();//等待索引修改完毕
//             info!("compact written done");
//             //删除所有旧文件
//             for ffid in self.compactfrom_fids{
//                 info!("remove old {}",ffid.id);
//                 fs::remove_file(ffid.get_pathbuf()).unwrap();
//                 // println!("remove {}",ffid.id);
//             }
//         }
//         self.pakvcaller.end_compact();
//     }
//     pub fn add_new_compact2_fid(&mut self) {
//         let mut max = 0;
//         for fid in &self.compactfrom_fids {
//             if fid.id > max {
//                 max = fid.id;
//             }
//         }
//         for fid in &self.compact2_fids {
//             if fid.id > max {
//                 max = fid.id;
//             }
//         }
//         if self.tarfid.id > max {
//             max = self.tarfid.id;
//         }
//         let add = LogFileId {
//             id: max + 1
//         };
//         add.touch_if_not_exist();
//         self.compact2_fids.push(add);
//     }
// }

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use crate::pakv::file::{FilePos, LogFileId};
use std::io::{BufWriter, Write, SeekFrom, Seek, BufRead};


const LOG_FILE_MAX:u64 =1 << 20;

pub struct Compactor{
    pub kv:HashMap<String, FilePos>,//从上下文拷贝来的kv数据
    pub kvranked:HashMap<u64,Vec<u64>>,//fid->offsets
    pub(crate) disactived_files:HashMap<u64,File>,//需要被压缩的文件
    pub(crate) curfid:u64,//当前目标文件
    pub compactto_fids:Vec<u64>,//已经压缩到的目标文件
}

impl Compactor{
    pub fn if_need_compact(curpos:u64)
        ->bool{
        curpos>=LOG_FILE_MAX
    }
    fn find_compactto_fid(&self) -> LogFileId {
        let mut max=0;
        if self.curfid>max {
            max=self.curfid
        }
        for (fid,_v) in &self.disactived_files{
            if fid> &max {
                max= *fid;
            }
        }

        LogFileId{
            id: max
        }
    }
    //对kv数据进行集合，同一个文件的放到一起，这样同一个文件的可以一起读
    pub fn calc_kvranked(&mut self){
        let mut rank:HashMap<u64,Vec<u64>>=Default::default();
        for (_,v) in &self.kv{
            let tarvec=rank.get_mut(&v.file_id);
            match tarvec{
                None => {
                    let mut vec=Vec::new();
                    vec.push(v.pos);
                    rank.insert(v.file_id,vec);
                }
                Some(vec) => {
                    vec.push(v.pos);
                }
            }
        }
        for (_,vec) in &mut rank{
            vec.sort_unstable();
        }

        self.kvranked=rank;
    }
    pub fn startpact(&self){
        //1.选取一个文件id，作为，压缩入的文件
        let fid=self.find_compactto_fid();
        let mut bufwriter =BufWriter::new(OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(fid.get_pathbuf()).unwrap());
        let mut writedcnt =0;
        //2.遍历已经计算好的需要读取的文件以及文件中的偏移
        for (fid,offs) in &self.kvranked{
            let mut fid =LogFileId{id: *fid };
            let mut reader =fid.open_reader();
            for off in offs{
                reader.seek(SeekFrom::Start(*off)).unwrap();
                let mut s = String::new();
                reader.read_line(&mut s).unwrap();
                bufwriter.write_all(s.as_bytes());
                writedcnt+=s.as_bytes().len();
                if writedcnt>= LOG_FILE_MAX as usize {
                    //超出大小，一个文件压缩完，
                    bufwriter.flush();//确保写入到文件
                    fid.id+=1;
                    bufwriter=BufWriter::new(
                        OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open(fid.get_pathbuf()).unwrap()
                    );
                    writedcnt=0;//writecnt 清零
                }
            }
        }
    }
}