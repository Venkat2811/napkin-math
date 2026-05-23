use criterion::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn coordination_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordination");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(1));
    group.bench_function("mutex_contended", |b| {
        b.iter_custom(|iters| {
            let mutex = Arc::new(Mutex::new(0_u64));
            let stop = Arc::new(AtomicBool::new(false));
            let barrier = Arc::new(Barrier::new(2));

            let worker_mutex = Arc::clone(&mutex);
            let worker_stop = Arc::clone(&stop);
            let worker_barrier = Arc::clone(&barrier);

            let worker = thread::spawn(move || {
                worker_barrier.wait();
                while !worker_stop.load(Ordering::Relaxed) {
                    let mut data = worker_mutex.lock().unwrap();
                    *data = data.wrapping_add(1);
                    black_box(*data);
                }
            });

            barrier.wait();

            let start = Instant::now();
            for _ in 0..iters {
                let mut data = mutex.lock().unwrap();
                *data = data.wrapping_add(1);
                black_box(*data);
            }
            let duration = start.elapsed();

            stop.store(true, Ordering::Relaxed);
            worker.join().unwrap();

            duration
        })
    });
    group.finish();
}

criterion_group!(benches, coordination_benchmark);
