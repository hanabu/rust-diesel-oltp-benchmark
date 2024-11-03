pub type Error = Box<(dyn std::error::Error + Send + Sync + 'static)>;

/// Initialize & build Axum route
pub async fn app() -> axum::Router {
    use axum::routing::get;

    // If you need S3 client, databae connection etc. initialize here.
    //let sdk_cfg = aws_config::load_from_env().await;
    //let s3_client = aws_sdk_s3::Client::new(&sdk_cfg);

    axum::Router::new().route("/", get(hello))
    //    .layer(axum::extract::Extension(s3_client))
}

/// Sample hello handler
async fn hello() -> axum::response::Html<String> {
    let body = "<html><body><p>Hello</p></body></html>".to_string();

    axum::response::Html(body)
}
