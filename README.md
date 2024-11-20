# TPC-C like database benchmark written in Rust Diesel ORM

todo!()

## Test results (with transaction)

WSL2 , AmazonLinux 2023 on Windows Dev Kit 2023 (Snapdragon 8cx Gen3)

| Database      | Concurrency |  TPM  |
|:-------------:| -----------:| -----:|
| SQLite 3.44.0 |           1 |  2403 |
| SQLite 3.44.0 |           2 |  2717 |
| SQLite 3.44.0 |           4 |  2611 |
| SQLite 3.44.0 |           8 |  2554 |
| Postgres 15.4 |           1 |  1317 |
| Postgres 15.4 |           2 |   |
| Postgres 15.4 |           4 |   |
| Postgres 15.4 |           8 |   |


AWS Lambda (Arm64, 1792MB, ap-northeast-1) + EFS (elastic throughput)

| Database      | Concurrency |  TPM  |
|:-------------:| -----------:| -----:|
| SQLite 3.44.0 |           1 |    45 |

AWS Lambda (Arm64, 1792MB, ap-northeast-1) + EFS (burst throughput)

| Database      | Concurrency |  TPM  |
|:-------------:| -----------:| -----:|
| SQLite 3.44.0 |           1 |    41 |


## Test results (without transaction)

WSL2 , AmazonLinux 2023 on Windows Dev Kit 2023 (Snapdragon 8cx Gen3)

| Database      | Concurrency |  TPM  |
|:-------------:| -----------:| -----:|
| SQLite 3.44.0 |           1 |  1826 |
| SQLite 3.44.0 |           2 |  1994 |
| SQLite 3.44.0 |           4 |  2024 |
| SQLite 3.44.0 |           8 |  2014 |
| Postgres 15.4 |           1 |  1134 |
| Postgres 15.4 |           2 |  2970 |
| Postgres 15.4 |           4 |  4568 |
| Postgres 15.4 |           8 |  6570 |


AWS Lambda (Arm64, 1792MB, ap-northeast-1) + EFS (elastic throughput)

| Database      | Concurrency |  TPM  |
|:-------------:| -----------:| -----:|
| SQLite 3.44.0 |           1 |    34 |

