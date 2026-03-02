use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
};
use tokio::{net::TcpListener, sync::watch, task::JoinHandle};

use crate::aggregator::AppMetrics;

#[derive(Clone)]
struct ExporterState {
    metrics: Arc<AppMetrics>,
}

pub async fn spawn(
    metrics: Arc<AppMetrics>,
    bind_addr: &str,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(SocketAddr, JoinHandle<Result<()>>)> {
    let listener = TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("failed to bind metrics exporter to {bind_addr}"))?;
    let local_addr = listener.local_addr()?;

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/healthz", get(health_handler))
        .with_state(ExporterState { metrics });

    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                loop {
                    if shutdown_rx.changed().await.is_err() {
                        break;
                    }
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            })
            .await
            .context("metrics exporter server exited with error")
    });

    Ok((local_addr, handle))
}

async fn metrics_handler(State(state): State<ExporterState>) -> impl IntoResponse {
    match state.metrics.render() {
        Ok(body) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static(
                    "application/openmetrics-text; version=1.0.0; charset=utf-8",
                ),
            );
            (StatusCode::OK, headers, body).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to render metrics: {err:#}"),
        )
            .into_response(),
    }
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
