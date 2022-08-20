use std::sync::atomic::{AtomicUsize, Ordering};
use crate::server_app;
use crate::net::msg2app_sender::NetMsg2App;

static TOTAL: AtomicUsize = AtomicUsize::new(0);

pub fn conccurent_bench(){
    let (t, r)=tokio::sync::mpsc::channel(10);
    tokio::spawn(async move{
        server_app::PaKVServerApp::new().await
            .hold(r).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(10));
    //创建多个task，进行set并等待结果，看一秒内，有多少个成功收到结果
    for _i in 0..10{
        let t = t.clone();
        tokio::spawn(async move {
            loop {
                let (t1, r1) =
                    NetMsg2App::make_result_chan();
                t.send(NetMsg2App::SetWithResultSender {
                    sender: t1,
                    k: "ggg".to_string(),
                    v: "ggg".to_string()
                }).await.unwrap();
                let _r = r1.await.unwrap();
                TOTAL.fetch_add(1, Ordering::Release);//内存屏障，防止乱序优化
            }
        });
    }

    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("total set {}",TOTAL.load(Ordering::Relaxed));
}