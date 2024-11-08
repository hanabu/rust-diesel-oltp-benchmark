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
    concurrent: f32,
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
    pub fn customer_by_lastname(&self) -> url::Url {
        self.customer.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    use clap::Parser;
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
    println!(
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
    println!(
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
    let mut rand = tpcc_rand::TpcRandom::new();
    let endpoints = EndpointUrls::try_from(args.endpoint.as_str())?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    new_order_req(1, &endpoints, &client, &mut rand).await?;
    payment_req(1, &endpoints, &client, &mut rand).await?;
    Ok(())
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
    println!("New-Order succeeded in {:.03}s", t.elapsed().as_secs_f32());

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

    // 2.5.1.2
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

    let _resp = resp.json::<if_types::PaymentResponse>().await?;
    println!("Payment succeeded in {:.03}s", t.elapsed().as_secs_f32());

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
    let customers = resp.json::<if_types::CustomersResponse>().await?.customers;
    let customer = &customers[customers.len() / 2];
    println!("Customer by lastname in {:.03}s", t.elapsed().as_secs_f32());

    Ok(customer.customer_id)
}
