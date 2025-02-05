use fregate::axum::response::IntoResponse;
use fregate::axum::routing::get;
use fregate::axum::Router;
use fregate::hyper::{Body, Request, Response, StatusCode};
use fregate::middleware::{ProxyError, ProxyLayer};
use fregate::ConfigSource::EnvPrefix;
use fregate::{axum, bootstrap, hyper, tracing, AppConfig, Application};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn on_proxy_request<TBody>(request: &Request<TBody>, _ext: &()) {
    tracing::info!("Proxy sends request to: {}", request.uri());
}

fn on_proxy_response(response: &mut Response<Body>, _ext: &()) {
    tracing::info!("Proxy response status code: {}", response.status());
}

fn on_proxy_error(err: ProxyError, _ext: &()) -> axum::response::Response {
    tracing::info!("Proxy error: {}", err);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

// Proxy every second request.
async fn should_proxy(counter: Arc<AtomicUsize>) -> bool {
    counter.fetch_add(1, Ordering::Acquire) % 2 == 0
}

#[tokio::main]
async fn main() {
    std::env::set_var("LOCAL_PORT", "8001");
    let conf: AppConfig = bootstrap([EnvPrefix("LOCAL")]).unwrap();

    start_server();

    let counter = Arc::new(AtomicUsize::new(0));

    let http_client = hyper::Client::new();

    // You might want apply additional layers to Client.
    // use fregate::tower::timeout::TimeoutLayer;
    // use fregate::tower::ServiceBuilder;
    // use std::time::Duration;
    //
    // let http_client = ServiceBuilder::new()
    //     .layer(TimeoutLayer::new(Duration::from_nanos(10)))
    //     .service(hyper::Client::new());

    let proxy_layer = ProxyLayer::new(
        http_client,
        "http://localhost:8000",
        on_proxy_error,
        on_proxy_request,
        on_proxy_response,
        move |_request, _ext| Box::pin(should_proxy(counter.clone())),
    )
    .unwrap();

    let local_handler = Router::new()
        .route("/local", get(|| async { "Hello, Local Handler!" }))
        .layer(proxy_layer);

    Application::new(&conf)
        .router(local_handler)
        .use_default_tracing_layer(false)
        .serve()
        .await
        .unwrap();
}

fn start_server() {
    let remote_handler = Router::new().route("/local", get(|| async { "Hello, Remote Handler!" }));

    // This will start server on 8000 port by default
    tokio::task::spawn(async {
        Application::new(&AppConfig::default())
            .router(remote_handler)
            .use_default_tracing_layer(false)
            .serve()
            .await
            .unwrap();
    });
}

// curl http://0.0.0.0:8001/local
