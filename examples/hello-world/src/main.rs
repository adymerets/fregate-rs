use fregate::{
    axum::{routing::get, Router},
    bootstrap, tokio, AppConfig, Application,
};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let config: AppConfig = bootstrap([]).unwrap();

    Application::new(&config)
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
