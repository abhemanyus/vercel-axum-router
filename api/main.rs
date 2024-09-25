use axum::{routing::get, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use vercel_runtime::{process_request, process_response, run_service, Error, ServiceBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| "vermicelli=debug,tower_sessions=debug,sqlx=warn,tower_http=debug".into(),
        )))
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;
    let app = Router::new().route("/hello", get(root));
    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .layer(vermicelli::LambdaLayer::default())
        .service(app);
    run_service(handler).await
}
async fn root() -> &'static str {
    "Hello, World!"
}
