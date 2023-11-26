use axum::{routing::get, Router};
use vercel_runtime::{process_request, process_response, run_service, Error, ServiceBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
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
