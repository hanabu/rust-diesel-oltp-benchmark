[![en](https://img.shields.io/badge/lang-en-blue.svg)](README.md)
[![ja](https://img.shields.io/badge/lang-ja-green.svg)](README.ja.md)

# TPC-C like database benchmark written in Rust Diesel ORM

[TPC-C](https://www.tpc.org/tpcc/) OLTP benchmark を Rust の [Diesel ORM](https://diesel.rs/) で実装したもの。

## 動機

[AWS EFS](https://aws.amazon.com/efs/) に [SQLite](https://www.sqlite.org/) database を載せたときにどのぐらい遅いか測ってみたい。妥当な database benchmark を設計するだけの経験を持っていないため、先人の知恵の詰まっているであろう TPCの標準 benchmark 仕様で計ってみた。

## 使い方

HTTP server となる測定対象部 (SUT, System Under Test) と、HTTP client となる模擬 client (Remote Terminal Emulator) からなる。先に SUT を起動しておいた状態で、RTE から連続して request 行って測定する。

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
1426.0 tpm = 713 new_order transactions in 30.000 secs
new_order:          932 calls, 0.023s/call
payment:            931 calls, 0.008s/call
order_status:        85 calls, 0.003s/call
delivery:            84 calls, 0.042s/call
stock_level:        840 calls, 0.003s/call
customer_by_name:   584 calls, 0.002s/call
```

`1426.0 tpm` が benchmark の測定値になる。TPC-C 標準では、new_order, payment, order_status, delivery, stock_level の 5つの transaction を一定の割合で呼び出したときの 1分あたりの new_order 実行数を測定指標としている。

## TPC-C 標準への準拠

なるべく TPC-C 5.11 の仕様に合わせて実装しているが、以下の点は標準に従っていない。

- 仕様では、RTE は key 入力待ち時間を模擬することになっているが、本実装では待たずに連続して request を行っている
- 仕様では、RTE の画面表示も規定されているが、画面表示は実装していない (画面表示に必要な項目は、SUT の応答として JSON 形式で返してはいる)
- Scale factor>1 で複数の倉庫を扱う場合に、他の倉庫 (remote warehouse) の在庫引き当てをする処理が規定されているが、本実装では正しく実装していない
- 他にも非準拠はあるかも

----

## SQLite on EFS

PC の local SSD で SQLite を走らせることに比べて、network file system である EFS で走らせた場合、1/10 ～ 1/20 程度の性能。比較すると遅いが、使えなくはないレベル感。

```
126.0 tpm = 126 new_order transactions in 60.000 secs
new_order:          143 calls, 0.189s/call
payment:            143 calls, 0.118s/call
order_status:        13 calls, 0.065s/call
delivery:            13 calls, 0.437s/call
stock_level:        130 calls, 0.061s/call
customer_by_name:    85 calls, 0.045s/call
```

Benchmark 実行 70秒間で 517 requests, EFS burst credit を 102.3MB 消費。SQLite database の file size は 約90MB。

条件は以下

- AWS ap-northeast-1 Tokyo region
- SUTの実行環境 : AWS Lambda Arm64, 1792MB, 
- RTEの設定 : Scale factor=1, 1並列
- EFS: burst throughput mode
- SQLite-3.48.0, Diesel-2.2.8, Rust-1.85.0

### EFS で SQLite を使う場合の注意

- [WAL mode](https://www.sqlite.org/wal.html) は使用できない。WAL は mmap による共有 memory を必要とするが、NFS では利用できないため。WAL mode で使うと SQLite file が壊れるため `PRAGMA journal_mode = DELETE;` で利用する。
- [cache size](https://www.sqlite.org/pragma.html#pragma_cache_size) は大きくしたほうが良い。上記の TPC-C benchmark では cache size 2MB -> 32MiB で 約40tpm -> 約120tpm へ改善。