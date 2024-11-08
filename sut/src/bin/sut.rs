use sut::*;

/// main() function for regular server environment or localhost
#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!(
        "Server starts listening on {:?}",
        listener.local_addr().unwrap()
    );
    let db_connections = 2 * num_cpus::get() as u32;
    axum::serve(listener, app(db_connections).await).await?;

    Ok(())
}
