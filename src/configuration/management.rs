use crate::extensions::DeserializeExt;
use crate::static_assert;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

const HEALTH_ENDPOINT: &str = "/health";
const LIVE_ENDPOINT: &str = "/live";
const READY_ENDPOINT: &str = "/ready";
const METRICS_ENDPOINT: &str = "/metrics";
const VERSION_ENDPOINT: &str = "/version";

const HEALTH_PTR: &str = "/health";
const LIVE_PTR: &str = "/live";
const READY_PTR: &str = "/ready";
const METRICS_PTR: &str = "/metrics";
const VERSION_PTR: &str = "/version";

#[derive(Debug, Default, Clone, Deserialize)]
/// [`Management`](https://github.com/elefant-dev/fregate-rs/blob/main/src/application/management.rs) configuration. Currently only endpoints configuration is supported.
pub struct ManagementConfig {
    /// health and metrics endpoints.
    pub endpoints: Endpoints,
}

/// By default endpoints are:
/// ```no_run
/// const HEALTH_ENDPOINT: &str = "/health";
/// const LIVE_ENDPOINT: &str = "/live";
/// const READY_ENDPOINT: &str = "/ready";
/// const METRICS_ENDPOINT: &str = "/metrics";
/// const VERSION_ENDPOINT: &str = "/{component_name}/version";
/// ```
/// You might want to change those:\
/// Example:
/// ```no_run
/// use fregate::{
///     axum::{routing::get, Router},
///     bootstrap, tokio, AppConfig, Application, ConfigSource,
/// };
///
/// async fn handler() -> &'static str {
///     "Hello, World!"
/// }
///
/// #[tokio::main]
/// async fn main() {
///     std::env::set_var("TEST_MANAGEMENT_ENDPOINTS_METRICS", "/observability");
///     std::env::set_var("TEST_MANAGEMENT_ENDPOINTS_HEALTH", "///also_valid");
///     // this is invalid default "/live" endpoint will be used.
///     std::env::set_var("TEST_MANAGEMENT_ENDPOINTS_LIVE", "invalid");
///     // this is invalid default "/ready" endpoint will be used.
///     std::env::set_var("TEST_MANAGEMENT_ENDPOINTS_READY", "");
///
///     let config: AppConfig = bootstrap([ConfigSource::EnvPrefix("TEST")]).unwrap();
///
///     Application::new(&config)
///         .router(Router::new().route("/", get(handler)))
///         .serve()
///         .await
///         .unwrap();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Endpoints {
    /// health endpoint
    pub health: Endpoint,
    /// live endpoint
    pub live: Endpoint,
    /// ready endpoint
    pub ready: Endpoint,
    /// metrics endpoint
    pub metrics: Endpoint,
    /// version endpoint
    pub version: Endpoint,
}

#[allow(clippy::indexing_slicing)]
impl<'de> Deserialize<'de> for Endpoints {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        static_assert!(HEALTH_ENDPOINT.as_bytes()[0] == b'/');
        static_assert!(LIVE_ENDPOINT.as_bytes()[0] == b'/');
        static_assert!(READY_ENDPOINT.as_bytes()[0] == b'/');
        static_assert!(METRICS_ENDPOINT.as_bytes()[0] == b'/');
        static_assert!(VERSION_ENDPOINT.as_bytes()[0] == b'/');

        let value = Value::deserialize(deserializer)?;

        let health = value
            .pointer_and_deserialize::<_, D::Error>(HEALTH_PTR)
            .unwrap_or_else(|_| Endpoint(HEALTH_ENDPOINT.to_owned()));
        let live = value
            .pointer_and_deserialize::<_, D::Error>(LIVE_PTR)
            .unwrap_or_else(|_| Endpoint(LIVE_ENDPOINT.to_owned()));
        let ready = value
            .pointer_and_deserialize::<_, D::Error>(READY_PTR)
            .unwrap_or_else(|_| Endpoint(READY_ENDPOINT.to_owned()));
        let metrics = value
            .pointer_and_deserialize::<_, D::Error>(METRICS_PTR)
            .unwrap_or_else(|_| Endpoint(METRICS_ENDPOINT.to_owned()));
        let version = value
            .pointer_and_deserialize::<_, D::Error>(VERSION_PTR)
            .unwrap_or_else(|_| Endpoint(VERSION_ENDPOINT.to_owned()));

        Ok(Endpoints {
            health,
            live,
            ready,
            metrics,
            version,
        })
    }
}

#[allow(clippy::indexing_slicing)]
impl Default for Endpoints {
    fn default() -> Self {
        static_assert!(HEALTH_ENDPOINT.as_bytes()[0] == b'/');
        static_assert!(LIVE_ENDPOINT.as_bytes()[0] == b'/');
        static_assert!(READY_ENDPOINT.as_bytes()[0] == b'/');
        static_assert!(METRICS_ENDPOINT.as_bytes()[0] == b'/');
        static_assert!(VERSION_ENDPOINT.as_bytes()[0] == b'/');

        Self {
            health: Endpoint(HEALTH_ENDPOINT.to_owned()),
            live: Endpoint(LIVE_ENDPOINT.to_owned()),
            ready: Endpoint(READY_ENDPOINT.to_owned()),
            metrics: Endpoint(METRICS_ENDPOINT.to_owned()),
            version: Endpoint(VERSION_ENDPOINT.to_owned()),
        }
    }
}

#[derive(Debug, Clone)]
/// This is simply a wrapper over [`String`] but it checks if [`String`] starts with '/' symbol.
pub struct Endpoint(String);

impl Endpoint {
    /// Creates new [`Endpoint`].
    /// Returns error if str does not start with '/' symbol.
    pub fn new(path: &str) -> Result<Self, &'static str> {
        if path.starts_with('/') {
            Ok(Endpoint(path.to_owned()))
        } else {
            Err("Endpoint must start with a `/`")
        }
    }
}

impl<'de> Deserialize<'de> for Endpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let endpoint = String::deserialize(deserializer)?;
        Endpoint::new(endpoint.as_str())
            .map_err(|err| D::Error::invalid_value(Unexpected::Str(&endpoint), &err))
    }
}

impl AsRef<str> for Endpoint {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
