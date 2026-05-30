//! WebSocket signaling client for R.A.T sessions.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use rat_protocol::SignalingMessage;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub type MessageHandler = Arc<dyn Fn(SignalingMessage) + Send + Sync>;

pub struct SignalingClient {
    outbound: mpsc::UnboundedSender<SignalingMessage>,
}

impl SignalingClient {
    pub async fn connect(url: &str, on_message: MessageHandler) -> Result<Self> {
        let ws_url = normalize_ws_url(url);
        let (ws, _) = connect_async(&ws_url)
            .await
            .with_context(|| format!("connect to {ws_url}"))?;
        let (write, mut read) = ws.split();
        let write = Arc::new(Mutex::new(write));
        let (tx, mut rx) = mpsc::unbounded_channel::<SignalingMessage>();

        let write_clone = write.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    let mut w = write_clone.lock().await;
                    if w.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
        });

        tokio::spawn(async move {
            while let Some(Ok(Message::Text(text))) = read.next().await {
                if let Ok(msg) = serde_json::from_str::<SignalingMessage>(&text) {
                    on_message(msg);
                }
            }
        });

        Ok(Self { outbound: tx })
    }

    pub fn send(&self, msg: SignalingMessage) {
        let _ = self.outbound.send(msg);
    }
}

fn normalize_ws_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
        return format!("{trimmed}/ws");
    }
    if trimmed.starts_with("http://") {
        return format!("{}/ws", trimmed.replacen("http://", "ws://", 1));
    }
    if trimmed.starts_with("https://") {
        return format!("{}/ws", trimmed.replacen("https://", "wss://", 1));
    }
    format!("ws://{trimmed}/ws")
}
