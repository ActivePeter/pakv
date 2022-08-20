use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use crate::pakv::file::{FilePos, LogFileId};
use std::io::{BufWriter, Write, SeekFrom, Seek, BufRead, BufReader};
use std::fs;


const LOG_FILE_MAX:u64 =1 << 20;

pub struct Compactor{
    pub kv:HashMap<String, FilePos>,//从上下文拷贝来的kv数据
    pub kvranked:HashMap<u64,Vec<u64>>,//fid->offsets
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
    pub fn startpact(&mut self){
        //1.选取一个文件id，作为，压缩入的文件
        let fid=self.find_compactto_fid();
        let mut bufwriter =BufWriter::new(OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(fid.get_pathbuf()).unwrap());
        let mut writedcnt =0;
        //2.遍历已经计算好的需要读取的文件以及文件中的偏移，写入文件
        for (fid,offs) in &self.kvranked{
            let mut fid =LogFileId{id: *fid };
            let mut reader =BufReader::new(
                // self.disactived_files.get(&fid.id).unwrap());
            fid.open_reader());
            for off in offs{
                reader.seek(SeekFrom::Start(*off)).unwrap();
                let mut s = String::new();
                reader.read_line(&mut s).unwrap();
                bufwriter.write_all(s.as_bytes()).unwrap();
                writedcnt+=s.as_bytes().len();
                if writedcnt>= LOG_FILE_MAX as usize {
                    //超出大小，一个文件压缩完，

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
        self.compact_to_files.insert(
            fid.id,()//bufwriter.into_inner().unwrap()
        );
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