use std::thread;
use std::sync::atomic::{AtomicBool, Ordering, AtomicU64};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::{PaKVCtx, KVIndexStore};
use crate::file::{DbFileHandle, FilePos};
use crate::serial::{KvOpe, KvOpeE};
use std::path::Path;
use std::cmp::{min, max};
use std::collections::{HashMap, VecDeque, HashSet};

// compress

// - compress threshold should be dynamic 

//   for example: first time compress threshold is 10, after compressing we get 5,
//   the next compress threshold should be 5+N or 5*N

// - writing old kvs to new file may take some time, so there should
//   be a schedule when compress writing take too long time 

// lv：思路层级
//lv0 用户优先级更高，压缩更懒

//lv1 用户请求进来，ur原子置1，用户线程等压缩线程让出（通过类似锁的东西）：压缩线程一段时间，发现ur原子为1，让出
//用户请求结束，ur原子置0， 设置ut用户请求结束时间，压缩线程确保用户请求一段时间后，且ur为0，开始压缩。

lazy_static::lazy_static! {
    pub static ref G_COMPRESSER : Compresser = Compresser::new();
}

pub struct CompressRes{
    pub map:HashMap<String, FilePos>,
    pub fhandle:DbFileHandle,
}
impl CompressRes{
    pub fn new(fhandle:DbFileHandle)->CompressRes{
        CompressRes{
            map:Default::default(),
            fhandle
        }
    }
}
pub struct Compresser{
    threshold :AtomicU64, //压缩阈值
    pub user_reading:AtomicBool,
    pub user_readtime:AtomicU64,
    pub compact_res:parking_lot::Mutex<Option<CompressRes>>,
    pub compacting:AtomicBool,
    pub new_opes_when_compact:parking_lot::Mutex<HashMap<String,FilePos>>,
    oldmap_sender:parking_lot::Mutex<Option<crossbeam_channel::Sender<HashMap<String,FilePos>>>>,
    // pub new_opes_wrote_idx:AtomicU64,
    thread_on:AtomicBool,
}

impl Compresser{
    pub fn new() -> Compresser{
        Compresser{
            threshold: AtomicU64::new(1000),
            user_reading: AtomicBool::new(false),
            user_readtime: AtomicU64::new(0),
            compact_res: Default::default(),
            compacting: AtomicBool::new(false),
            new_opes_when_compact: Default::default(),
            oldmap_sender: Default::default(),
            thread_on: AtomicBool::new(false),

        }
    }
    pub fn get()->&'static Compresser{
        return &G_COMPRESSER;
    }
    pub fn update_threshold(&self,logsz:u64){
        self.threshold.store(max(1000,logsz)*2,Ordering::Relaxed);
    }

    //先来了一部分压缩的，后来压缩过程中，又有用户请求，然后会吧一系列新操作加入到队列里
    // 直到用户结束后，在批量从队列读出来
    pub fn start_compress_if_need(&self,cur_f_sz:u64,store:&KVIndexStore,fpath:String,new_ope:Option<(String,FilePos)>,insert_or_del:bool){
// return;
        // println!("a_");
        if cur_f_sz<self.threshold.load(Ordering::Relaxed){
            return
        }
        if self.compacting.load(Ordering::Relaxed){
            //  压缩过程中出现的删除，先去看压缩过程中有没有已经写入的，（结果hash）
            //     如果有，就从结果hash移除
            //     如果没有. 不作操作
            //     对于追加map，需要清空（原本待追加，但是清除了）
            //  如果是insert，
            //     有没有都放到追加map

            if let Some((key,fp))= new_ope {
                // println!("start_compress_if_need compacting {}",key);
                if insert_or_del{
                    Compresser::get().new_opes_when_compact.lock().insert(key,fp);
                    // Compresser::get().new_opes_when_compact.lock().get(&key).unwrap();
                    // println!("start_compress_if_need compacting {}",key);
                }else{
                    Compresser::get().compact_res.lock().as_mut().unwrap().map.remove(&key);
                    Compresser::get().new_opes_when_compact.lock().remove(&key);
                }
            }
           return
        }
        self.compacting.store(true,Ordering::Relaxed);
        println!("start_compress_if_need");
        //todo 后续有新写入需要再告知压缩线程
        // let mut k2f =;
        // fn fpath2compfpath(fpath:&String) -> String {
        //     Path::new(&*fpath).parent().unwrap().to_str().unwrap().to_string()+"/comp"
        // }
        // let compfpath=fpath2compfpath(&fpath);
        // println!("a");
        let mut senderoption =self.oldmap_sender.lock();
        if senderoption.is_none(){
            let (s,r)=crossbeam_channel::bounded(1);
            senderoption.replace(s);
            println!("start comp thread");
            let _handle=thread::spawn(move ||{
                let r=r;
                //一个循环为一次压缩
                loop{
                    let k2f =r.recv();
                    if let Ok(mut k2f)=k2f{
                        //初始化结果集
                        // let mut comp=;
                        let mut h=DbFileHandle::create(fpath.clone()+"/db").unwrap();
                        Compresser::get().compact_res.lock().replace(CompressRes::new(
                            DbFileHandle::create(fpath.clone()+"/comp").unwrap()
                        ));
                        let mut iter=k2f.iter_mut();
                        loop{
                            //这个循环会sleep200ms，用于检测有压缩任务，和用户空闲的情况下，开始压缩
                            // 一次压缩结束后，跳出这个循环
                            let now=std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;

                            //有未完成压缩任务，用户空闲
                            if now-Compresser::get().user_readtime.load(Ordering::Relaxed)>200 //用户操作200ms后，如果compact标志位，开始压缩
                                &&Compresser::get().compacting.load(Ordering::Relaxed){
                                println!("part compact start");
                                //用于装压缩后的hash索引
                                let mut compact_res_hold_ = Compresser::get().compact_res.lock();
                                let mut compact_res_hold=compact_res_hold_.as_mut().unwrap();

                                //当user操作时停止，或者写了一定量，或者压缩结束
                                while !Compresser::get().user_reading.load(Ordering::Relaxed)&&
                                    Compresser::get().compacting.load(Ordering::Relaxed) {
                                    let mut beginpos=compact_res_hold.fhandle.get_w_offset();
                                    //写点数据
                                    loop {
                                        //return true when arrive threshold
                                        fn push_ope(comp:&mut DbFileHandle,ope:KvOpe,beginpos:u64,res_map:&mut HashMap<String, FilePos>) -> bool {
                                            let pos=match ope.ope {
                                                KvOpeE::KvOpeSet {k,v  }=>{
                                                    let newfp=comp.append_log(KvOpe::create(KvOpeE::KvOpeSet {k:k.clone(),v}).to_str()+"\n");
                                                    // targetfp.offset=newfp.offset;
                                                    res_map.insert(k,newfp);
                                                    comp.get_w_offset()
                                                },
                                                _ => {unreachable!()}
                                            };

                                            if pos-beginpos>1000{
                                                // println!("pause comp and check user");
                                                // break;
                                                return true;
                                            }
                                            false
                                        }
                                        match iter.next(){
                                            None => {
                                                let mut new_opes =Compresser::get().new_opes_when_compact.lock();
                                                {

                                                    let mut removekeys=Vec::new();
                                                    removekeys.reserve(new_opes.len());
                                                    //写入新加入的用户请求
                                                    //todo 优化追加，
                                                    {
                                                        let mut iter = new_opes.iter();
                                                        while let Some((k, fp)) = iter.next() {
                                                            // let mut ope=KvOpe::create(KvOpeE::KvOpeDel {k:"".to_string()});
                                                            // {//take out the ope
                                                            //     let ope1=new_opes.get_mut(Compresser::get().new_opes_wrote_idx.load(Ordering::Relaxed) as usize).unwrap();
                                                            //     Compresser::get().new_opes_wrote_idx.fetch_add(1,Ordering::Relaxed);
                                                            //     std::mem::swap(&mut ope,ope1);
                                                            // }
                                                            removekeys.push(k.clone());//记录已消费的log
                                                            let ope = h.get_log_by_pos(fp);
                                                            //达到一定量，检查用户请求
                                                            if push_ope(&mut compact_res_hold.fhandle, ope, beginpos, &mut compact_res_hold.map) {
                                                                break;
                                                            }
                                                        }
                                                        //free iter borrow
                                                    }
                                                    // removekeys.sort();
                                                    // print!("new ope when comp\n   ");
                                                    // for i in &removekeys{
                                                    //     print!("{},",i);
                                                    //     new_opes.remove(i).unwrap();
                                                    // }
                                                    // println!();
                                                    if new_opes.len()!=0{
                                                        break;
                                                    }
                                                    //没有追加了，压缩结束
                                                }
                                                //完成了全部压缩
                                                //阈值倍数增长
                                                Compresser::get().threshold.store(max(compact_res_hold.fhandle.get_w_offset(),1000)*2,Ordering::Relaxed);
                                                Compresser::get().compacting.store(false,Ordering::Relaxed);

                                                std::fs::rename(fpath.clone()+"/db",fpath.clone()+"/olddb"
                                                    // + &*std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()
                                                ).unwrap();
                                                std::fs::rename(fpath.clone()+"/comp",fpath.clone()+"/db").unwrap();
                                                // compact_res_hold.fhandle.get_log_by_pos()
                                                println!("all comped");
                                                break;}
                                            Some((k,fp)) => {
                                                let l=h.get_log_by_pos(fp);
                                                if push_ope(&mut compact_res_hold.fhandle, l, beginpos, &mut compact_res_hold.map){
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                println!("part end compe");
                                // let v=_hold.as_mut().unwrap(),
                                //释放锁
                            }

                            //压缩结束
                            if !Compresser::get().compacting.load(Ordering::Relaxed){
                                if
                                std::fs::read_dir(fpath.clone()).unwrap().filter(|f|{
                                    return f.as_ref().unwrap().file_name()=="olddb"
                                }).next().is_some(){
                                    std::fs::remove_file(fpath.clone()+"/olddb").unwrap();
                                }

                                break;
                            }

                            //1.内存化一部分要写入的数据，
                            //定时扫描
                            thread::sleep(Duration::from_millis(200));
                        }
                    }else{
                        println!("compress thread end");
                        break;
                    }
                    // hold.as_mut().unwrap().insert(Default::default());
                    // std::mem::swap(hold.as_mut().unwrap().as_mut().unwrap(), &mut k2f);
                }
            });


        }
        senderoption.as_mut().unwrap().send(store.map.clone()).unwrap();
        // println!("b");
    } 
}


