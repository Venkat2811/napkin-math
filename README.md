# Napkin Math

The goal of this project is to collect software, numbers, and techniques to
quickly estimate the expected performance of systems from first-principles. For
example, how quickly can you read 1 GB of memory? By composing these resources
you should be able to answer interesting questions like: how much storage cost
should you expect to pay for logging for an application with 100,000 RPS?

The best introduction to this skill is [through my talk at
SRECON](https://www.youtube.com/watch?v=IxkSlnrRFqc).

The best way to practise napkin math in the grand domain of computers is to work
on your own problems. The second-best is to **subscribe to [this
newsletter](http://sirupsen.com/napkin) where you'll get a problem every few
weeks to practise on**. It should only take you a few minutes to solve each one as your
facility with these techniques improve.

The archive of problems to practise with are
[here](https://sirupsen.com/napkin/). The solution will be in the following
newsletter.

## Numbers

Below are numbers rounded for memorization, not faux precision.
The rows this repo can currently refresh on a single host were re-measured and
revalidated on fresh GCP `c4-standard-48-lssd` instances on March 8, 2026
(Intel Xeon 6985P-C, 48 vCPU / 24 physical cores, 180 GB RAM, Ubuntu 22.04.5
LTS).

[9]: https://gist.github.com/sirupsen/766f266eebf6bdf2525bdbb309e17a41

**Note 1:** Some throughput and latency numbers don't line up, this is
intentional for ease of calculations.

**Note 2:** Take the numbers with a grain of salt. E.g. for I/O, [`fio`][fio] is
the state-of-the-art. I am continuously updating these numbers as I learn more
to improve accuracy and as hardware improves.

| Operation                           | Latency     | Throughput | 1 MiB  | 1 GiB  |
| ----------------------------------- | -------     | ---------- | ------ | ------ |
| Sequential Memory R/W (64 bytes)    | 0.5 ns      |            |        |        |
| ├ Single Thread                     |             | 20 GiB/s   | 50 μs  | 50 ms  |
| ├ Threaded                          |             | 200 GiB/s  | 5 μs   | 5 ms   |
| Network Same-Zone                   |             | 10 GiB/s   | 100 μs | 100 ms |
| ├ Inside VPC                        |             | 10 GiB/s   | 100 μs | 100 ms |
| ├ Outside VPC                       |             | 3 GiB/s    | 300 μs | 300 ms |
| Hashing, not crypto-safe (64 bytes) | 10 ns       | 5 GiB/s    | 200 μs | 200 ms |
| Random Memory R/W (64 bytes)        | 20 ns       | 3 GiB/s    | 300 μs | 300 ms |
| Fast Serialization `[8]` `[9]` †    | N/A         | 1 GiB/s    | 1 ms   | 1s     |
| Fast Deserialization `[8]` `[9]` †  | N/A         | 1 GiB/s    | 1 ms   | 1s     |
| System Call                         | 300 ns      | N/A        | N/A    | N/A    |
| Hashing, crypto-safe (64 bytes)     | 100 ns      | 1 GiB/s    | 1 ms   | 1s     |
| Sequential SSD read (8 KiB)         | 1 μs        | 8 GiB/s    | 100 μs | 100 ms |
| Context Switch `[1] [2]`            | 10 μs       | N/A        | N/A    | N/A    |
| Sequential SSD write, -fsync (8KiB) | 2 μs        | 3 GiB/s    | 300 μs | 300 ms |
| TCP Echo Server (32 KiB)            | 50 μs       | 500 MiB/s  | 2 ms   | 2s     |
| Random SSD Read (8 KiB)             | 100 μs      | 70 MiB/s   | 15 ms  | 15s    |
| Decompression `[11]`                | N/A         | 1 GiB/s    | 1 ms   | 1s     |
| Compression `[11]`                  | N/A         | 500 MiB/s  | 2 ms   | 2s     |
| Sorting (64-bit integers)           | N/A         | 500 MiB/s  | 2 ms   | 2s     |
| Proxy: Envoy/ProxySQL/Nginx/HAProxy | 50 μs       | ?          | ?      | ?      |
| Network within same region          | 250 μs      | 2 GiB/s    | 500 μs | 500 ms |
| Premium network within zone/VPC     | 250 μs      | 25 GiB/s   | 50 μs  | 40 ms  |
| Sequential SSD write, +fsync (8KiB) | 300 μs      | 30 MiB/s   | 30 ms  | 30s    |
| {MySQL, Memcached, Redis, ..} Query | 500 μs      | ?          | ?      | ?      |
| Serialization `[8]` `[9]` †         | N/A         | 100 MiB/s  | 10 ms  | 10s    |
| Deserialization `[8]` `[9]` †       | N/A         | 100 MiB/s  | 10 ms  | 10s    |
| Sequential HDD Read (8 KiB)         | 10 ms       | 250 MiB/s  | 2 ms   | 2s     |
| Random HDD Read (8 KiB)             | 10 ms       | 0.7 MiB/s  | 2 s    | 30m    |
| Blob Storage GET, if-not-match 304  | 30 ms       |            |        |        |
| Blob Storage GET, 1 conn (128KiB)   | 80 ms       | 100 MiB/s  | 10 ms  | 10s    |
| Blob Storage GET, n conn (offsets)  | 80 ms       | NW limit   |        |        |
| Blob Storage LIST                   | 100 ms      |            |        |        |
| Blob Storage PUT, 1 conn (128KiB)   | 200 ms      | 100 MiB/s  | 10 ms  | 10s    |
| Blob Storage PUT, n conn (multipart)| 200 ms      | NW limit   | 10 ms  | 10s    |
| Network between regions `[6]`       | [Varies][i] | 25 MiB/s   | 40 ms  | 40s    |
| Network NA Central <-> East         | 25 ms       | 25 MiB/s   | 40 ms  | 40s    |
| Network NA Central <-> West         | 40 ms       | 25 MiB/s   | 40 ms  | 40s    |
| Network NA East <-> West            | 60 ms       | 25 MiB/s   | 40 ms  | 40s    |
| Network EU West <-> NA East         | 80 ms       | 25 MiB/s   | 40 ms  | 40s    |
| Network EU West <-> NA Central      | 100 ms      | 25 MiB/s   | 40 ms  | 40s    |
| Network NA West <-> Singapore       | 180 ms      | 25 MiB/s   | 40 ms  | 40s    |
| Network EU West <-> Singapore       | 160 ms      | 25 MiB/s   | 40 ms  | 40s    |

[i]: https://www.cloudping.co/ 

**†:** "Fast serialization/deserialization" is typically a simple wire-protocol
that just dumps bytes, or a very efficient environment. Typically standard
serialization such as e.g. JSON will be of the slower kind. We include both here
as serialization/deserialization is a very, very broad topic with extremely
different performance characteristics depending on data and implementation.

For the active Criterion suite, run `./script/bench-criterion`. It keeps the
safe part of the old `./run` wrapper: `--release` plus
`RUSTFLAGS='-C target-cpu=native'`, without assuming `sudo`, Intel-specific
sysfs knobs, or hyperthreading control. The older `./run` wrapper is still
available when you want aggressive host tuning on a dedicated Linux box.

**Note:** The active benchmark path today is Criterion.rs in `benches/`, and
the checked-in Criterion entrypoint currently owns only `memory_read` and
`compressed_memory_read`. `src/main.rs` is still the older ad hoc harness and
remains the source of truth for `memory_random`, `hash`, `syscall`, `sort`,
`disk`, `tcp`, `redis`, `mysql`, and `mutex` style probes that have not been
fully migrated and revalidated yet. Use `./script/bench-legacy` for the local
legacy benches, `./script/bench-redis` for the Redis probe, and
`./script/bench-mysql` for the MySQL write probe. The root
[`bench_status.json`](bench_status.json) file records which README rows are
Criterion-backed, legacy-harness-backed, external reference text, or currently
stale.

The `compressed_memory_read` Criterion bench is a BitPacker integer-unpack
microbenchmark; it should not be used to rewrite the generic `[11]`
compression/decompression rows above.

The object-storage rows above are currently reference text only in this
checkout. Earlier README notes referred to a `blob_storage` Criterion group plus
helpers such as `src/bin/s3_latency.rs` and `./script/blob-*`, but those files
are not present in the checked-in tree today. Treat the blob-storage numbers as
curated heuristics until benchmark code is restored and wired back into the
repo.

`memory_read` now emits explicit `No SIMD` and `SIMD` variants in Criterion,
but the README intentionally collapses them to one single-thread row and one
threaded row for memorability.

### Host-Local Communication

The main table above intentionally emphasizes broad machine constants.
Host-local communication is trickier: one generic `IPC latency` row hides too
much. Inter-thread and inter-process numbers move materially with topology
(`1p1c` vs `1pNc`), pacing (max throughput vs fixed rate), batching, payload
size, and backend (`shm` vs `mmap`).

For now, treat the table below as a companion reference range rather than a
single memorized constant. The active benchmark path in this repo does not yet
own this surface. These rows are distilled from dedicated transport benchmark
suites in [`disruptor-rs`](https://github.com/Venkat2811/disruptor-rs/tree/7b32d11)
for inter-thread rings and
[`myelon`](https://github.com/Venkat2811/myelon/tree/35d68fb) for
inter-process SHM/mmap rings on modern x86 hosts.

| Operation                          | Shape                            | Heuristic                         | Notes |
| ---------------------------------- | -------------------------------- | --------------------------------- | ----- |
| Inter-thread lock-free handoff     | SPSC, burst = `1`                | `7-17 ns`, `60-150M msgs/s`       | One producer, one consumer, busy-spin style ring / bounded channel regime. |
| Inter-thread batched handoff       | SPSC, bursts of `10-100`         | `2.6-3.2 ns / msg`, `315-380M msgs/s` | Batching changes the answer enough that a single queue-latency number is misleading. |
| Inter-thread multi-producer handoff | MPSC aggregate                  | `~340M msgs/s` ring, `~75M msgs/s` bounded channel | Aggregate throughput across two producers; use only for CPU-burning same-host paths. |
| Inter-process signal               | `1p1c`, `64B` event, no ack      | `~160-330M ops/s` total signal ceiling | Not RTT; producer and consumer are both doing work every event. |
| Inter-process ping-pong            | `1p1c` SHM, `64B`                | `~120-190 ns p50`, `~5-6M RTT/s`  | Request/response RTT on one host with busy-spin waiting. |
| Inter-process broadcast            | `1p4c` mmap, `1 KiB`             | `~9M msgs/s` producer, `~9M msgs/s` per consumer | Every consumer sees every message; aggregate delivered bandwidth scales with fan-out. |
| Inter-process payload bandwidth    | `1p4c` mmap, `128 KiB`           | `~14 GiB/s` producer-side publish bandwidth | Large-payload same-host fan-out regime; do not reuse for small-message latency budgeting. |

If you need one quick mental checksum: inter-thread rings are usually in the
single-digit-to-tens-of-nanoseconds regime, inter-process SHM ping-pong is
usually in the low-hundreds-of-nanoseconds regime, and broadcast / fan-out
should be modeled separately from RTT.

I am aware of some inefficiencies in this suite. I intend to improve my skills
in this area, in order to ensure the numbers are the upper-bound of performance
you may be able to squeeze out in production. I find it highly unlikely any of
them will be more than 2-3x off, which shouldn't be a problem for most users.

## Cost Numbers

Approximate numbers that should be consistent between Cloud providers.

| What                | Amount | \$ / Month | 1y commit \$ /month | Spot \$ /month | Hourly Spot \$ |
| --------------------| ------ | ---------  | ------------------ | ------------- | ------------- |
| CPU                 | 1      | \$15       | \$10                | \$2            |  \$0.005       |
| GPU                 | 1      | \$5000     | \$3000              | \$1500         |  \$2           |
| Memory              | 1 GB   | \$2        | \$1                 | \$0.2          |  \$0.0005      |
| Storage             |        |            |                    |               |               |
| ├ Warehouse Storage | 1 GB   | \$0.02     |                    |               |               |
| ├ Blob (S3, GCS)    | 1 GB   | \$0.02     |                    |               |               |
| ├ Zonal HDD         | 1 GB   | \$0.05     |                    |               |               |
| ├ Ephemeral SSD     | 1 GB   | \$0.08     | \$0.05             | \$0.05        |  \$0.07        |
| ├ Regional HDD      | 1 GB   | \$0.1      |                    |               |               |
| ├ Zonal SSD         | 1 GB   | \$0.2      |                    |               |               |
| ├ Regional SSD      | 1 GB   | \$0.35     |                    |               |               |
| Networking          |        |            |                    |               |               |
| ├ Same Zone         | 1 GB   | \$0        |                    |               |               |
| ├ Blob              | 1 GB   | \$0        |                    |               |               |
| ├ Ingress           | 1 GB   | \$0        |                    |               |               |
| ├ L4 LB             | 1 GB   | \$0.008    |                    |               |               |
| ├ Inter-Zone        | 1 GB   | \$0.01     |                    |               |               |
| ├ Inter-Region      | 1 GB   | \$0.02     |                    |               |               |
| ├ Internet Egress † | 1 GB   | \$0.1      |                    |               |               |
| CDN Egress          | 1 GB   | \$0.05     |                    |               |               |
| CDN Fill ‡          | 1 GB   | \$0.01     |                    |               |               |
| Warehouse Query     | 1 GB   | \$0.005    |                    |               |               |
| Logs/Traces    ♣    | 1 GB   | \$0.5      |                    |               |               |
| Metrics             | 1000   | \$20       |                    |               |               |
| EKM Keys            | 1      | \$1        |                    |               |               |

† This refers to network leaving your cloud provider, e.g. sending data to S3
from GCP or egress network for sending HTML from AWS to a client.

‡ An additional per cache-fill fee is incurred that costs close to blob storage
write costs (see just below).

7 This is standard pricing among a few logging providers, but e.g. [Datadog
pricing](https://www.datadoghq.com/pricing/?product=log-management#products) is
different and charges \$0.1 per ingested logs with \$1.5 per 1m on top for 7d
retention.

Furthermore, for blob storage (S3/GCS/R2/...), you're charged per read/write
operation (fewer, large files is cheaper):

 |                | 1M      | 1000     |
 |----------------|---------|----------|
 | Reads          | \$0.4   | \$0.0004 |
 | Writes         | \$5     | \$0.005  |
 | EKM Encryption | \$3     | \$0.003  |

## Compression Ratios

This is sourced from a few sources. `[3]` `[4]` `[5]` Note that compression speeds (but
generally not ratios) vary by an order of magnitude depending on the algorithm
and the level of compression (which trades speed for compression).

I typically ballpark that another _x in compression ratio decreases performance
by 10x_. E.g. we can [get a 2x ratio on English
Wikipedia](https://quixdb.github.io/squash-benchmark/#results-table) at ~200
MiB/s, and 3x at ~20MiB/s, and 4x at 1MB/s.

| What        | Compression Ratio |
| ----------- | ----------------- |
| HTML        | 2-3x              |
| English     | 2-4x              |
| Source Code | 2-4x              |
| Executables | 2-3x              |
| RPC         | 5-10x             |
| SSL         | -2% `[10]`        |

## Techniques

* **Don't overcomplicate.** If you are basing your calculation on more than 6
    assumptions, you're likely making it harder than it should be.
* **Keep the units.** They're good checksumming.
    [Wolframalpha](https://wolframalpha.com) has terrific support if you need a
    hand in converting e.g. KiB to TiB.
* **Calculate with exponents.** A lot of back-of-the-envelope calculations are
    done with just coefficients and exponents, e.g. `c * 10^e`. Your goal is to
    get within an order of magnitude right--that's just `e`. `c` matters a lot
    less. Only worrying about single-digit coefficients and exponents makes it
    much easier on a napkin (not to speak of all the zeros you avoid writing).
* **Perform Fermi decomposition.** Write down things you can guess at until you
    can start to hint at an answer. When you want to know the cost of storage
    for logging, you're going to want to know how big a log line is, how many of
    those you have per second, what that costs, and so on.

## Resources

* `[1]`: https://eli.thegreenplace.net/2018/measuring-context-switching-and-memory-overheads-for-linux-threads/
* `[2]`: https://blog.tsunanet.net/2010/11/how-long-does-it-take-to-make-context.html
* `[3]`: https://cran.r-project.org/web/packages/brotli/vignettes/brotli-2015-09-22.pdf
* `[4]`: https://github.com/google/snappy
* `[5]`: https://quixdb.github.io/squash-benchmark/
* `[6]`: https://dl.acm.org/doi/10.1145/1879141.1879143
* `[7]`: https://en.wikipedia.org/wiki/Hard_disk_drive_performance_characteristics#Seek_times_&_characteristics
* `[8]`: https://github.com/simdjson/simdjson#performance-results
* `[9]`: https://github.com/protocolbuffers/protobuf/blob/d20e9a92/docs/performance.md
* `[10]`: https://www.imperialviolet.org/2010/06/25/overclocking-ssl.html
* `[11]`: https://github.com/inikep/lzbench
* ["How to get consistent results when benchmarking on
  Linux?"](https://easyperf.net/blog/2019/08/02/Perf-measurement-environment-on-Linux#2-disable-hyper-threading).
  Great compilation of various Kernel and CPU features to toggle for reliable
  bench-marking, e.g. CPU affinity, disabling turbo boost, etc. It also has
  resources on proper statistical methods for benchmarking.
* [LLVM benchmarking tips](https://www.llvm.org/docs/Benchmarking.html). Similar
  to the above in terms of dedicating CPUs, disabling address space
  randomization, etc.
* [Top-Down performance analysis
  methodology](https://easyperf.net/blog/2019/02/09/Top-Down-performance-analysis-methodology).
  Useful post about using `toplev` to find the bottlenecks. This is particularly
  useful for the benchmarking suite we have here, to ensure the programs are
  correctly written (I have not taken them through this yet, but plan to).
* [Godbolt's compiler explorer](https://gcc.godbolt.org/#). Fantastic resource
  for comparing assembly between Rust and e.g. C with Clang/GCC.
* [cargo-show-asm](https://github.com/pacak/cargo-show-asm). Cargo extension to allow
  disassembling functions. Unfortunately the support for closure is a bit
  lacking, which requires some refactoring.
* [Agner's Assembly
  Guide](https://www.agner.org/optimize/optimizing_assembly.pdf). An excellent
  resource on writing optimum assembly, which will be useful to inspect the
  various functions for inefficiencies in our suite.
* [Agner's Instruction
  Tables](https://www.agner.org/optimize/instruction_tables.pdf). Thorough
  resource on the expected throughput for various instructions which is helpful
  to inspect the assembly.
* [halobates.de](http://halobates.de/). Useful resource for low-level
  performance by the author of `toplev`.
* [Systems Performance (book)](https://www.amazon.com/Systems-Performance-Enterprise-Brendan-Gregg/dp/0133390098/ref=sr_1_1?keywords=systems+performance&qid=1580733419&sr=8-1). Fantastic book about analyzing system performance, finding bottlenecks, and understanding operating systems.
* [io_uring](https://lwn.net/Articles/776703/). Best summary, it links to many
  resources.
* [How Long Does It Takes To Make a Context Switch](https://blog.tsunanet.net/2010/11/how-long-does-it-take-to-make-context.html)
* [Integer Compression Comparisons](https://github.com/powturbo/TurboPFor-Integer-Compression)
* [Files are hard](https://danluu.com/file-consistency/)
* [`disruptor-rs`](https://github.com/Venkat2811/disruptor-rs). Dedicated
  inter-thread ring benchmark surface used for the SPSC / MPSC host-local
  communication reference rows above.
* [`myelon`](https://github.com/Venkat2811/myelon). Dedicated inter-process
  SHM / mmap transport benchmark surface used for the signal / ping-pong /
  broadcast reference rows above.

[fio]: https://github.com/axboe/fio
