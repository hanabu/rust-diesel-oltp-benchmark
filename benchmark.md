# Benchmark results

## AWS RDS PostgreSQL

- ap-northeast-1 Tokyo region
- SUT : AWS Lambda Arm64, 1792MB, AmazonLinux 2023 runtime
- RTE : Scale factor=1
- AWS RDS PostgreSQL : t4g.micro, EBS gp3 20GiB, PostgreSQL 17.2

concurrency=1

```
546.0 tpm  ( 546 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           632,  0.051235,  0.001667,  0.035959,  0.001737
payment:             631,  0.024300,  0.001525,  0.009458,  0.001518
order_status:         58,  0.016820,  0.001396,  0.002477,  0.000600
delivery:             57,  0.084813,  0.001476,  0.069637,  0.001707
stock_level:         570,  0.018030,  0.001578,  0.004044,  0.000682
customer_by_name:    417,  0.014973,  0.001599,  0.000980,  0.000608
```

concurrency=2

```
821.0 tpm  ( 821 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           994,  0.055478,  0.001648,  0.039799,  0.001641
payment:             994,  0.026466,  0.001608,  0.010865,  0.001524
order_status:         91,  0.018530,  0.002113,  0.002832,  0.000678
delivery:             90,  0.341902,  0.001602,  0.326087,  0.001679
stock_level:         890,  0.018272,  0.001531,  0.004085,  0.000761
customer_by_name:    639,  0.015587,  0.001555,  0.001032,  0.000699
```

## neon.tech 0.25 CU

- us-west-2 Oregon region
- SUT : AWS Lambda Arm64, 1792MB, AmazonLinux 2023 runtime
- RTE : Scale factor=1
- PostgreSQL : neon.tech 0.25 CU, PostgreSQL 16

concurrency=1

```
413.0 tpm  ( 413 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           470,  0.076005,  0.003440,  0.060015,  0.002034
payment:             470,  0.029552,  0.002326,  0.014776,  0.001919
order_status:         43,  0.025726,  0.002287,  0.011419,  0.001019
delivery:             42,  0.126120,  0.002491,  0.111276,  0.002052
stock_level:         420,  0.020154,  0.002314,  0.006392,  0.001016
customer_by_name:    309,  0.017979,  0.002418,  0.003916,  0.001117
```

concurrency=2

```
554.0 tpm  ( 554 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           650,  0.115629,  0.003708,  0.098815,  0.002687
payment:             650,  0.043158,  0.003670,  0.026656,  0.002576
order_status:         60,  0.024390,  0.003873,  0.008565,  0.001764
delivery:             58,  0.226177,  0.003530,  0.209703,  0.002611
stock_level:         580,  0.024238,  0.003829,  0.008241,  0.001794
customer_by_name:    427,  0.019205,  0.003642,  0.003453,  0.001736
```

## CockroachDB serverless

- us-west-2 Oregon region
- SUT : AWS Lambda Arm64, 1792MB, AmazonLinux 2023 runtime
- RTE : Scale factor=1
- PostgreSQL : CockroachDB serverless AWS us-west-2

concurrency=1  
consumed RU = 140k

```
153.0 tpm  ( 153 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           173,  0.177054,  0.002817,  0.151157,  0.004905
payment:             173,  0.057259,  0.003097,  0.037676,  0.004856
order_status:         16,  0.028386,  0.002755,  0.012474,  0.001353
delivery:             16,  1.343715,  0.002691,  1.315904,  0.014290
stock_level:         150,  0.036606,  0.002871,  0.020644,  0.001592
customer_by_name:    116,  0.021458,  0.002821,  0.005241,  0.001429
```

concurrency=2  
consumed RU = 120k

```
148.0 tpm  ( 148 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           173,  0.178433,  0.002924,  0.155781,  0.004895
payment:             172,  0.057827,  0.002940,  0.038421,  0.004914
order_status:         16,  0.027044,  0.003120,  0.011152,  0.001489
delivery:             15,  1.395790,  0.002872,  1.366557,  0.014695
stock_level:         150,  0.037370,  0.003047,  0.021291,  0.001498
customer_by_name:    111,  0.021215,  0.003017,  0.005252,  0.001470
```
