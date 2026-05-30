//! R.A.T background agent daemon (tray companion / headless session host).

use anyhow::Result;
use rat_agent::{AgentConfig, Role};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    let role = if args.iter().any(|a| a == "--joiner") {
        Role::Joiner
    } else {
        Role::Admin
    };

    let signaling = args
        .iter()
        .position(|a| a == "--signaling")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "ws://127.0.0.1:4899".into());

    let code = args
        .iter()
        .position(|a| a == "--code")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let (_handle, mut events) = rat_agent::session::spawn_agent(AgentConfig {
        role,
        signaling_url: signaling,
        session_code: code,
    })
    .await?;

    while let Some(ev) = events.recv().await {
        tracing::info!(?ev, "agent event");
    }
    Ok(())
}
