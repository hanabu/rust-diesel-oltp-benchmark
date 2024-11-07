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
    /// Wait : 0.0 for no wait, 1.0 for value in TPC-C spec.
    #[arg(short, long, default_value = "1.0")]
    wait: f32,
    /// Endpoint URL of SUT
    endpoint: String,
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

    let endpoint = url::Url::parse(&args.endpoint)?;
    let endpoint_with_path = endpoint.join("/prepare_db")?;
    println!(
        "Requesting POST {} with scale_factor={}",
        endpoint_with_path.as_str(),
        args.scale_factor
    );
    
    let t = std::time::Instant::now();
    let resp = client
        .post(endpoint_with_path)
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
async fn run(_args: RunArgs) -> Result<(), Error> {
    Ok(())
}
