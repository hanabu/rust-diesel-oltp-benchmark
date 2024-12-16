use sut::*;

/// main() function for regular server environment or localhost
#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!(
        "Server starts listening on {:?}",
        listener.local_addr().unwrap()
    );
    // DB connections
    let db_connections = if let Ok(db_conn) = std::env::var("DB_CONN") {
        db_conn
            .parse::<u32>()
            .expect("Can not parse DB_CONN as integer")
    } else {
        2 * num_cpus::get() as u32
    };
    axum::serve(listener, app(db_connections).await).await?;

    Ok(())
}
