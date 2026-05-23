use criterion::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::Duration;

type CacheLine = [u64; 8];

const BYTES_PER_ELEMENT: usize = std::mem::size_of::<CacheLine>();

struct RandomMemoryBenchmark {
    vec: Vec<CacheLine>,
    order: Vec<usize>,
    write_seed: u64,
}

impl RandomMemoryBenchmark {
    fn new(size_bytes: usize) -> Self {
        let size_in_elements = size_bytes / BYTES_PER_ELEMENT;

        let mut vec = Vec::new();
        vec.resize(size_in_elements, [1, 2, 3, 4, 5, 6, 7, 8]);
        unsafe {
            let data = vec.as_mut_ptr() as *mut libc::c_void;
            libc::madvise(
                data,
                size_in_elements * std::mem::size_of::<CacheLine>(),
                libc::MADV_RANDOM,
            );
        }

        let mut order: Vec<usize> = (0..size_in_elements).collect();
        unsafe {
            let data = order.as_mut_ptr() as *mut libc::c_void;
            libc::madvise(
                data,
                size_in_elements * std::mem::size_of::<usize>(),
                libc::MADV_SEQUENTIAL,
            );
        }
        order.shuffle(&mut thread_rng());

        Self {
            vec,
            order,
            write_seed: 8,
        }
    }

    fn run_read_once(&self) -> u64 {
        let mut checksum = 0u64;
        for &index in &self.order {
            let value = self.vec[index];
            checksum = checksum.wrapping_add(value[0]);
            black_box(value);
        }
        checksum
    }

    fn run_write_once(&mut self) -> u64 {
        let seed = self.write_seed;
        self.write_seed = self.write_seed.wrapping_add(1);
        let payload = [
            seed,
            seed.wrapping_add(1),
            seed.wrapping_add(2),
            seed.wrapping_add(3),
            seed.wrapping_add(4),
            seed.wrapping_add(5),
            seed.wrapping_add(6),
            seed.wrapping_add(7),
        ];

        let mut checksum = 0u64;
        for &index in &self.order {
            self.vec[index] = payload;
            checksum = checksum.wrapping_add(self.vec[index][0]);
            black_box(self.vec[index]);
        }
        checksum
    }
}

fn memory_random_benchmark(c: &mut Criterion) {
    let size_bytes = n_gib_bytes!(1) as usize;
    let mut benchmark = RandomMemoryBenchmark::new(size_bytes);

    let mut group = c.benchmark_group("memory_random");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(25));
    group.throughput(Throughput::Bytes(size_bytes as u64));
    group.bench_function("read/64 B", |b| {
        b.iter(|| black_box(benchmark.run_read_once()))
    });
    group.bench_function("write/64 B", |b| {
        b.iter(|| black_box(benchmark.run_write_once()))
    });
    group.finish();
}

criterion_group!(benches, memory_random_benchmark);
