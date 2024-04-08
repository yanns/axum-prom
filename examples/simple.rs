use axum::extract::Path;
use axum::{routing::get, Router};
use axum_prom::PrometheusMetricsBuilder;

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello {}!", name)
}

#[tokio::main]
async fn main() {
    let (prometheus, prometheus_registry) = PrometheusMetricsBuilder::new("myapp").pair().unwrap();

    // possibility to register your own metrics in `&prometheus_registry.registry`

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/hello/:name", get(hello))
        .route(
            axum_prom::DEFAULT_ENDPOINT,
            get(|| async move { prometheus_registry.metrics() }),
        )
        .layer(prometheus);

    // run it with hyper on localhost:3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
