use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File, read_dir};
use std::{fs, thread};
use std::io::{Error, Read, Write};
use std::sync;
use crate::file::{file_check, KvOpe, KvOpeE};
use std::sync::mpsc::{RecvError, Sender, Receiver};

pub struct KVStore{
    map:HashMap<String,String>
}
impl KVStore{
    pub fn create() -> KVStore {
        return KVStore{
            map:HashMap::new()
        }
    }
    pub fn do_ope(&mut self,ope:&KvOpe){
        match &ope.ope{
            KvOpeE::KvOpeSet {k,v } => {
                self.set(k.clone(),v.clone());
            }
            KvOpeE::KvOpeDel { k} => {
                self.del(k.clone());
            }
        }
    }
    pub fn set(&mut self,k:String,v:String){
        self.map.entry(k).and_modify(|mut v1|{
            *v1=v.clone();
        }).or_insert(v);
    }
    pub fn get(&self, k:String) -> Option<&String> {
        return self.map.get(&k);
    }
    pub fn del(&mut self, k:String) -> Option<String> {
        self.map.remove(&k)
    }
}

pub enum UserKvOpe{
    KvOpeSet{k:String,v:String,
        // resp:sync::mpsc::Sender<bool>
    },
    KvOpeDel{k:String,
        // resp:sync::mpsc::Sender<bool>
    },
    KvOpeGet{k:String,
        resp:sync::mpsc::Sender<Option<String>>},
}
impl UserKvOpe{
    pub fn create_get_chan() -> (Sender<Option<String>>, Receiver<Option<String>>) {
        let c:(
        sync::mpsc::Sender<Option<String>>,
        sync::mpsc::Receiver<Option<String>>
        )=sync::mpsc::channel();

        c
    }
}
pub struct PaKVCtx{
    pub store:KVStore,
    pub tarfpath:Option<PathBuf>,
}
impl PaKVCtx{
    pub fn create() -> PaKVCtx {
        return PaKVCtx{
            store: KVStore::create(),
            tarfpath: None
        }
    }
    fn append_log(&mut self,str:String){
        match self.tarfpath.clone(){
            None => {
                panic!("tar path is not chosen");
            }
            Some(p) => {
                let mut filer = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(p);
                match filer{
                    Ok(mut file) => {

                        file.write(str.as_bytes()).unwrap();
                        // let v=["\n"];
                        file.write("\n".as_bytes()).unwrap();

                        // let s=&*str;
                        // if let Err(e) = writeln!(file,s) {
                        //     eprintln!("Couldn't write to file: {}", e);
                        // }
                    }
                    Err(_) => {

                        panic!("open tar file failed")
                    }
                }
            }
        }


    }
    pub fn set(&mut self, k:String, v:String){
        //1.log
        let ope=KvOpe{
            ope: KvOpeE::KvOpeSet {k:k.clone(),v:v.clone()}
        };
        self.append_log(ope.to_line_str().unwrap());
        self.store.set(k,v);
    }
    pub fn del(&mut self, k:String){
        //1.log
        let ope=KvOpe{
            ope: KvOpeE::KvOpeDel {k:k.clone()}
        };
        self.append_log(ope.to_line_str().unwrap());
        self.store.del(k);
    }
    pub fn get(&self, k:String) -> Option<&String> {
        self.store.get(k)
    }
}
pub fn start_kernel() -> Sender<UserKvOpe> {
    let mut ctx=PaKVCtx::create();
    file_check(&mut ctx);
    let (tx,rx)
        :(sync::mpsc::Sender<UserKvOpe>,
          sync::mpsc::Receiver<UserKvOpe>)
        =sync::mpsc::channel();
    fn handle_ope(ctx:&mut PaKVCtx, ope:UserKvOpe){
        match ope{
            UserKvOpe::KvOpeSet {
                k,v } => {
                ctx.set(k,v);
            }
            UserKvOpe::KvOpeDel { k } => {
                ctx.del(k);
            }
            UserKvOpe::KvOpeGet {
                k,resp } => {
                match ctx.get(k){
                    None => {
                        resp.send(None);
                    }
                    Some(v) => {
                        resp.send(Some(v.clone()));
                    }
                }
            }
        }
    }
    let handle = thread::spawn(move || {
        loop {
            let r=rx.recv();
            match r{
                Ok(rr) => {
                    handle_ope(&mut ctx,rr);
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    tx
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_none() {
        let mut kvs=KVStore::create();
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(kvs.get(("1").to_owned()), None);
        assert_eq!(kvs.get("2".to_owned()), None);
    }

    #[test]
    fn test_add_get() {
        let mut kvs=KVStore::create();
        kvs.set("1".to_owned(),"111".to_owned());
        kvs.set("2".to_owned(),"222".to_owned());
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(kvs.get("1".to_owned()).unwrap(), &"111".to_owned());
        assert_eq!(kvs.get("2".to_owned()).unwrap(), &"222".to_owned());
    }

    #[test]
    fn test_del() {
        let mut kvs=KVStore::create();
        kvs.set("1".to_owned(),"111".to_owned());
        kvs.set("2".to_owned(),"222".to_owned());
        kvs.del("1".to_owned());
        kvs.del("2".to_owned());
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(kvs.get("1".to_owned()), None);
        assert_eq!(kvs.get("2".to_owned()), None);
    }
}