use axum::{
    extract::ws::WebSocketUpgrade,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use log::*;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::services::ServeDir;

use crate::{
    agent,
    config::{get_cert, get_key, get_src_addr, get_web},
};

pub async fn run() -> anyhow::Result<()> {
    let cert = get_cert();
    // configure certificate and private key used by https
    let config = if cert.is_empty() {
        None
    } else {
        Some(RustlsConfig::from_pem_file(cert, get_key()).await?)
    };

    // build our application with some routes
    let app = Router::new()
        .layer(ConcurrencyLimitLayer::new(2))
        .fallback(
            get_service(
                ServeDir::new(get_web())
                    .precompressed_gzip()
                    .append_index_html_on_directories(true),
            )
            .handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        .route("/websockify", get(ws_handler));

    // run it with hyper
    let addr = get_src_addr().parse()?;

    match config {
        Some(config) => {
            axum_server::bind_rustls(addr, config)
                .serve(app.into_make_service())
                .await?
        }
        _ => {
            axum_server::bind(addr)
                .serve(app.into_make_service())
                .await?
        }
    }
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    info!("A client connectted");
    ws.protocols(["binary"]).on_upgrade(agent::handle_client)
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
