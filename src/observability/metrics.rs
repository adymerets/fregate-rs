pub(crate) mod recorder;
#[cfg(feature = "tokio-metrics")]
pub mod tokio_metrics;

use crate::error::Result;
use crate::observability::metrics::recorder::{get_handle, get_recorder};

/// Return rendered metrics.
/// By default fregate sets `/metrics` endpoint for your [`Application]` which uses [`metrics_exporter_prometheus::PrometheusHandle::render`] fn to get currently available metrics.
/// How callback might be used see in [`example`](https://github.com/elefant-dev/fregate-rs/tree/main/examples/metrics-callback).
pub fn render_metrics(callback: Option<&(dyn Fn() + Send + Sync + 'static)>) -> String {
    if let Some(callback) = callback {
        callback();
    }

    get_handle().render()
}

/// Initialise PrometheusRecorder
pub fn init_metrics() -> Result<()> {
    metrics::set_recorder(get_recorder())?;

    #[cfg(feature = "tokio-metrics")]
    tokio_metrics::register_metrics();

    Ok(())
}
