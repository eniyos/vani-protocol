//! Vani Protocol — Rust-native MCP server for vernacular Solana DeFi.
//!
//! Speaks MCP over stdio. Tools are defined in [`server`].

mod config;
mod jupiter;
mod rpc;
mod server;
mod vanicommand;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("Vani MCP server starting over stdio");

    let handler = server::VaniServer::from_env()?;
    let service = handler.serve(stdio()).await?;

    tracing::info!("Vani MCP server ready — connect an MCP client (Claude Code, Cursor) to stdio");
    service.waiting().await?;

    Ok(())
}