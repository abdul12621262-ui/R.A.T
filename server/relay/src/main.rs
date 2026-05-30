//! UDP media relay for NAT traversal (TURN-lite MVP).

use anyhow::Result;
use rat_protocol::DEFAULT_RELAY_PORT;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let socket = UdpSocket::bind(("0.0.0.0", DEFAULT_RELAY_PORT)).await?;
    info!("R.A.T relay listening on UDP {}", DEFAULT_RELAY_PORT);

    let sessions: Arc<Mutex<HashMap<String, Vec<SocketAddr>>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut buf = vec![0u8; 65507];

    loop {
        let (len, peer) = socket.recv_from(&mut buf).await?;
        if len < 8 {
            continue;
        }
        let session_id = String::from_utf8_lossy(&buf[..8]).to_string();
        let payload = &buf[8..len];

        let mut map = sessions.lock().await;
        let peers = map.entry(session_id).or_default();
        if !peers.contains(&peer) {
            peers.push(peer);
            info!("relay peer joined session, count={}", peers.len());
        }
        let targets: Vec<SocketAddr> = peers.iter().filter(|&&p| p != peer).copied().collect();
        drop(map);

        for target in targets {
            if let Err(e) = socket.send_to(payload, target).await {
                warn!("relay forward failed: {e}");
            }
        }
    }
}
