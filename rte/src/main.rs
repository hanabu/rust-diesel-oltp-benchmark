type Error = Box<(dyn std::error::Error + Send + Sync + 'static)>;

/// Run TPC-C like database benchmark
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Prepare initial database
    Prepare(PrepareArgs),
    /// Run benchmark
    Run(RunArgs),
}

#[derive(clap::Args, Debug)]
struct PrepareArgs {
    /// Scale factor (Warehouse count for TPC-C)
    #[arg(short, long, default_value = "1")]
    scale_factor: i32,
    /// Endpoint URL of SUT
    endpoint: String,
}

#[derive(clap::Args, Debug)]
struct RunArgs {
    /// Cuncurrency / scale_factor
    #[arg(short, long, default_value = "1")]
    concurrent: i32,
    /// Duration in secs
    #[arg(short, long, default_value = "60")]
    duration: f32,
    /// Wait : 0.0 for no wait, 1.0 for value in TPC-C spec.
    #[arg(short, long, default_value = "1.0")]
    wait: f32,
    /// Endpoint URL of SUT
    endpoint: String,
}

/// Endpoint URLs for each request
struct EndpointUrls {
    base: url::Url,
    new_order: url::Url,
    payment: url::Url,
    customer: url::Url,
    delivery: url::Url,
    prepare_db: url::Url,
}

impl TryFrom<&str> for EndpointUrls {
    type Error = url::ParseError;
    fn try_from(url_str: &str) -> Result<Self, Self::Error> {
        let base = url::Url::parse(url_str)?.join("/")?;

        Ok(Self {
            new_order: base.join("/orders")?,
            payment: base.join("/payment")?,
            customer: base.join("/customers")?,
            delivery: base.join("/delivery")?,
            prepare_db: base.join("/prepare_db")?,
            base,
        })
    }
}

impl EndpointUrls {
    pub fn new_order(&self) -> url::Url {
        self.new_order.clone()
    }
    pub fn prepare_db(&self) -> url::Url {
        self.prepare_db.clone()
    }
    pub fn payment(&self) -> url::Url {
        self.payment.clone()
    }
    pub fn order_status(&self, warehouse_id: i32, district_id: i32, customer_id: i32) -> url::Url {
        let path = format!(
            "/customers/{}/{}/{}/orders",
            warehouse_id, district_id, customer_id
        );
        let mut order_status = self.base.clone();
        order_status.set_path(&path);
        order_status
    }
    pub fn delivery(&self) -> url::Url {
        self.delivery.clone()
    }
    pub fn check_stocks(&self, warehouse_id: i32, district_id: i32) -> url::Url {
        let path = format!("/districts/{}/{}/check_stocks", warehouse_id, district_id);
        let mut check_stocks = self.base.clone();
        check_stocks.set_path(&path);
        check_stocks
    }
    pub fn customer_by_lastname(&self) -> url::Url {
        self.customer.clone()
    }
    /*
    pub fn status(&self) -> url::Url {
        self.base.clone()
    }
    */
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    use clap::Parser;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    match cli.command {
        Command::Prepare(args) => prepare(args).await?,
        Command::Run(args) => run(args).await?,
    }

    Ok(())
}

/// Prepare database
async fn prepare(args: PrepareArgs) -> Result<(), Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let endpoints = EndpointUrls::try_from(args.endpoint.as_str())?;
    let endpoint = endpoints.prepare_db();
    log::info!(
        "Requesting POST {} with scale_factor={}",
        endpoint.as_str(),
        args.scale_factor
    );

    let t = std::time::Instant::now();
    let resp = client
        .post(endpoint)
        .json(&if_types::PrepareDbRequest {
            scale_factor: args.scale_factor,
        })
        .send()
        .await?;

    let resp = resp.json::<if_types::DbStatusResponse>().await?;
    log::info!(
        "Preparation succeeded in {:.03}s",
        t.elapsed().as_secs_f32()
    );
    println!("  warehouse = {}", resp.warehouse_count);
    println!("  district  = {}", resp.district_count);
    println!("  customer  = {}", resp.customer_count);
    println!("  order     = {}", resp.order_count);
    println!("  db bytes  = {}", resp.database_bytes);

    Ok(())
}

/// Run benchmark
async fn run(args: RunArgs) -> Result<(), Error> {
    let endpoints = EndpointUrls::try_from(args.endpoint.as_str())?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let start_t = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let end_t = start_t + std::time::Duration::from_secs_f32(args.duration);
    let term_t = end_t + std::time::Duration::from_secs(5);

    let perf: [PerfSummary; 6] = Default::default();
    log::info!("Start benchmark");
    let futs = (0..(args.concurrent)).map(|_i| async {
        benchmark_single_terminal(start_t, end_t, term_t, 1, &perf, &endpoints, &client).await
    });
    let counts = futures::future::try_join_all(futs).await?;
    log::info!("Finished");

    let total_counts = counts.into_iter().sum::<i32>();
    println!(
        "\n{:.1} tpm  ( {} new_order transactions in {:.3} secs )\n",
        (total_counts as f32) * 60.0 / args.duration,
        total_counts,
        args.duration,
    );

    println!("##                calls , e2e total,  begin   ,  query   ,  commit");
    println!("##             ( counts ) (sec/call) (sec/call) (sec/call) (sec/call)");
    println!(
        "new_order:        {:6}, {:9.06}, {:9.06}, {:9.06}, {:9.06}",
        perf[0].counts(),
        perf[0].avg_e2e(),
        perf[0].avg_begin(),
        perf[0].avg_query(),
        perf[0].avg_commit(),
    );
    println!(
        "payment:          {:6}, {:9.06}, {:9.06}, {:9.06}, {:9.06}",
        perf[1].counts(),
        perf[1].avg_e2e(),
        perf[1].avg_begin(),
        perf[1].avg_query(),
        perf[1].avg_commit(),
    );
    println!(
        "order_status:     {:6}, {:9.06}, {:9.06}, {:9.06}, {:9.06}",
        perf[2].counts(),
        perf[2].avg_e2e(),
        perf[2].avg_begin(),
        perf[2].avg_query(),
        perf[2].avg_commit(),
    );
    println!(
        "delivery:         {:6}, {:9.06}, {:9.06}, {:9.06}, {:9.06}",
        perf[3].counts(),
        perf[3].avg_e2e(),
        perf[3].avg_begin(),
        perf[3].avg_query(),
        perf[3].avg_commit(),
    );
    println!(
        "stock_level:      {:6}, {:9.06}, {:9.06}, {:9.06}, {:9.06}",
        perf[4].counts(),
        perf[4].avg_e2e(),
        perf[4].avg_begin(),
        perf[4].avg_query(),
        perf[4].avg_commit(),
    );
    println!(
        "customer_by_name: {:6}, {:9.06}, {:9.06}, {:9.06}, {:9.06}",
        perf[5].counts(),
        perf[5].avg_e2e(),
        perf[5].avg_begin(),
        perf[5].avg_query(),
        perf[5].avg_commit(),
    );

    Ok(())
}

async fn benchmark_single_terminal(
    start_t: std::time::Instant,
    end_t: std::time::Instant,
    term_t: std::time::Instant,
    warehouse_id: i32,
    perf: &[PerfSummary; 6],
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
) -> Result<i32, Error> {
    let mut counts = 0u32;
    let mut new_orders = 0;
    let mut rand = tpcc_rand::TpcRandom::new();

    while std::time::Instant::now() < term_t {
        // 5.2.3
        // Mix of each transaction
        match counts % 25 {
            0 | 2 | 4 | 6 | 9 | 11 | 13 | 15 | 18 | 20 | 22 => {
                // 44%
                new_order_req(warehouse_id, &perf[0], &endpoints, &client, &mut rand).await?;
                // Only count up in benchmark period (excludes ramp-up, ramp-down)
                let now = std::time::Instant::now();
                if start_t <= now && now < end_t {
                    new_orders += 1;
                }
            }
            1 | 3 | 5 | 7 | 10 | 12 | 14 | 16 | 19 | 21 | 23 => {
                // 44%
                payment_req(
                    warehouse_id,
                    &perf[1],
                    &perf[5],
                    &endpoints,
                    &client,
                    &mut rand,
                )
                .await?;
            }
            8 => {
                // 4%
                order_status_req(
                    warehouse_id,
                    &perf[2],
                    &perf[5],
                    &endpoints,
                    &client,
                    &mut rand,
                )
                .await?;
            }
            17 => {
                // 4%
                delivery_req(warehouse_id, &perf[3], &endpoints, &client, &mut rand).await?;
            }
            24 => {
                // 4%
                stock_level_req(warehouse_id, &perf[4], &endpoints, &client, &mut rand).await?;
            }
            _ => {}
        }
        counts += 1
    }
    Ok(new_orders)
}

/// New-Order Transaction
/// TPC-C standard spec. 2.4
async fn new_order_req(
    warehouse_id: i32,
    perf: &PerfSummary,
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
    rand: &mut tpcc_rand::TpcRandom,
) -> Result<bool, Error> {
    // 2.4.1.3
    let item_count = rand.i32_range(5..=15);
    // 2.4.1.5
    let items = (0..item_count)
        .map(|_| if_types::NewOrderRequestItem {
            item_id: rand.non_uniform_i32(8191, 1..=100000),
            quantity: rand.i32_range(1..=10),
        })
        .collect::<Vec<_>>();

    // 2.4.1.2
    let req = if_types::NewOrderRequest {
        terminal_id: warehouse_id,
        warehouse_id,
        district_id: rand.i32_range(1..=10),
        customer_id: rand.non_uniform_i32(1023, 1..=3000),
        items,
        // 2.4.1.4, rollback in 1/100 transaction
        inject_rollback: rand.i32_range(0..=99) == 0,
    };

    let t = std::time::Instant::now();
    let resp = client.post(endpoints.new_order()).json(&req).send().await?;

    let resp = resp.json::<if_types::NewOrderResponse>().await?;
    let elapsed = t.elapsed();

    perf.add(&resp.perf, elapsed);
    log::debug!("New-Order succeeded in {:.03}s", elapsed.as_secs_f32());

    Ok(true)
}

/// Payment Transaction
/// TPC-C standard spec. 2.5
async fn payment_req(
    warehouse_id: i32,
    perf: &PerfSummary,
    perf_c: &PerfSummary,
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
    rand: &mut tpcc_rand::TpcRandom,
) -> Result<bool, Error> {
    // 2.5.1.2
    let district_id = rand.i32_range(1..=10);

    let (c_w_id, c_d_id) = if rand.i32_range(1..=100) <= 85 {
        // home district
        (warehouse_id, district_id)
    } else {
        // remote district
        // ToDo: random remote warehouse ID
        (warehouse_id, rand.i32_range(1..=10))
    };

    let c_id = if rand.i32_range(1..=100) <= 60 {
        // by name
        let name_idx = rand.non_uniform_i32(255, 0..=999);
        let lastname = tpcc_rand::TpcRandom::last_name(name_idx);

        customer_id_by_lastname(
            warehouse_id,
            district_id,
            lastname,
            perf_c,
            endpoints,
            client,
        )
        .await?
    } else {
        // by id
        rand.non_uniform_i32(1023, 1..=3000)
    };

    let req = if_types::PaymentRequest {
        terminal_id: warehouse_id,
        warehouse_id: warehouse_id,
        district_id,
        customer_warehouse_id: c_w_id,
        customer_district_id: c_d_id,
        customer_id: c_id,
        // 2.5.1.3
        amount: rand.f64_range(1.00..=5_000.00),
    };

    let t = std::time::Instant::now();
    let resp = client.post(endpoints.payment()).json(&req).send().await?;

    let resp = resp
        .error_for_status()?
        .json::<if_types::PaymentResponse>()
        .await?;
    let elapsed = t.elapsed();

    perf.add(&resp.perf, elapsed);
    log::debug!("Payment succeeded in {:.03}s", elapsed.as_secs_f32());

    Ok(true)
}

/// Order-Status Transaction
/// TPC-C standard spec. 2.6
async fn order_status_req(
    warehouse_id: i32,
    perf: &PerfSummary,
    perf_c: &PerfSummary,
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
    rand: &mut tpcc_rand::TpcRandom,
) -> Result<bool, Error> {
    // 2.6.1.2
    let district_id = rand.i32_range(1..=10);

    let c_id = if rand.i32_range(1..=100) <= 60 {
        // by name
        let name_idx = rand.non_uniform_i32(255, 0..=999);
        let lastname = tpcc_rand::TpcRandom::last_name(name_idx);

        customer_id_by_lastname(
            warehouse_id,
            district_id,
            lastname,
            perf_c,
            endpoints,
            client,
        )
        .await?
    } else {
        // by id
        rand.non_uniform_i32(1023, 1..=3000)
    };

    let t = std::time::Instant::now();
    let resp = client
        .get(endpoints.order_status(warehouse_id, district_id, c_id))
        .send()
        .await?;

    let resp = resp
        .error_for_status()?
        .json::<if_types::OrderStatusResponse>()
        .await?;
    let elapsed = t.elapsed();

    perf.add(&resp.perf, elapsed);
    log::debug!(
        "Order-Status succeeded in {:.03}s, {} order found.",
        elapsed.as_secs_f32(),
        resp.contents.orders.len()
    );

    Ok(true)
}

/// Delivery Transaction
/// TPC-C standard spec. 2.7
async fn delivery_req(
    warehouse_id: i32,
    perf: &PerfSummary,
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
    rand: &mut tpcc_rand::TpcRandom,
) -> Result<i32, Error> {
    // 2.7.1.2
    let carrier_id = rand.i32_range(1..=10);
    let req = if_types::DeliveryRequest {
        warehouse_id,
        carrier_id,
    };

    let t = std::time::Instant::now();
    let resp = client.post(endpoints.delivery()).json(&req).send().await?;

    let resp = resp
        .error_for_status()?
        .json::<if_types::DeliveryResponse>()
        .await?;
    let elapsed = t.elapsed();

    perf.add(&resp.perf, elapsed);
    log::debug!(
        "Delivery succeeded in {:.03}s, {} orders delivered.",
        elapsed.as_secs_f32(),
        resp.contents.deliverd_orders
    );

    Ok(resp.contents.deliverd_orders)
}

/// Stock-Level Transaction
/// TPC-C standard spec. 2.8
async fn stock_level_req(
    warehouse_id: i32,
    perf: &PerfSummary,
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
    rand: &mut tpcc_rand::TpcRandom,
) -> Result<bool, Error> {
    for district_id in 1..=10 {
        let stock_level = rand.i32_range(10..=20);

        let t = std::time::Instant::now();
        let resp = client
            .get(endpoints.check_stocks(warehouse_id, district_id))
            .query(&if_types::StockLevelParams { stock_level })
            .send()
            .await?;

        let resp = resp
            .error_for_status()?
            .json::<if_types::StockLevelResponse>()
            .await?;
        let elapsed = t.elapsed();

        perf.add(&resp.perf, elapsed);
        log::debug!(
            "Stock-Level succeeded in {:.03}s, in district {}, {} low stocks found.",
            elapsed.as_secs_f32(),
            district_id,
            resp.contents.low_stocks
        );
    }
    Ok(true)
}

/// Search customer_id by lastname
async fn customer_id_by_lastname(
    warehouse_id: i32,
    district_id: i32,
    lastname: String,
    perf: &PerfSummary,
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
) -> Result<i32, Error> {
    // Get customer ID by name
    let t = std::time::Instant::now();
    let resp = client
        .get(endpoints.customer_by_lastname())
        .query(&if_types::CustomersByLastnameParams {
            warehouse_id,
            district_id,
            lastname,
        })
        .send()
        .await?;
    let resp = resp.json::<if_types::CustomersResponse>().await?;
    let elapsed = t.elapsed();
    perf.add(&resp.perf, elapsed);

    let customers = resp.contents.customers;
    let customer = &customers[customers.len() / 2];
    log::debug!("Customer by lastname in {:.03}s", elapsed.as_secs_f32());

    Ok(customer.customer_id)
}

/*
/// Query benchmark status
async fn status(
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
) -> Result<if_types::DbStatusResponse, Error> {
    let resp = client.get(endpoints.status()).send().await?;

    let status = resp.json::<if_types::DbStatusResponse>().await?;

    Ok(status)
}
*/

#[derive(Default)]
struct PerfSummary {
    counts: std::sync::atomic::AtomicUsize,
    begin_us: std::sync::atomic::AtomicUsize,
    query_us: std::sync::atomic::AtomicUsize,
    commit_us: std::sync::atomic::AtomicUsize,
    e2e_total_us: std::sync::atomic::AtomicUsize,
}

impl PerfSummary {
    fn add(&self, perf: &if_types::PerformanceMetrics, e2e: std::time::Duration) {
        use std::sync::atomic::Ordering::Relaxed;

        self.counts.fetch_add(1, Relaxed);
        self.begin_us
            .fetch_add((perf.begin * 1_000_000.0) as usize, Relaxed);
        self.query_us
            .fetch_add((perf.query * 1_000_000.0) as usize, Relaxed);
        self.commit_us
            .fetch_add((perf.commit * 1_000_000.0) as usize, Relaxed);
        self.e2e_total_us
            .fetch_add(e2e.as_micros() as usize, Relaxed);
    }

    fn counts(&self) -> usize {
        use std::sync::atomic::Ordering::Relaxed;
        self.counts.load(Relaxed)
    }

    fn avg_begin(&self) -> f64 {
        use std::sync::atomic::Ordering::Relaxed;
        (self.begin_us.load(Relaxed) as f64) / (self.counts.load(Relaxed) as f64) * 0.000_001
    }

    fn avg_query(&self) -> f64 {
        use std::sync::atomic::Ordering::Relaxed;
        (self.query_us.load(Relaxed) as f64) / (self.counts.load(Relaxed) as f64) * 0.000_001
    }

    fn avg_commit(&self) -> f64 {
        use std::sync::atomic::Ordering::Relaxed;
        (self.commit_us.load(Relaxed) as f64) / (self.counts.load(Relaxed) as f64) * 0.000_001
    }

    fn avg_e2e(&self) -> f64 {
        use std::sync::atomic::Ordering::Relaxed;
        (self.e2e_total_us.load(Relaxed) as f64) / (self.counts.load(Relaxed) as f64) * 0.000_001
    }
}
