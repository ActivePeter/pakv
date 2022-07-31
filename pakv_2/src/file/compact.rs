use std::collections::HashMap;
use std::fs::OpenOptions;
use crate::file::{logfile_gothroughlogs, LOG_FILE_MAX, LogFileId, scan_log_files};
use crate::file::serial::KvOpeE;
use std::io::Write;
use std::fs;
use crate::pakv::{PaKVCtx, PaKVCtxChannelCaller};

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
        // META_FILE_OPE.with(|file|{
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
                logfile_gothroughlogs(&f, |_, line, kvope| {
                    match &(kvope.ope) {
                        KvOpeE::KvOpeSet { k, v:_ } => {
                            map_k2_opestr.entry(k.clone()).and_modify(|ss| {
                                ss.clone_from(line);
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
            f.write(opestr.as_bytes()).unwrap();
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
            // println!("waiting update");
            haswait.recv().unwrap();//等待索引修改完毕
            //删除所有旧文件
            for ffid in self.compactfrom_fids{
                fs::remove_file(ffid.get_pathbuf()).unwrap();
                // println!("remove {}",ffid.id);
            }
        }
        self.pakvcaller.end_compact();
    }
    pub fn add_new_compact2_fid(&mut self) {
        let mut max = 0;
        for fid in &self.compactfrom_fids {
            if fid.id > max {
                max = fid.id;
            }
        }
        for fid in &self.compact2_fids {
            if fid.id > max {
                max = fid.id;
            }
        }
        if self.tarfid.id > max {
            max = self.tarfid.id;
        }
        let add = LogFileId {
            id: max + 1
        };
        add.touch_if_not_exist();
        self.compact2_fids.push(add);
    }
}