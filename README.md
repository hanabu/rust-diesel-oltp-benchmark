[![en](https://img.shields.io/badge/lang-en-blue.svg)](README.md)
[![ja](https://img.shields.io/badge/lang-ja-green.svg)](README.ja.md)

(This English README is machine translated from Japanese.)

# TPC-C like database benchmark written in Rust Diesel ORM

[TPC-C](https://www.tpc.org/tpcc/) OLTP benchmark implemented in Rust's [Diesel ORM](https://diesel.rs/).

##  Motivation

 I want to measure how slow [SQLite](https://www.sqlite.org/) database is on [AWS EFS](https://aws.amazon.com/efs/). Since I don't have enough experience to design a reasonable database benchmark, I used TPC's standard benchmark specification, which I believe contains the wisdom of our ancestors.

##  How to use

 Benchmark consists of an HTTP server (SUT, System Under Test) and a simulated HTTP client (Remote Terminal Emulator). With the SUT running first, perform the measurement by making a series of requests from the RTE.

``` console
 $ cd diesel-tpc-c/sut

(Run SQLite backend)
$ cargo run --release

(Run PostgreSQL backend)
$ export DATABASE_URL=postgres://user:password@db_host/db_name
$ cargo run --release --no-default-features --features=postgres
```

 With the SUT running as described above, run benchmark from the RTE.

- `-s`: Scale factor (number of warehouses)
- `-c`: Number of simultaneous connections
- `-d`: Measurement time (seconds)

``` console
 $ cd diesel-tpc-c/rte

(Prepare database)
$ cargo run -- prepare -s 1 http://localhost:3000

(Run benchmark)
$ cargo run -- run -c 1 -d 30 http://localhost:3000
...
[INFO rte] Start benchmark
[INFO rte] Finished
2403.0 tpm  ( 2403 new_order transactions in 60.000 secs )

##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:          2817,  0.011591,  0.000069,  0.001447,  0.009043
payment:            2816,  0.009881,  0.000063,  0.000538,  0.008327
order_status:        256,  0.002263,  0.000043,  0.001324,  0.000070
delivery:            256,  0.012014,  0.000069,  0.001776,  0.009159
stock_level:        2560,  0.001402,  0.000040,  0.000624,  0.000071
customer_by_name:   1828,  0.001150,  0.000048,  0.000160,  0.000069
```

Command example above shows `2403.0 tpm` as the benchmark result indicator. The TPC-C standard measures the number of new\_order executions per minute when five transactions (new\_order, payment, order\_status, delivery, and stock\_level) are called at a certain rate. \
 The number of new\_order executions per minute is used as an indicator.

##  Compliance with TPC-C standards

 Although the implementation conforms to the TPC-C 5.11 specification as much as possible, the following points do not conform to the standard.

-  According to the specification, the RTE is supposed to simulate the key input waiting time, but in this implementation, the RTE does not wait and makes a request continuously.
-  The specification also stipulates screen display of the RTE, but screen display is not implemented (items required for screen display are returned in JSON format as a response from the SUT).
-  When multiple warehouses are handled with a scale factor\>1, the specification specifies a process to allocate inventory to other warehouses (remote warehouses), but this implementation does not correctly implement it.
-  There may be other non-compliances.

-----

##  SQLite on EFS

 Compared to running on a PC with SSD, the performance is about 1/10 to 1/20 when running on EFS, a network file system. It is slow in comparison, but not unusable.

```
149.0 tpm  ( 149 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           172,  0.179153,  0.025044,  0.055892,  0.080491
payment:             171,  0.107907,  0.024298,  0.008262,  0.060720
order_status:         16,  0.080568,  0.000043,  0.062220,  0.004832
delivery:             15,  0.186698,  0.025183,  0.017058,  0.129534
stock_level:         150,  0.071711,  0.000042,  0.052575,  0.004807
customer_by_name:    108,  0.055207,  0.000041,  0.034713,  0.005373
```

Benchmark run: 524 transactions in 70 seconds, EFS burst credit consumed 105MB, SQLite database file size is about 90MB.

Conditions are as follows:

-  AWS ap-northeast-1 Tokyo region
-  SUT execution environment : AWS Lambda Arm64, 1792MB, AmazonLinux 2023 runtime
-  RTE configuration : Scale factor=1, 1 concurrent
-  EFS: burst throughput mode
-  SQLite-3.48.0, Diesel-2.2.8, Rust-1.85.0

###  Notes on using SQLite with EFS

- [WAL mode](https://www.sqlite.org/wal.html) cannot be used, because WAL requires shared memory by mmap, which is not available in NFS. Instead of WAL, use `PRAGMA journal_mode = DELETE;`
-  It is better to use a large [cache size](https://www.sqlite.org/pragma.html#pragma_cache_size). In the above TPC-C benchmark, cache size 2MB -\> 32MiB, about 40tpm -\> about 120tpm.
-  It is recommended to combine multiple queries into `BEGIN TRANSACTION; ... COMMIT;`. File lock is be done only once for each transaction. EFS uses file lock via network, so the delay is large and reducing the number of file locks will easily affect performance.
