# Crucible Backend

This is the backend service layer for the Crucible project.

## Technologies
- **Axum**: Web framework
- **Tokio**: Async runtime
- **SQLx**: PostgreSQL driver (with `uuid` and `chrono` support)
- **Redis**: Caching and job queues
- **Tracing**: Observability

## Structure
- `src/api/` – API handlers and routing
- `src/config/` – Application configuration and hot-reload
- `src/db/` – Database utilities and seed data
- `src/services/` – Business logic and external integrations

### API Handlers (`src/api/handlers/`)

| Module | Description |
|---|---|
| `profiling` | System status and profiling trigger endpoints |
| `dashboard` | Aggregated dashboard data endpoint with Redis caching |



| Module | Description |
|---|---|
| `sys_metrics` | Collects and exposes system metrics (CPU, memory, uptime) |
| `error_recovery` | Tracks retry state for failing tasks with configurable max retries |
| `log_aggregator` | Async MPSC-based log pipeline; persists entries via a background worker |
| `log_alerts` | Threshold-based alerting over the log pipeline with sliding-window evaluation |
| `feature_flags` | Feature flag management backed by PostgreSQL with Redis caching |
| `alerts` | Critical-error notification dispatcher — deduplication, in-memory queue, Redis pub/sub |
| `tracing` | OpenTelemetry tracing initialisation — wires `tracing` spans to an OTLP HTTP exporter |

### Database (`src/db/`)

| Module | Description |
|---|---|
| `seeds` | Idempotent seed data for development and test environments |

## API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/status` | System health, metrics, and active recovery tasks |
| `POST` | `/api/profile` | Trigger a profiling collection run |
| `GET` | `/api/dashboard` | Aggregated dashboard data: metrics, recovery tasks, and active alerts (Redis-cached, 30 s TTL) |

## Running
```bash
cargo run -p backend
```

## Testing
```bash
# All tests (unit + integration + load)
cargo test -p backend

# Load tests only
cargo test -p backend --test load_tests -- --nocapture
```

## Configuration Hot-Reload

`ConfigWatcher` holds the live `AppConfig` behind an `Arc<RwLock<_>>`. Any part of the application that holds a `ConfigHandle` sees new values immediately after a reload — no restart required.

```rust
use std::sync::Arc;
use backend::config::reload::{AppConfig, ConfigWatcher};

let watcher = Arc::new(ConfigWatcher::new(AppConfig::default()));
let handle = watcher.handle(); // cheap to clone, share across handlers

// Manual reload
watcher.reload(AppConfig { maintenance_mode: true, ..AppConfig::default() }).await;

// Reload from Redis key `config:current`
watcher.reload_from_redis(&redis_client).await?;

// Background watcher — subscribes to `config:reload` pub/sub channel
watcher.watch(redis_client); // returns a JoinHandle
```

Trigger a reload from the Redis CLI:

```bash
redis-cli SET config:current '{"log_level":"info","max_connections":50,"request_timeout_secs":30,"maintenance_mode":false,"redis_config_key":"config:current"}'
redis-cli PUBLISH config:reload reload
```

| Field | Default | Description |
|---|---|---|
| `log_level` | `backend=debug` | Tracing filter directive |
| `max_connections` | `10` | DB pool size |
| `request_timeout_secs` | `30` | HTTP request timeout |
| `maintenance_mode` | `false` | Maintenance banner flag |
| `redis_config_key` | `config:current` | Redis key for config JSON |

## Critical Error Alerting

`AlertDispatcher` sits on top of `log_alerts` and dispatches notifications when a critical condition fires. It deduplicates within a configurable cooldown window and publishes to Redis pub/sub.

```rust
use std::sync::Arc;
use backend::services::alerts::{AlertDispatcher, AlertNotification, NotificationLevel};

let dispatcher = Arc::new(AlertDispatcher::new(Some(redis_client), 60));

// Dispatch directly
dispatcher.dispatch(AlertNotification {
    alert_key: "db_down".to_string(),
    level: NotificationLevel::Critical,
    title: "Database unreachable".to_string(),
    message: "Pool exhausted after 3 retries".to_string(),
    metadata: Default::default(),
}).await?;

// Or derive from a fired log_alerts::Alert (only Critical severity is dispatched)
dispatcher.dispatch_alert(&fired_alert).await?;

// Drain the in-memory queue
let pending = dispatcher.drain_notifications().await;
```

Redis pub/sub channel defaults to `alerts:critical`; override with `.with_channel("my-channel")`.



Spans from every `#[tracing::instrument]`-annotated function are exported to an OTLP-compatible collector over HTTP/protobuf.

```rust
use backend::services::tracing::{init, TracingConfig};

let _guard = init(TracingConfig::from_env())?;
// spans are now exported; _guard flushes them on drop
```

| Environment variable | Default | Description |
|---|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://localhost:4318` | OTLP HTTP collector URL |
| `OTEL_SERVICE_NAME` | `backend` | Service name on every span |
| `RUST_LOG` | `backend=debug` | Log/span filter directive |

Run a local collector with Docker:

```bash
docker run -d -p4317:4317 -p4318:4318 -p16686:16686 jaegertracing/all-in-one:latest
# View traces at http://localhost:16686
```



Feature flags are stored in PostgreSQL and cached in Redis with a 5-minute TTL.

```rust
let service = FeatureFlagService::new(pool, redis_client);

// Check a flag
if service.is_enabled("new_dashboard").await? {
    // render new UI
}

// Create / update a flag
service.set("new_dashboard", true, "Enable redesigned dashboard").await?;
```

## Log Alerts

Alert rules evaluate incoming log entries against a pattern within a sliding time window.

```rust
let manager = AlertManager::new();
manager.add_rule(AlertRule {
    id: Uuid::new_v4(),
    name: "High error rate".to_string(),
    pattern: "ERROR".to_string(),
    severity: AlertSeverity::Critical,
    threshold: 5,
    window_secs: 60,
}).await?;

// Evaluate a log entry
manager.evaluate(&log_entry).await;

// Retrieve fired alerts
let alerts = manager.get_active_alerts().await;
```

## Database Seeds

Seeds are idempotent and safe to run multiple times:

```bash
# In application code
run_all(&pool).await?;
```

Seeds populate:
- `users` table with two default accounts (`admin`, `dev`)
- `feature_flags` table with baseline flags (`new_dashboard`, `beta_api`)
