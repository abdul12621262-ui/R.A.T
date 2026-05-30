//! Bridges Slint UI to rat-agent session.

use anyhow::Result;
use rat_agent::{AgentConfig, AgentEvent, AgentHandle, Role};
use rat_protocol::ControlAction;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct UiAgent {
    inner: Arc<Mutex<UiAgentInner>>,
}

struct UiAgentInner {
    handle: Option<AgentHandle>,
    events: Option<mpsc::UnboundedReceiver<AgentEvent>>,
    pending: Vec<AgentEvent>,
}

impl UiAgent {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(UiAgentInner {
                handle: None,
                events: None,
                pending: Vec::new(),
            })),
        }
    }

    pub async fn start_admin(&mut self) -> Result<()> {
        let host = std::env::var("RAT_SIGNALING_HOST")
            .unwrap_or_else(|_| "your-signaling-server.com:4899".to_string());
        let url = normalize_url(&host);
        let (handle, mut rx) = rat_agent::session::spawn_agent(AgentConfig {
            role: Role::Admin,
            signaling_url: url,
            session_code: None,
        })
        .await?;
        let mut inner = self.inner.lock().unwrap();
        inner.handle = Some(handle);
        drain_events(&mut rx, &mut inner.pending);
        inner.events = Some(rx);
        Ok(())
    }

    pub async fn start_joiner(&mut self, code: String) -> Result<()> {
        let host = std::env::var("RAT_SIGNALING_HOST")
            .unwrap_or_else(|_| "your-signaling-server.com:4899".to_string());
        let url = normalize_url(&host);
        let (handle, mut rx) = rat_agent::session::spawn_agent(AgentConfig {
            role: Role::Joiner,
            signaling_url: url,
            session_code: Some(code),
        })
        .await?;
        let mut inner = self.inner.lock().unwrap();
        inner.handle = Some(handle);
        drain_events(&mut rx, &mut inner.pending);
        inner.events = Some(rx);
        Ok(())
    }

    pub fn grant_consent(&mut self, accepted: bool) {
        if let Some(h) = self.inner.lock().unwrap().handle.as_ref() {
            h.grant_consent(accepted);
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(h) = self.inner.lock().unwrap().handle.as_ref() {
            h.disconnect();
        }
        let mut inner = self.inner.lock().unwrap();
        inner.handle = None;
        inner.events = None;
        inner.pending.clear();
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.send_control(ControlAction::MouseMove { x, y });
    }

    pub fn mouse_click(&mut self, btn: &str, double: bool) {
        use rat_protocol::MouseButton;
        let button = match btn {
            "right" => MouseButton::Right,
            "middle" => MouseButton::Middle,
            _ => MouseButton::Left,
        };
        self.send_control(ControlAction::MouseClick {
            button,
            double_click: double,
        });
    }

    pub fn send_chat(&mut self, text: &str) {
        self.send_control(ControlAction::ChatMessage {
            text: text.to_string(),
            from: "admin".into(),
        });
    }

    pub fn send_terminal(&mut self, cmd: &str) {
        self.send_control(ControlAction::TerminalCmd {
            command: cmd.to_string(),
        });
    }

    pub fn browse_files(&mut self, path: &str) {
        self.send_control(ControlAction::FileBrowse {
            path: path.to_string(),
        });
    }

    pub fn refresh_stats(&mut self) {
        self.send_control(ControlAction::SystemStatsRequest);
    }

    fn send_control(&mut self, action: ControlAction) {
        if let Some(h) = self.inner.lock().unwrap().handle.as_ref() {
            h.send_control(action);
        }
    }

    pub fn poll_event(&self) -> Option<AgentEvent> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(rx) = &mut inner.events {
            let mut batch = Vec::new();
            while let Ok(ev) = rx.try_recv() {
                batch.push(ev);
            }
            inner.pending.extend(batch);
        }
        inner.pending.pop()
    }
}

fn drain_events(rx: &mut mpsc::UnboundedReceiver<AgentEvent>, pending: &mut Vec<AgentEvent>) {
    while let Ok(ev) = rx.try_recv() {
        pending.push(ev);
    }
}

fn normalize_url(host: &str) -> String {
    let h = host.trim();
    if h.starts_with("ws://") || h.starts_with("wss://") || h.starts_with("http") {
        h.to_string()
    } else {
        format!("ws://{h}")
    }
}
