use criterion::{black_box, criterion_group, criterion_main, Criterion};


fn criterion_benchmark(c: &mut Criterion) {

    c.bench_function("del", |b| b.iter(|| {
        pakv_client_lib::client::PakVClient::new().oneshot("del a".to_string());
    }));
    c.bench_function("set", |b| b.iter(|| {
        pakv_client_lib::client::PakVClient::new().oneshot("set a b".to_string());
    }));
    c.bench_function("get", |b| b.iter(|| {
        // pakv_chan_handler.del("mmm".to_string());
        pakv_client_lib::client::PakVClient::new().oneshot("get a".to_string());
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);