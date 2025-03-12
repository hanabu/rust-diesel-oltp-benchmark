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
    pub fn status(&self) -> url::Url {
        self.base.clone()
    }
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

    log::info!("Start benchmark");
    let futs = (0..(args.concurrent)).map(|_i| async {
        benchmark_single_terminal(start_t, end_t, term_t, 1, &endpoints, &client).await
    });
    let counts = futures::future::try_join_all(futs).await?;
    log::info!("Finished");

    let total_counts = counts.into_iter().sum::<i32>();
    println!(
        "{:.1} tpm = {} new_order transactions in {:.3} secs",
        (total_counts as f32) * 60.0 / args.duration,
        total_counts,
        args.duration,
    );

    let st = status(&endpoints, &client).await?.statistics;
    println!(
        "new_order:        {:5} calls, {:.03}s/call",
        st.new_order_count,
        st.new_order_secs / st.new_order_count as f64
    );
    println!(
        "payment:          {:5} calls, {:.03}s/call",
        st.payment_count,
        st.payment_secs / st.payment_count as f64
    );
    println!(
        "order_status:     {:5} calls, {:.03}s/call",
        st.order_status_count,
        st.order_status_secs / st.order_status_count as f64
    );
    println!(
        "delivery:         {:5} calls, {:.03}s/call",
        st.delivery_count,
        st.delivery_secs / st.delivery_count as f64
    );
    println!(
        "stock_level:      {:5} calls, {:.03}s/call",
        st.stock_level_count,
        st.stock_level_secs / st.stock_level_count as f64
    );
    println!(
        "customer_by_name: {:5} calls, {:.03}s/call",
        st.customer_by_name_count,
        st.customer_by_name_secs / st.customer_by_name_count as f64
    );

    Ok(())
}

async fn benchmark_single_terminal(
    start_t: std::time::Instant,
    end_t: std::time::Instant,
    term_t: std::time::Instant,
    warehouse_id: i32,
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
                new_order_req(warehouse_id, &endpoints, &client, &mut rand).await?;
                // Only count up in benchmark period (excludes ramp-up, ramp-down)
                let now = std::time::Instant::now();
                if start_t <= now && now < end_t {
                    new_orders += 1;
                }
            }
            1 | 3 | 5 | 7 | 10 | 12 | 14 | 16 | 19 | 21 | 23 => {
                // 44%
                payment_req(warehouse_id, &endpoints, &client, &mut rand).await?;
            }
            8 => {
                // 4%
                order_status_req(warehouse_id, &endpoints, &client, &mut rand).await?;
            }
            17 => {
                // 4%
                delivery_req(warehouse_id, &endpoints, &client, &mut rand).await?;
            }
            24 => {
                // 4%
                stock_level_req(warehouse_id, &endpoints, &client, &mut rand).await?;
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

    let _resp = resp.json::<if_types::NewOrderResponse>().await?;
    log::debug!("New-Order succeeded in {:.03}s", t.elapsed().as_secs_f32());

    Ok(true)
}

/// Payment Transaction
/// TPC-C standard spec. 2.5
async fn payment_req(
    warehouse_id: i32,
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

        customer_id_by_lastname(warehouse_id, district_id, lastname, endpoints, client).await?
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

    let _resp = resp
        .error_for_status()?
        .json::<if_types::PaymentResponse>()
        .await?;
    log::debug!("Payment succeeded in {:.03}s", t.elapsed().as_secs_f32());

    Ok(true)
}

/// Order-Status Transaction
/// TPC-C standard spec. 2.6
async fn order_status_req(
    warehouse_id: i32,
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

        customer_id_by_lastname(warehouse_id, district_id, lastname, endpoints, client).await?
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
    log::debug!(
        "Order-Status succeeded in {:.03}s, {} order found.",
        t.elapsed().as_secs_f32(),
        resp.contents.orders.len()
    );

    Ok(true)
}

/// Delivery Transaction
/// TPC-C standard spec. 2.7
async fn delivery_req(
    warehouse_id: i32,
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
    log::debug!(
        "Delivery succeeded in {:.03}s, {} orders delivered.",
        t.elapsed().as_secs_f32(),
        resp.contents.deliverd_orders
    );

    Ok(resp.contents.deliverd_orders)
}

/// Stock-Level Transaction
/// TPC-C standard spec. 2.8
async fn stock_level_req(
    warehouse_id: i32,
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
        log::debug!(
            "Stock-Level succeeded in {:.03}s, in district {}, {} low stocks found.",
            t.elapsed().as_secs_f32(),
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
    let customers = resp
        .json::<if_types::CustomersResponse>()
        .await?
        .contents
        .customers;
    let customer = &customers[customers.len() / 2];
    log::debug!("Customer by lastname in {:.03}s", t.elapsed().as_secs_f32());

    Ok(customer.customer_id)
}

/// Query benchmark status
async fn status(
    endpoints: &EndpointUrls,
    client: &reqwest::Client,
) -> Result<if_types::DbStatusResponse, Error> {
    let resp = client.get(endpoints.status()).send().await?;

    let status = resp.json::<if_types::DbStatusResponse>().await?;

    Ok(status)
}
