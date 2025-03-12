# Benchmark results

## neon.tech 0.25 CU

- us-west-2 Oregon region
- SUT : AWS Lambda Arm64, 1792MB, AmazonLinux 2023 runtime
- RTE : Scale factor=1
- PostgreSQL : neon.tech 0.25 CU

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
