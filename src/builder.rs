use axum::{BoxError, Router as AxumRouter};
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Request, Server};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::signal;
use tonic::transport::server::Router as TonicRouter;
use tower::make::Shared;
use tower::steer::Steer;
use tower::ServiceExt;
use tracing::info;

use crate::utils::*;

const DEFAULT_PORT: u16 = 8000;
const DEFAULT_MANAGEMENT_PORT: u16 = 8001;

pub struct Application<H: Health> {
    health_indicator: Option<H>,
    host: Option<IpAddr>,
    port: Option<u16>,
    management_port: Option<u16>,
    rest_router: Option<AxumRouter>,
    grpc_router: Option<TonicRouter>,
}

impl Application<NoHealth> {
    pub fn new_without_health() -> Application<NoHealth> {
        Application::<NoHealth> {
            health_indicator: None,
            host: None,
            port: None,
            management_port: None,
            rest_router: None,
            grpc_router: None,
        }
    }
}

impl<H: Health> Application<H> {
    pub fn new_with_health(health: H) -> Self {
        Self {
            health_indicator: Some(health),
            host: None,
            port: None,
            management_port: None,
            rest_router: None,
            grpc_router: None,
        }
    }

    pub async fn run(mut self) -> hyper::Result<()> {
        let rest_application = build_application_router(self.rest_router);
        let rest_management = build_management_router(self.health_indicator);

        let application_socket = SocketAddr::new(
            self.host.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            self.port.unwrap_or(DEFAULT_PORT),
        );

        let management_socket = SocketAddr::new(
            self.host.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            self.management_port.unwrap_or(DEFAULT_MANAGEMENT_PORT),
        );

        // TODO: MAKE GRPC A FEATURE ?
        // TODO: GENERIC FOR SERVER TYPE TO REMOVE DIFFERENT FUNCTIONS
        if let Some(grpc) = self.grpc_router.take() {
            tokio::try_join!(
                run_rest_and_grpc_service(
                    &application_socket,
                    rest_application,
                    grpc,
                    true,
                    "rest + grpc",
                ),
                run_rest_service(&management_socket, rest_management, false, "management")
            )?
        } else {
            tokio::try_join!(
                run_rest_service(&application_socket, rest_application, true, "rest"),
                run_rest_service(&management_socket, rest_management, false, "management")
            )?
        };

        Ok(())
    }

    pub fn rest_router(mut self, router: AxumRouter) -> Self {
        self.rest_router = Some(router);
        self
    }

    pub fn grpc_router(mut self, router: TonicRouter) -> Self {
        self.grpc_router = Some(router);
        self
    }

    pub fn host(mut self, host: impl Into<IpAddr>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: impl Into<u16>) -> Self {
        self.port = Some(port.into());
        self
    }

    pub fn management_port(mut self, management_port: impl Into<u16>) -> Self {
        self.management_port = Some(management_port.into());
        self
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Termination signal, starting shutdown...");
}

async fn run_rest_service(
    socket: &SocketAddr,
    rest: AxumRouter,
    graceful_shutdown: bool,
    server_type: &str,
) -> hyper::Result<()> {
    let server = Server::bind(socket).serve(rest.into_make_service());
    info!(target: "server", server_type = server_type, "Started: http://{socket}");

    if graceful_shutdown {
        server.with_graceful_shutdown(shutdown_signal()).await
    } else {
        server.await
    }
}

async fn run_rest_and_grpc_service(
    socket: &SocketAddr,
    rest: AxumRouter,
    grpc: TonicRouter,
    graceful_shutdown: bool,
    server_type: &str,
) -> hyper::Result<()> {
    let rest = rest.map_err(BoxError::from).boxed_clone();

    let grpc = grpc
        .into_service()
        .map_response(|r| r.map(axum::body::boxed))
        .boxed_clone();

    let rest_grpc = Steer::new(vec![rest, grpc], |req: &Request<Body>, _svcs: &[_]| {
        if req.headers().get(CONTENT_TYPE).map(|v| v.as_bytes()) != Some(b"application/grpc") {
            0
        } else {
            1
        }
    });

    let server = Server::bind(socket).serve(Shared::new(rest_grpc));

    info!(target: "server", server_type = server_type, "Started: http://{socket}");

    if graceful_shutdown {
        server.with_graceful_shutdown(shutdown_signal()).await
    } else {
        server.await
    }
}
