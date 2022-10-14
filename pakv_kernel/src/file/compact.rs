use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use crate::pakv::file::{FilePos, LogFileId};
use std::io::{BufWriter, Write, SeekFrom, Seek, BufRead, BufReader};
use std::fs;
use crate::pakv::KernelOtherWorkerSend2SelfMain;
use crate::pakv::KernelWorker2Main::BunchUpdate;


const LOG_FILE_MAX:u64 =1 <<
8;
//20;

pub struct Compactor{
    pub kv:HashMap<String, FilePos>,//从上下文拷贝来的kv数据
    pub kvranked:HashMap<u64,Vec<(String,u64)>>,//fid->(k,offset)s
    pub(crate) disactived_files:HashMap<u64,()>,//需要被压缩的文件
    pub(crate) curfid:u64,//当前目标文件
    pub compact_to_files:HashMap<u64,()>,
}

impl Compactor{
    pub fn new() -> Compactor {
        Compactor{
            kv: Default::default(),
            kvranked: Default::default(),
            disactived_files: Default::default(),
            curfid: 0,
            compact_to_files: Default::default()
        }
    }
    pub fn if_need_compact(curpos:u64)
        ->bool{
        println!("if_need_compact {} {}",curpos,LOG_FILE_MAX);
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
        println!("find max {}",max);
        LogFileId{
            id: max+1
        }
    }
    //对kv数据进行集合，同一个文件的放到一起，这样同一个文件的可以一起读
    pub fn calc_kvranked(&mut self){
        // fid -> k,logoff
        let mut rank:HashMap<u64,Vec<(String,u64)>>=Default::default();
        for (k,v) in &self.kv{
            let tarvec=rank.get_mut(&v.file_id);
            match tarvec{
                None => {
                    let mut vec=Vec::new();
                    vec.push((k.clone(),v.pos));
                    rank.insert(v.file_id,vec);
                }
                Some(vec) => {
                    vec.push((k.clone(),v.pos));
                }
            }
        }
        for (fid,vec) in &mut rank{
            self.disactived_files.insert(*fid,());
            vec.sort_unstable();
        }

        self.kvranked=rank;
    }
    pub fn startpact(&mut self,send2main: &KernelOtherWorkerSend2SelfMain){
        println!("start compact");
        //1.选取一个文件id，作为，压缩入的文件
        let mut fid =self.find_compactto_fid();
        let mut bufwriter =BufWriter::new(OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(fid.get_pathbuf()).unwrap());
        let mut writedcnt =0;
        let mut logoffs=Vec::new();
        fn sync_logoffs_to_main(logoffs:&mut Vec<(String,usize)>,
                                send2main: &KernelOtherWorkerSend2SelfMain,
        fid:u64
        ){
            let mut newlogoffs=Vec::new();
            std::mem::swap(&mut newlogoffs,logoffs);
            send2main.sender.blocking_send(
                BunchUpdate {
                    fileid: fid,
                    k2off:newlogoffs
                }
            ).unwrap();
        }
        //2.遍历已经计算好的需要读取的文件以及文件中的偏移，写入文件
        for (rfid,offs) in &mut self.kvranked{
            let mut rfid =LogFileId{id: *rfid };
            let mut reader =BufReader::new(
                // self.disactived_files.get(&fid.id).unwrap());
            rfid.open_reader());
            for (k,off) in offs{
                reader.seek(SeekFrom::Start(*off)).unwrap();
                let mut s = String::new();
                reader.read_line(&mut s).unwrap();
                bufwriter.write_all(s.as_bytes()).unwrap();
                logoffs.push((k.clone(),writedcnt));
                writedcnt+=s.as_bytes().len();
                if writedcnt>= LOG_FILE_MAX as usize {
                    println!("one file filled {} {}",writedcnt,fid.id);
                    //超出大小，一个文件压缩完，
                    //0.更新索引到主主循环
                    sync_logoffs_to_main(&mut logoffs,send2main,fid.id);
                    //1.确保写入到文件
                    bufwriter.flush().unwrap();
                    //2.记录文件
                    self.compact_to_files.insert(
                        fid.id,()
                        // bufwriter.into_inner().unwrap()
                    );
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
        //3.最后一个写入的文件不一定写满了，但是要记录
        if writedcnt>0{
            println!("one file filled {} {}",writedcnt,fid.id);
            self.compact_to_files.insert(
                fid.id,()//bufwriter.into_inner().unwrap()
            );
            bufwriter.flush().unwrap();
            //3.1 也要同步
            sync_logoffs_to_main(&mut logoffs,send2main,fid.id);

        }else{
            fs::remove_file(fid.get_pathbuf()).unwrap();
        }
        //4.删除所有的disactive文件
        for (fid,_offs) in &self.kvranked{
            fs::remove_file(LogFileId{
                id: *fid
            }.get_pathbuf()).unwrap();
        }
        //5.将这些文件更新到disactive里
        std::mem::swap(&mut self.compact_to_files,&mut self.disactived_files);
    }
}