use criterion::*;
use std::fs;
use std::process;
use std::time::{Duration, SystemTime};

fn syscall_benchmark(c: &mut Criterion) {
    let file = fs::File::open("/tmp").unwrap();

    let mut group = c.benchmark_group("syscall");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("getpid", |b| b.iter(|| black_box(process::id())));
    group.bench_function("gettimeofday", |b| {
        b.iter(|| black_box(SystemTime::now()))
    });
    group.bench_function("getrusage", |b| {
        b.iter(|| unsafe {
            let mut rusage = std::mem::MaybeUninit::<libc::rusage>::uninit();
            libc::getrusage(libc::RUSAGE_SELF, rusage.as_mut_ptr());
            black_box(rusage.assume_init())
        })
    });
    group.bench_function("stat", |b| {
        b.iter(|| black_box(file.metadata().unwrap()))
    });
    group.finish();
}

criterion_group!(benches, syscall_benchmark);
