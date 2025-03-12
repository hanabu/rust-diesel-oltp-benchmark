[![en](https://img.shields.io/badge/lang-en-blue.svg)](README.md)
[![ja](https://img.shields.io/badge/lang-ja-green.svg)](README.ja.md)

# TPC-C like database benchmark written in Rust Diesel ORM

[TPC-C](https://www.tpc.org/tpcc/) OLTP benchmark を Rust の [Diesel ORM](https://diesel.rs/) で実装したもの。

## 動機

[AWS EFS](https://aws.amazon.com/efs/) に [SQLite](https://www.sqlite.org/) database を載せたときにどのぐらい遅いか測ってみたい。妥当な database benchmark を設計するだけの経験を持っていないため、先人の知恵の詰まっているであろう TPCの標準 benchmark 仕様で計ってみた。

## 使い方

Benchmark は HTTP server となる測定対象部 (SUT, System Under Test) と、HTTP client となる模擬 client (Remote Terminal Emulator) からなる。先に SUT を起動しておいた状態で、RTE から連続して request 行って測定する。

```console
$ cd diesel-tpc-c/sut

(Run SQLite backend)
$ cargo run --release

(Run PostgreSQL backend)
$ export DATABASE_URL=postgres://user:password@db_host/db_name
$ cargo run --release --no-default-features --features=postgres
```

上記のように SUT を起動しておいた状態で、RTE から benchmark を実行。

- `-s` : Scale factor (倉庫の数)
- `-c` : 同時に接続する接続数
- `-d` : 測定時間(秒)

```console
$ cd diesel-tpc-c/rte

(Prepare database)
$ cargo run -- prepare -s 1 http://localhost:3000

(Run benchmark)
$ cargo run -- run -c 1 -d 30 http://localhost:3000
...
[INFO  rte] Start benchmark
[INFO  rte] Finished
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

上記の結果例では、`2403.0 tpm` が benchmark の測定値になる。  
TPC-C 標準では、new\_order, payment, order\_status, delivery, stock\_level の 5つの transaction を一定の割合で呼び出したときの 1分あたりの new_order 実行数を測定指標としている。

## TPC-C 標準への準拠

なるべく TPC-C 5.11 の仕様に合わせて実装しているが、以下の点は標準に従っていない。

- 仕様では、RTE は key 入力待ち時間を模擬することになっているが、本実装では待たずに連続して request を行っている
- 仕様では、RTE の画面表示も規定されているが、画面表示は実装していない (画面表示に必要な項目は、SUT からの応答として JSON 形式で返してはいる)
- Scale factor>1 で複数の倉庫を扱う場合に、他の倉庫 (remote warehouse) の在庫引き当てをする処理が規定されているが、本実装では正しく実装していない
- 他にも非準拠はあるかも

----

## SQLite on EFS

SSD を載せた PC 上で走らせること比較して、network file system である EFS で走らせた場合、1/10 ～ 1/20 程度の性能。比較すると遅いが、使えなくはないレベル感。

1並列: Benchmark 実行 70秒間で 524 transactions, EFS burst credit を 105MB 消費。SQLite database の file size は 約90MB。

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

2並列: Benchmark 実行 70秒間で 146 transactions, EFS burst credit を 143MB 消費。SQLite database の file size は 約90MB。  
同時に並行して request すると lock 競合して遅くなるうえに、cache が効かないため EFS の IO 量も増加。

```
39.0 tpm  ( 39 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:            49,  0.895723,  0.193402,  0.238216,  0.425100
payment:              48,  0.716544,  0.193371,  0.039067,  0.470254
order_status:          5,  0.119379,  0.000038,  0.101246,  0.005766
delivery:              4,  1.225100,  0.233055,  0.294449,  0.683861
stock_level:          40,  1.338246,  0.000037,  1.318307,  0.006060
customer_by_name:     27,  0.210721,  0.000040,  0.187270,  0.009398
```

条件は以下:

- AWS ap-northeast-1 Tokyo region
- SUTの実行環境 : AWS Lambda Arm64, 1792MB, AmazonLinux 2023 runtime
- RTEの設定 : Scale factor=1
- EFS: burst throughput mode
- SQLite-3.48.0, Diesel-2.2.8, Rust-1.85.0

### EFS で SQLite を使う場合の注意

- [WAL mode](https://www.sqlite.org/wal.html) は使用できない。WAL は mmap による共有 memory を必要とするが、NFS では利用できないため。WAL mode で使うと SQLite file が壊れるため `PRAGMA journal_mode = DELETE;` で利用する。
- [cache size](https://www.sqlite.org/pragma.html#pragma_cache_size) は大きくしたほうが良い。上記の TPC-C benchmark では cache size 2MB -> 32MiB で 約40tpm -> 約120tpm へ改善。
- 複数の query を `BEGIN TRANSACTION; ... COMMIT;` にまとめるとよい。File lock が transaction 単位にまとめて 1回 で済む。EFS は network 経由で file lock のため遅延が大きく、file lock 回数削減は性能に影響が出やすい。
