use criterion::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const BUFFER_SIZE: usize = n_kib_bytes!(8) as usize;
const FILE_FILL_CHUNK_SIZE: usize = n_mib_bytes!(8) as usize;
const SEQ_READ_FILE_SIZE: usize = n_gib_bytes!(1) as usize;
const RANDOM_READ_FILE_SIZE: usize = n_gib_bytes!(8) as usize;
const WRITE_WRAP_SIZE: u64 = n_gib_bytes!(1) as u64;

fn benchmark_file_path(suffix: &str) -> PathBuf {
    let base = std::env::var("NAPKIN_BENCH_FILE").unwrap_or_else(|_| String::from("/tmp/napkin.txt"));
    let base = PathBuf::from(base);

    if base.is_dir() {
        return base.join(format!("napkin-{suffix}.dat"));
    }

    let file_name = base
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("napkin.txt"));
    base.with_file_name(format!("{file_name}.{suffix}"))
}

#[cfg(target_os = "linux")]
fn drop_file_page_cache(file: &std::fs::File) {
    unsafe {
        libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED);
    }
}

#[cfg(not(target_os = "linux"))]
fn drop_file_page_cache(_file: &std::fs::File) {}

fn fill_file(file: &mut std::fs::File, total_size: usize) {
    let buffer = vec![0u8; FILE_FILL_CHUNK_SIZE];
    let mut remaining = total_size;

    while remaining > 0 {
        let write_len = remaining.min(buffer.len());
        file.write_all(&buffer[..write_len]).unwrap();
        remaining -= write_len;
    }

    file.sync_data().unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
}

struct PreparedFile {
    path: PathBuf,
    size_bytes: usize,
}

impl PreparedFile {
    fn new(path: PathBuf, size_bytes: usize) -> Self {
        let _ = fs::remove_file(&path);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        fill_file(&mut file, size_bytes);
        drop_file_page_cache(&file);
        drop(file);

        Self { path, size_bytes }
    }

    fn open_read(&self) -> std::fs::File {
        OpenOptions::new().read(true).open(&self.path).unwrap()
    }

    fn open_write_truncate(&self) -> std::fs::File {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .unwrap()
    }
}

impl Drop for PreparedFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct SequentialReadSample {
    file: std::fs::File,
    buffer: [u8; BUFFER_SIZE],
}

impl SequentialReadSample {
    fn new(prepared: &PreparedFile) -> Self {
        let mut file = prepared.open_read();
        file.seek(SeekFrom::Start(0)).unwrap();
        drop_file_page_cache(&file);

        #[cfg(target_os = "linux")]
        unsafe {
            libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_SEQUENTIAL);
        }

        Self {
            file,
            buffer: [0; BUFFER_SIZE],
        }
    }

    fn run(&mut self, iters: u64) -> Duration {
        let start = Instant::now();
        for _ in 0..iters {
            let n = self.file.read(&mut self.buffer).unwrap();
            if n == 0 {
                self.file.seek(SeekFrom::Start(0)).unwrap();
                self.file.read_exact(&mut self.buffer).unwrap();
            }
            black_box(self.buffer);
        }
        start.elapsed()
    }
}

struct RandomReadSample {
    file: std::fs::File,
    pages: Vec<u64>,
    i: usize,
    buffer: [u8; BUFFER_SIZE],
}

impl RandomReadSample {
    fn new(prepared: &PreparedFile) -> Self {
        let file = prepared.open_read();
        drop_file_page_cache(&file);

        #[cfg(target_os = "linux")]
        unsafe {
            libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_RANDOM);
        }

        let page_size = page_size::get();
        let mut pages = Vec::with_capacity(prepared.size_bytes / page_size);
        for i in 0..(prepared.size_bytes / page_size) {
            pages.push((i * page_size + 1) as u64);
        }
        pages.shuffle(&mut thread_rng());

        Self {
            file,
            pages,
            i: 0,
            buffer: [0; BUFFER_SIZE],
        }
    }

    fn run(&mut self, iters: u64) -> Duration {
        let start = Instant::now();
        for _ in 0..iters {
            if self.i == self.pages.len() {
                self.pages.shuffle(&mut thread_rng());
                self.i = 0;
                drop_file_page_cache(&self.file);
            }

            self.file.seek(SeekFrom::Start(self.pages[self.i])).unwrap();
            self.file.read_exact(&mut self.buffer).unwrap();
            black_box(self.buffer);
            self.i += 1;
        }
        start.elapsed()
    }
}

struct SequentialWriteSample {
    file: std::fs::File,
    buffer: [u8; BUFFER_SIZE],
    offset: u64,
}

impl SequentialWriteSample {
    fn new(prepared: &PreparedFile) -> Self {
        let file = prepared.open_write_truncate();
        let mut buffer = [0u8; BUFFER_SIZE];
        for byte in &mut buffer {
            *byte = rand::random::<u8>();
        }

        Self {
            file,
            buffer,
            offset: 0,
        }
    }

    fn run(&mut self, iters: u64, sync: bool) -> Duration {
        let start = Instant::now();
        for _ in 0..iters {
            if self.offset + BUFFER_SIZE as u64 > WRITE_WRAP_SIZE {
                self.file.seek(SeekFrom::Start(0)).unwrap();
                self.offset = 0;
            }

            self.file.write_all(&self.buffer).unwrap();
            if sync {
                self.file.sync_data().unwrap();
            }
            self.offset += BUFFER_SIZE as u64;
        }
        start.elapsed()
    }
}

fn sequential_disk_read_benchmark(c: &mut Criterion) {
    let prepared = OnceLock::new();
    let mut group = c.benchmark_group("disk");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("sequential_read/8 KiB", |b| {
        b.iter_custom(|iters| {
            let prepared = prepared.get_or_init(|| {
                PreparedFile::new(benchmark_file_path("criterion-sequential-read"), SEQ_READ_FILE_SIZE)
            });
            let mut sample = SequentialReadSample::new(prepared);
            sample.run(iters)
        })
    });
    group.finish();
}

fn random_disk_read_benchmark(c: &mut Criterion) {
    let prepared = OnceLock::new();
    let mut group = c.benchmark_group("disk");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("random_read/8 KiB", |b| {
        b.iter_custom(|iters| {
            let prepared = prepared.get_or_init(|| {
                PreparedFile::new(benchmark_file_path("criterion-random-read"), RANDOM_READ_FILE_SIZE)
            });
            let mut sample = RandomReadSample::new(prepared);
            sample.run(iters)
        })
    });
    group.finish();
}

fn sequential_disk_write_benchmark(c: &mut Criterion) {
    let no_fsync_prepared = OnceLock::new();
    let fsync_prepared = OnceLock::new();
    let mut group = c.benchmark_group("disk");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("write_no_fsync/8 KiB", |b| {
        b.iter_custom(|iters| {
            let prepared = no_fsync_prepared.get_or_init(|| {
                PreparedFile::new(benchmark_file_path("criterion-write-no-fsync"), BUFFER_SIZE)
            });
            let mut sample = SequentialWriteSample::new(prepared);
            sample.run(iters, false)
        })
    });
    group.bench_function("write_fsync/8 KiB", |b| {
        b.iter_custom(|iters| {
            let prepared = fsync_prepared.get_or_init(|| {
                PreparedFile::new(benchmark_file_path("criterion-write-fsync"), BUFFER_SIZE)
            });
            let mut sample = SequentialWriteSample::new(prepared);
            sample.run(iters, true)
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    sequential_disk_read_benchmark,
    random_disk_read_benchmark,
    sequential_disk_write_benchmark
);
