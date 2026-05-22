use criterion::*;
use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::time::Duration;

const PAYLOAD_SIZE: usize = 64;

fn random_payload() -> Vec<u8> {
    (0..PAYLOAD_SIZE).map(|_| rand::random::<u8>()).collect()
}

fn hash_benchmark(c: &mut Criterion) {
    let input = random_payload();

    let mut group = c.benchmark_group("hash");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Bytes(PAYLOAD_SIZE as u64));
    group.bench_function("sha256/64 B", |b| {
        b.iter(|| black_box(Sha256::digest(black_box(&input))))
    });
    group.bench_function("crc32/64 B", |b| {
        b.iter(|| {
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(black_box(&input));
            black_box(hasher.finalize())
        })
    });
    group.bench_function("siphash/64 B", |b| {
        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            hasher.write(black_box(&input));
            black_box(hasher.finish())
        })
    });
    group.finish();
}

criterion_group!(benches, hash_benchmark);
