use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pakv_kernel::PaKVCtx;
// use criterion::async_executor::FuturesExecutor;

fn criterion_benchmark(c: &mut Criterion) {
    let pakv=PaKVCtx::create();
let mut i=0;
    c.bench_function("set", |b| b.iter(|| {
        pakv.set(format!("{}",i),"lll".to_string());
        // pakv.set("hhh".to_string(),"mmm".to_string());
        i=i+1;
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