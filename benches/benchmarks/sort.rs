use criterion::*;
use std::time::Duration;

const TOTAL_SIZE: usize = n_mib_bytes!(1) as usize;

fn sort_benchmark(c: &mut Criterion) {
    let elements = TOTAL_SIZE / std::mem::size_of::<u64>();
    let input: Vec<u64> = (0..elements).map(|_| rand::random::<u64>()).collect();

    let mut group = c.benchmark_group("sort");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Bytes(TOTAL_SIZE as u64));
    group.bench_function("u64/1 MiB", |b| {
        b.iter_batched(
            || input.clone(),
            |mut values| {
                values.sort_unstable();
                black_box(values)
            },
            BatchSize::LargeInput,
        )
    });
    group.finish();
}

criterion_group!(benches, sort_benchmark);
