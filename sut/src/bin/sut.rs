use sut::*;

/// main() function for regular server environment or localhost
#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!(
        "Server starts listening on {:?}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app().await).await?;

    Ok(())
}
