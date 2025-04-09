# Benchmark results

## SQLite on EFS one zone

- ap-northeast-1 Tokyo region
- SUT : AWS Lambda Arm64, 1792MB, AmazonLinux 2023 runtime
- RTE : Scale factor=1

concurrency=1

```
220.0 tpm  ( 220 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           252,  0.112680,  0.016763,  0.032799,  0.047440
payment:             252,  0.073684,  0.016223,  0.005195,  0.036628
order_status:         23,  0.044334,  0.000036,  0.026400,  0.002901
delivery:             23,  0.259329,  0.017222,  0.089148,  0.138034
stock_level:         220,  0.046042,  0.000037,  0.028021,  0.002990
customer_by_name:    163,  0.036475,  0.000038,  0.017906,  0.003150
```

concurrency=2

```
90.0 tpm  ( 90 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           110,  0.411410,  0.157407,  0.156791,  0.077164
payment:             110,  0.351931,  0.177957,  0.028139,  0.130443
order_status:         10,  0.105034,  0.000037,  0.086015,  0.003274
delivery:             10,  1.215236,  0.271560,  0.789010,  0.138446
stock_level:         100,  0.377463,  0.000038,  0.358737,  0.003403
customer_by_name:     74,  0.109154,  0.000038,  0.088428,  0.004883
```

## AWS RDS PostgreSQL

- ap-northeast-1 Tokyo region
- SUT : AWS Lambda Arm64, 1792MB, AmazonLinux 2023 runtime
- RTE : Scale factor=1
- AWS RDS PostgreSQL : t4g.micro(2vCPU, 1GiB RAM), EBS gp3 20GiB, PostgreSQL 17.2

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
334.0 tpm  ( 334 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           374,  0.099847,  0.002879,  0.080496,  0.002624
payment:             373,  0.034310,  0.003400,  0.018037,  0.002022
order_status:         34,  0.037226,  0.002556,  0.021379,  0.001258
delivery:             34,  0.155734,  0.002674,  0.139848,  0.002377
stock_level:         330,  0.023810,  0.002628,  0.009498,  0.001168
customer_by_name:    262,  0.020916,  0.002680,  0.005989,  0.001241
```

concurrency=2

```
608.0 tpm  ( 608 new_order transactions in 60.000 secs )
##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:           721,  0.103800,  0.003303,  0.087593,  0.002536
payment:             720,  0.039356,  0.003270,  0.023705,  0.002392
order_status:         65,  0.022078,  0.003169,  0.007387,  0.001502
delivery:             65,  0.198146,  0.003166,  0.182605,  0.002382
stock_level:         650,  0.022580,  0.003353,  0.007599,  0.001622
customer_by_name:    451,  0.017513,  0.003288,  0.002536,  0.001571
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

## SQLite on AWS EC2 EBS

- us-west-2 Oregon region
- SUT : AWS EC2 t4g.micro, EBS gp3 8GiB, AmazonLinux 2023 runtime
- RTE : Scale factor=1

concurrency=1

```
3545.0 tpm  ( 3545 new_order transactions in 60.000 secs )

##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:          4120,  0.007941,  0.000056,  0.001834,  0.005762
payment:            4120,  0.005753,  0.000055,  0.000610,  0.004815
order_status:        375,  0.000695,  0.000031,  0.000406,  0.000037
delivery:            374,  0.015478,  0.000056,  0.005054,  0.010086
stock_level:        3740,  0.001154,  0.000028,  0.000883,  0.000047
customer_by_name:   2715,  0.001173,  0.000033,  0.000831,  0.000051
```

concurrency=2

```
3473.0 tpm  ( 3473 new_order transactions in 60.000 secs )

##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:          4070,  0.014928,  0.006731,  0.002042,  0.005864
payment:            4070,  0.010921,  0.005055,  0.000675,  0.004912
order_status:        370,  0.000782,  0.000031,  0.000477,  0.000045
delivery:            370,  0.012089,  0.001429,  0.002528,  0.007852
stock_level:        3690,  0.006913,  0.000044,  0.006576,  0.000076
customer_by_name:   2676,  0.001689,  0.000037,  0.001314,  0.000063
```

concurrency=3

```
3462.0 tpm  ( 3462 new_order transactions in 60.000 secs )

##                calls , e2e total,  begin   ,  query   ,  commit
##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)
new_order:          4038,  0.021110,  0.012825,  0.002110,  0.005887
payment:            4037,  0.019462,  0.013466,  0.000696,  0.005028
order_status:        368,  0.002429,  0.000031,  0.002108,  0.000060
delivery:            366,  0.014725,  0.004032,  0.002549,  0.007858
stock_level:        3660,  0.009685,  0.000043,  0.009342,  0.000081
customer_by_name:   2660,  0.001698,  0.000034,  0.001334,  0.000065
```

