use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pakv_server_lib::{server_app};
use pakv_server_lib::net::msg2app_sender::NetMsg2App;
use pakv_server_lib::server2client::Server2ClientSender;
// use criterion::async_executor::FuturesExecutor;

fn criterion_benchmark(c: &mut Criterion) {
    env_logger::init();//remember to set RUST_LOG=INFO

    let app=tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let (t,mut r)=tokio::sync::mpsc::channel(10);
    app.spawn(async move{
        server_app::PaKVServerApp::new().await
            .hold(r).await;
    });

    // c.bench_function("continuous write", |b| b.iter(|| {
    //     let (t1,r1)=NetMsg2App::make_result_chan();
    //     t.blocking_send(NetMsg2App::SetWithResultSender {
    //         sender: t1,
    //         k: "ggg".to_string(),
    //         v: "ggg".to_string()
    //     }).unwrap();
    //     let r=r1.blocking_recv().unwrap();
    // }));

    c.bench_function("set", |b| b.iter(|| {
        let (t1,r1)=NetMsg2App::make_result_chan();
        t.blocking_send(NetMsg2App::SetWithResultSender {
            sender: t1,
            k: "ggg".to_string(),
            v: "ggg".to_string()
        }).unwrap();
        let r=r1.blocking_recv().unwrap();
    }));


    // c.bench_function("get exist", |b| b.iter(|| {
    //     pakv_chan_handler.get("kskskskk".to_string());
    // }));
}

// fn criterion_benchmark2(c: &mut Criterion) {
//     // env_logger::init();//remember to set RUST_LOG=INFO
//     let pakv_chan_handler=pakv::start_kernel();
//     pakv_chan_handler.del("lalala".to_string());
//     pakv_chan_handler.set("ksksksk".to_string(),"sss".to_string());
//
//     c.bench_function("del not exist, set", |b| b.iter(|| {
//         pakv_chan_handler.del("lalala".to_string());
//         pakv_chan_handler.set("ksksksk".to_string(),"sss".to_string());
//     }));
//     c.bench_function("del exist, set", |b| b.iter(|| {
//         pakv_chan_handler.del("ksksksk".to_string());
//         pakv_chan_handler.set("ksksksk".to_string(),"sss".to_string());
//     }));
//     c.bench_function("set", |b| b.iter(|| {
//         // pakv_chan_handler.del("mmm".to_string());
//         pakv_chan_handler.set("mmm".to_string(),"sss".to_string());
//     }));
// }

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);