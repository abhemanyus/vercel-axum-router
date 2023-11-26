use axum::{body::Body, http::Request, routing::get, Router};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use vercel_runtime::{process_request, process_response, run_service, Error, ServiceBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
    #[cfg(debug_assertions)]
    {
        dotenv::dotenv().ok();
    }
    let DATABASE_URL = std::env::var("DATABASE_URL").unwrap();
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();
    let trace_layer =
        TraceLayer::new_for_http().on_request(|_: &Request<Body>, _: &tracing::Span| {
            tracing::info!(message = "begin request!")
        });
    let cors_layer = CorsLayer::new()
        .allow_headers(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);
    let app = Router::new().route("/hello", get(root)).layer(cors_layer);
    // .layer(trace_layer)
    // .layer(CompressionLayer::new().gzip(true).deflate(true));
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
