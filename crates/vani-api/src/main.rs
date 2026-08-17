//! Vani API — REST API for vernacular crypto operations
//!
//! Exposes Vani's core functionality via HTTP for developer integration

use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vani_mcp::vanicommand;

#[derive(Debug, Serialize, Deserialize)]
struct ParseRequest {
    text: String,
    language: Option<String>, // "hindi", "telugu", "tamil", "english"
}

#[derive(Debug, Serialize, Deserialize)]
struct ParseResponse {
    action: String,
    source: Option<String>,
    target: Option<String>,
    amount: Option<f64>,
    raw: String,
    confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteRequest {
    intent: ParseResponse,
    wallet_address: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExecuteResponse {
    success: bool,
    transaction_id: Option<String>,
    error: Option<String>,
}

// Simple confidence calculation based on parsing success
fn calculate_confidence(intent: &vani_mcp::vanicommand::Intent) -> f32 {
    if intent.action == "unknown" {
        0.3
    } else if intent.amount.is_none() && (intent.action == "swap" || intent.action == "send") {
        0.6
    } else {
        0.95
    }
}

async fn parse_command(Json(req): Json<ParseRequest>) -> Result<Json<ParseResponse>, StatusCode> {
    // Use the real vernacular parser from vani-mcp
    let intent = vanicommand::parse(&req.text);
    let confidence = calculate_confidence(&intent);
    
    let response = ParseResponse {
        action: intent.action,
        source: intent.source,
        target: intent.target,
        amount: intent.amount,
        confidence,
        raw: req.text,
    };
    
    Ok(Json(response))
}

async fn execute_transaction(Json(req): Json<ExecuteRequest>) -> Result<Json<ExecuteResponse>, StatusCode> {
    // TODO: Integrate with vani-mcp's execute module
    // For now, return a mock response that simulates execution
    // Real execution would require Turnkey configuration and wallet setup
    
    let success = req.intent.action != "unknown";
    
    let response = ExecuteResponse {
        success,
        transaction_id: if success {
            Some(format!("tx_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()))
        } else {
            None
        },
        error: if !success {
            Some("Cannot execute unknown action".to_string())
        } else {
            None
        }
    };
    
    Ok(Json(response))
}

async fn health() -> &'static str {
    "Vani API is running"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vani_api=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/parse", post(parse_command))
        .route("/api/execute", post(execute_transaction))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Vani API listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;
    
    Ok(())
}