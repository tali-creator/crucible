mod api;
mod services;

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use crate::api::handlers::profiling::{AppState, get_system_status, trigger_profile_collection};
use crate::api::handlers::dashboard::{DashboardState, get_dashboard};
use crate::services::{
    sys_metrics::MetricsExporter,
    error_recovery::ErrorManager,
    log_aggregator::LogAggregator,
    log_alerts::AlertManager,
    tracing::{init as init_tracing, TracingConfig},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise OpenTelemetry tracing. The guard must be kept alive for the
    // duration of the process so spans are flushed on shutdown.
    let _tracing_guard = init_tracing(TracingConfig::from_env())
        .unwrap_or_else(|e| {
            eprintln!("OTel tracing init failed ({e}); falling back to fmt subscriber");
            tracing_subscriber::fmt().init();
            crate::services::tracing::TracingGuard::noop()
        });

    // Initialize services
    let metrics_exporter = Arc::new(MetricsExporter::new());
    let error_manager = Arc::new(ErrorManager::new());
    let alert_manager = Arc::new(AlertManager::new());
    let (_log_aggregator, log_receiver) = LogAggregator::new();

    // Spawn background workers
    tokio::spawn(MetricsExporter::run_collector(metrics_exporter.clone()));
    tokio::spawn(LogAggregator::run_worker(log_receiver));

    let redis = redis::Client::open(
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string()),
    )?;

    let profiling_state = Arc::new(AppState {
        metrics_exporter: metrics_exporter.clone(),
        error_manager: error_manager.clone(),
    });

    let dashboard_state = Arc::new(DashboardState {
        metrics_exporter,
        error_manager,
        alert_manager,
        redis,
    });

    // Build router
    let app = Router::new()
        .route("/api/status", get(get_system_status))
        .route("/api/profile", post(trigger_profile_collection))
        .with_state(profiling_state)
        .route("/api/dashboard", get(get_dashboard))
        .with_state(dashboard_state);

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
