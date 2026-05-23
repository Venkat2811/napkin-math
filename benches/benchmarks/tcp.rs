use criterion::*;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

const BUFFER_SIZE: usize = n_kib_bytes!(32) as usize;

fn spawn_tcp_echo_server() -> String {
    let (tx, rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        tx.send(listener.local_addr().unwrap()).unwrap();

        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(error) => panic!("{}", error),
            };

            stream.set_nodelay(true).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_millis(1000)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_millis(1000)))
                .unwrap();

            let mut buffer = [0u8; BUFFER_SIZE];
            loop {
                match stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => stream.write_all(&buffer[..n]).unwrap(),
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock
                                | std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::UnexpectedEof
                        ) =>
                    {
                        break;
                    }
                    Err(error) => panic!("{}", error),
                }
            }
        }
    });

    rx.recv().unwrap().to_string()
}

fn tcp_benchmark(c: &mut Criterion) {
    let endpoint = OnceLock::new();
    let payload: Vec<u8> = (0..BUFFER_SIZE).map(|_| rand::random::<u8>()).collect();

    let mut group = c.benchmark_group("tcp");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("echo/32 KiB", |b| {
        b.iter_custom(|iters| {
            let endpoint = endpoint.get_or_init(spawn_tcp_echo_server);
            let mut stream = TcpStream::connect(endpoint).unwrap();
            stream.set_nodelay(true).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_millis(1000)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_millis(1000)))
                .unwrap();
            let mut buffer = vec![0u8; BUFFER_SIZE];

            let start = Instant::now();
            for _ in 0..iters {
                stream.write_all(&payload).unwrap();
                stream.read_exact(&mut buffer).unwrap();
                black_box(&buffer);
            }
            let duration = start.elapsed();

            let _ = stream.shutdown(Shutdown::Both);
            duration
        })
    });
    group.finish();
}

criterion_group!(benches, tcp_benchmark);
