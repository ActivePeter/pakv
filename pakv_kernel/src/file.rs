use std::path::{ Path};
use std::fs::{File, OpenOptions};
use std::io::{  BufReader, Write, BufRead, Seek, SeekFrom};
use crate::serial::KvOpe;
#[derive(Clone)]
pub struct FilePos{
    pub offset:u64,
}


enum FileHandle {
    Reader(BufReader<File>),
    Writer(File),
    Temp,
}
impl FileHandle {
    pub fn reader_unwrap_moved(self) -> BufReader<File> {
        match self {
            FileHandle::Reader(c) => Some(c),
            _ => None,
        }.unwrap()
    }
    pub fn writer_unwrap_moved(self) -> File {
        match self {
            FileHandle::Writer(c) => Some(c),
            _ => None,
        }.unwrap()
    }
    pub fn reader_unwrap(&self) -> &BufReader<File> {
        match self {
            FileHandle::Reader(c) => Some(c),
            _ => None,
        }.unwrap()
    }
    pub fn reader_unwrap_mut(&mut self) -> &mut BufReader<File> {
        match self {
            FileHandle::Reader(c) => Some(c),
            _ => None,
        }.unwrap()
    }
    pub fn writer_unwrap(&self) -> &File {
        match self {
            FileHandle::Writer(c) => Some(c),
            _ => None,
        }.unwrap()
    }
    pub fn writer_unwrap_mut(&mut self) -> &mut File {
        match self {
            FileHandle::Writer(c) => Some(c),
            _ => None,
        }.unwrap()
    }
}

pub struct DbFileHandle{
    dbfile_path:String,
    dbfile_handle:FileHandle,
    r_offset:u64,
    w_offset:u64
}

impl DbFileHandle{
    pub fn create(path:String)->Option<DbFileHandle>{
        // println!("creare db file at",path);
        let f=OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .append(true)
            .open(path.clone());
        if let Ok(mut f)=f{
            let pos=f.seek(std::io::SeekFrom::End(0)).unwrap();
            return Some(DbFileHandle{
                dbfile_path:path,
                dbfile_handle:FileHandle::Writer(f),
                w_offset:pos,
                r_offset:0,
            })
        }
        else if let Err(e)=f{
            // log::error!("{}",e);
            eprintln!("{} path:{}",e,path);
        }
        None
    }
    fn switch_to_writer_if_is_reader(&mut self){
        //是reader，就将reader取出，换成writer
        let mut is_reader=false;
        if let FileHandle::Reader(_reader)=&self.dbfile_handle{
            is_reader=true;
        }
        if is_reader{
            let mut swapfilehandle=FileHandle::Temp;
            std::mem::swap(&mut self.dbfile_handle, &mut swapfilehandle);
            swapfilehandle=FileHandle::Writer(swapfilehandle.reader_unwrap_moved().into_inner());
            std::mem::swap(&mut self.dbfile_handle, &mut swapfilehandle);
            let pos=self.w_offset;
            self.dbfile_handle.writer_unwrap_mut().seek(SeekFrom::Start(pos)).unwrap();
        }
    }
    fn switch_to_reader_if_is_writer(&mut self){
        let mut is_writer=false;
        if let FileHandle::Writer(_f)=&self.dbfile_handle{
            is_writer=true;
        }
        if is_writer{
            let mut swapfilehandle=FileHandle::Temp;
            std::mem::swap(&mut self.dbfile_handle, &mut swapfilehandle);
            swapfilehandle=FileHandle::Reader(BufReader::new(swapfilehandle.writer_unwrap_moved()));
            std::mem::swap(&mut self.dbfile_handle, &mut swapfilehandle);
        }
    }
    //return position before append log
    pub fn append_log(&mut self,log:String)->FilePos{
        self.switch_to_writer_if_is_reader();
        let w=self.dbfile_handle.writer_unwrap().write(log.as_bytes()).unwrap();
        let ret=FilePos{
            offset:self.w_offset
        };
        self.w_offset+=w as u64;
        return ret;
    }
    pub fn get_log_by_pos(&mut self,fp:&FilePos)->KvOpe{
        self.switch_to_reader_if_is_writer();
        let reader=self.dbfile_handle.reader_unwrap_mut();
        reader.seek(SeekFrom::Start(fp.offset)).unwrap();
        let mut line_=String::new();
        let _n=reader.read_line(&mut line_);
        _n.unwrap();
        // self.fpos.w_offset=fp.w_offset+n.unwrap() as u64;
        KvOpe::from_str(&*line_).unwrap()
    }
    pub fn iter_start(&mut self){
        self.switch_to_reader_if_is_writer();
        self.r_offset=0;
        let reader=self.dbfile_handle.reader_unwrap_mut();
        reader.seek(SeekFrom::Start(0)).unwrap();
    }
    pub fn iter_readline(&mut self)->Option<(KvOpe,u64)>{
        self.switch_to_reader_if_is_writer();
        let reader=self.dbfile_handle.reader_unwrap_mut();
        let mut line_=String::new();
        let _n=reader.read_line(&mut line_);
        match _n{
            Ok(n) => {
                let ret=self.r_offset;
                self.r_offset+=n as u64;
                if line_.len()==0{
                    return None
                }
                Some((serde_json::from_str::<KvOpe>(&*line_).unwrap(),ret))
            },
            Err(e) => {
                eprintln!("readline fail {}",e);
                None
            },
        }
    }
}