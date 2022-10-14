// use crate::pakv;

// use std::path::Path;
// use std::fs::OpenOptions;
// use std::io::{Write, BufReader, Read};
// use pakv::file::serial::MetaFileStore;

// pub struct MetaFileOpe{
//     store:Option<MetaFileStore>
// }
// impl MetaFileOpe{
//     pub fn create() -> MetaFileOpe {
//         MetaFileOpe{
//             store:None
//         }
//     }
//     fn metafile_path() -> &'static str {
//         return "./store/meta"
//     }
//     pub fn update2file(store:&MetaFileStore){
//         let v=serde_json::to_string(store).unwrap();
//         let mut f =OpenOptions::new().write(true).open(Path::new(MetaFileOpe::metafile_path())).unwrap();
//         f.write(v.as_bytes()).unwrap();
//     }

//     pub fn makesure_exist(&self){
//         if let Err(e)= OpenOptions::new().create(true).write(true).open(Path::new(MetaFileOpe::metafile_path())){
//             info!("确保meta文件存在 {}",e);
//         }
//     }

//     //在更新ctx里的tarfid时要跟着变
//     pub fn set_usertar_fid(&mut self,id:u64){
//         match &mut self.store{
//             None => {
//                 self.store=Some(MetaFileStore::default());
//                 self.store.as_mut().unwrap().usertarfid=id;
//             }
//             Some( v) => {
//                 v.usertarfid=id;
//             }
//         }

//         MetaFileOpe::update2file(self.store.as_ref().unwrap());
//     }
//     pub fn get_usertar_fid(&mut self)->u64{
//         if let Some(v)=&self.store{
//             return v.usertarfid;
//         }else{
//             //读取并解析成功，则之前有，否则设为默认值
//             let f=OpenOptions::new().read(true).open(Path::new(MetaFileOpe::metafile_path())).unwrap();
//             let mut reader =BufReader::new(f);
//             // reader.seek(SeekFrom::Start(0));
//             let mut line=String::new();
//             reader.read_to_string(&mut line).unwrap();
//             // println!("unserial {}",line);
//             let r:serde_json::Result<MetaFileStore>=serde_json::from_str(&line);
//             match r {
//                 Ok(v) => {
//                     // println!("  unserial ok");
//                     self.store=Some(v);
//                     return self.store.as_ref().unwrap().usertarfid;
//                 }
//                 Err(_) => {
//                     self.set_usertar_fid(1);
//                     return 1;
//                 }
//             }
//         }
//     }
// }
// // thread_local! (