/// main() function for Lambda runtime
///
/// cargo build --release will generate target/release/bootstrap from this lambda_bootstrap.rs,
/// then compress this bootstrap in ZIP and deploy it to the Lambda function.
///
use sut::*;
#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // On lambda, single thread, single DB connection
    lambda_http::run(app(1).await).await?;

    Ok(())
}
