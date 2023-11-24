use axum::routing::get;
use vercel_runtime::{run, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = axum::Router::new().route("/", get(|_| "Hello, world!".into()));
    let app = tower::ServiceBuilder::new()
        .layer(vermicelli::LambdaLayer::default())
        .service(app);
    run(app).await
}
