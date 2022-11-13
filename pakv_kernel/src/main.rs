use pakv_kernel::PaKVCtx;
use std::time::Duration;
use std::collections::HashMap;

// #[test]
// fn test1(){
//     let mut test =HashMap::<i32,i32>::new();
//     for i in 1..1000{
//         test.insert(i,i);
//     }
//     let mut vec=Vec::new();
//     let mut iter=test.iter();
//     while let Some(v)=iter.next(){
//         vec.push(v);
//     }
//     vec.sort();
//     for v in &vec{
//         print!("{},",v.1);
//     }
//     println!();
//     println!("sz {}",vec.len());
// }

fn main() {
    let kv=PaKVCtx::create();

    // c.bench_function("continuous write", |b| b.iter(|| {
    //     let (t1,r1)=NetMsg2App::make_result_chan();
    //     t.blocking_send(NetMsg2App::SetWithResultSender {
    //         sender: t1,
    //         k: "ggg".to_string(),
    //         v: "ggg".to_string()
    //     }).unwrap();
    //     let r=r1.blocking_recv().unwrap();
    // }));
    // for iii in 1..20000 {
    //     // println!("{}",iii);
    //     // c.bench_function("set", |b| b.iter(|| {
    //     kv.set("ggg".to_string(),"ggg".to_string());
    //     kv.set("ggg".to_string(),"sss".to_string());
    //     // }));
    // }
    for iii in 1..20000 {
        if iii%1000==0{
            println!("sleep 1s {}",iii);
            //compress will finish when user not busy
            std::thread::sleep(Duration::from_millis(1000));
            println!("hhh");
            for jjj in if iii==1000 { iii - 999 }else {iii-1000}..iii{
                let v=kv.get(format!("ggg{}",jjj)).unwrap();
                if v !=format!("jjj{}",jjj){
                    panic!("not equal {} {}",jjj,v)
                }
                println!("get ggg{} ok{}",jjj,v);
            }
            println!("hhh");
        }
        // println!("{}",iii);
        // c.bench_function("set", |b| b.iter(|| {
        // kv.set("ggg".to_string(),"ggg".to_string());
        kv.set(format!("ggg{}",iii),format!("jjj{}",iii));
        // }));
    }
    std::thread::sleep(Duration::from_millis(10000));
    // println!("{}",kv.get("ggg".to_string()).unwrap());
}