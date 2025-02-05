#![allow(clippy::derive_partial_eq_without_eq)]

use axum::{routing::get, Router};
use fregate::{axum, bootstrap, extensions::RouterTonicExt, tokio, AppConfig, Application};
use resources::{grpc::MyEcho, proto::echo::echo_server::EchoServer, FILE_DESCRIPTOR_SET};

#[tokio::main]
async fn main() {
    let config: AppConfig = bootstrap([]).unwrap();
    let echo_service = EchoServer::new(MyEcho);

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let reflection = Router::from_tonic_service(service);

    let rest = Router::new().route("/", get(|| async { "Hello, World!" }));
    let grpc = Router::from_tonic_service(echo_service);

    let app_router = rest.merge(grpc).merge(reflection);

    Application::new(&config)
        .router(app_router)
        .serve()
        .await
        .unwrap();
}

/*
    grpcurl -plaintext 0.0.0.0:8000 list
    grpcurl -plaintext -d '{"message": "Echo"}' 0.0.0.0:8000 echo.Echo/ping
    curl http://0.0.0.0:8000
    curl http://0.0.0.0:8000/health
    curl http://0.0.0.0:8000/ready
    curl http://0.0.0.0:8000/live
*/
