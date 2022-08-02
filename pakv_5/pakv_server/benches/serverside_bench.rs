use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pakv_server_lib::pakv;

fn criterion_benchmark(c: &mut Criterion) {
    env_logger::init();//remember to set RUST_LOG=INFO
    let pakv_chan_handler=pakv::start_kernel();
    pakv_chan_handler.del("ksksksk".to_string());
    pakv_chan_handler.set("kskskskk".to_string(),"ss".to_string());
    c.bench_function("get not exist", |b| b.iter(|| {
        pakv_chan_handler.get("ksksksk".to_string());
    }));
    c.bench_function("get exist", |b| b.iter(|| {
        pakv_chan_handler.get("kskskskk".to_string());
    }));
}

fn criterion_benchmark2(c: &mut Criterion) {
    // env_logger::init();//remember to set RUST_LOG=INFO
    let pakv_chan_handler=pakv::start_kernel();
    pakv_chan_handler.del("lalala".to_string());
    pakv_chan_handler.set("ksksksk".to_string(),"sss".to_string());

    c.bench_function("del not exist, set", |b| b.iter(|| {
        pakv_chan_handler.del("lalala".to_string());
        pakv_chan_handler.set("ksksksk".to_string(),"sss".to_string());
    }));
    c.bench_function("del exist, set", |b| b.iter(|| {
        pakv_chan_handler.del("ksksksk".to_string());
        pakv_chan_handler.set("ksksksk".to_string(),"sss".to_string());
    }));
    c.bench_function("set", |b| b.iter(|| {
        // pakv_chan_handler.del("mmm".to_string());
        pakv_chan_handler.set("mmm".to_string(),"sss".to_string());
    }));
}

criterion_group!(benches, criterion_benchmark,criterion_benchmark2);
criterion_main!(benches);