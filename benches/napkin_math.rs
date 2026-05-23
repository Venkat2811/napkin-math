mod benchmarks;
#[macro_use]
extern crate byte_unit;
use criterion::criterion_main;

criterion_main! {
    benchmarks::memory_read::benches,
    benchmarks::compressed_memory_read::benches,
    benchmarks::hash::benches,
    benchmarks::memory_random::benches,
    benchmarks::sort::benches,
    benchmarks::syscall::benches,
}
