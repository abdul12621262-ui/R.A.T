//! Agent session orchestration.

use crate::fs_ops;
use anyhow::Result;
use rat_capture::{create_capture, CapturedFrame, ScreenCapture};
use rat_crypto::SessionCrypto;
use rat_input::{create_injector, InputInjector};
use rat_protocol::{ControlAction, SignalingMessage, SystemStats};
use rat_transport::SignalingClient;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::sync::{mpsc, Mutex};
use tracing::error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Joiner,
}

pub struct AgentConfig {
    pub role: Role,
    pub signaling_url: String,
    pub session_code: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    SessionCreated { code: String },
    SessionLinked { code: String },
    LinkError { message: String },
    FrameReceived { jpeg: Vec<u8>, width: u32, height: u32 },
    ConsentRequired,
    SessionEnded { reason: String },
    Chat { text: String, from: String },
    TerminalData { chunk: String },
    TerminalEnd,
    FileBrowseRes {
        path: String,
        items: Vec<rat_protocol::FileEntry>,
        error: Option<String>,
    },
    FileReadRes {
        path: String,
        content: Option<String>,
        error: Option<String>,
    },
    SystemStatsRes { stats: SystemStats },
    ScreenResolution { width: u32, height: u32 },
}

pub struct AgentHandle {
    cmd_tx: mpsc::UnboundedSender<AgentCommand>,
}

#[derive(Debug)]
enum AgentCommand {
    CreateSession,
    JoinSession { code: String },
    ConsentResponse { accepted: bool },
    SendControl(ControlAction),
    Disconnect,
}

struct SharedState {
    role: Role,
    code: Option<String>,
    client: Option<Arc<SignalingClient>>,
    capture: Option<Box<dyn ScreenCapture>>,
    injector: Option<Box<dyn InputInjector>>,
    crypto: Option<Arc<Mutex<SessionCrypto>>>,
    crypto_ready: bool,
    _crypto_initiator: bool,
    consent: bool,
    capture_running: bool,
}

pub async fn spawn_agent(
    config: AgentConfig,
) -> Result<(AgentHandle, mpsc::UnboundedReceiver<AgentEvent>)> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();

    let state = Arc::new(Mutex::new(SharedState {
        role: config.role,
        code: config.session_code.clone(),
        client: None,
        capture: None,
        injector: None,
        crypto: None,
        crypto_ready: false,
        _crypto_initiator: config.role == Role::Admin,
        consent: config.role == Role::Admin,
        capture_running: false,
    }));

    let event_tx_msg = event_tx.clone();
    let state_msg = state.clone();
    let on_msg: Arc<dyn Fn(SignalingMessage) + Send + Sync> = Arc::new(move |msg| {
        let tx = event_tx_msg.clone();
        let st = state_msg.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_incoming(tx, st, msg).await {
                error!("handle message: {e}");
            }
        });
    });

    let client = Arc::new(SignalingClient::connect(&config.signaling_url, on_msg).await?);
    {
        let mut s = state.lock().await;
        s.client = Some(client.clone());
    }

    let state_task = state.clone();
    let event_task = event_tx.clone();
    tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                AgentCommand::CreateSession => {
                    client.send(SignalingMessage::CreateSession);
                }
                AgentCommand::JoinSession { code } => {
                    {
                        let mut s = state_task.lock().await;
                        s.code = Some(code.clone());
                    }
                    client.send(SignalingMessage::JoinSession {
                        session_code: code,
                    });
                }
                AgentCommand::ConsentResponse { accepted } => {
                    let mut s = state_task.lock().await;
                    s.consent = accepted;
                    if accepted {
                        if s.capture.is_none() {
                            s.capture = create_capture().ok();
                        }
                        if s.injector.is_none() {
                            s.injector = create_injector().ok();
                        }
                        try_start_capture(&mut s, client.clone());
                    } else {
                        client.send(SignalingMessage::SessionTerminated {
                            reason: "Consent denied".into(),
                        });
                    }
                }
                AgentCommand::SendControl(action) => {
                    let s = state_task.lock().await;
                    if let Some(code) = &s.code {
                        client.send(SignalingMessage::ControlAction {
                            session_code: code.clone(),
                            action,
                        });
                    }
                }
                AgentCommand::Disconnect => {
                    let s = state_task.lock().await;
                    if let Some(code) = &s.code {
                        client.send(SignalingMessage::ControlAction {
                            session_code: code.clone(),
                            action: ControlAction::Disconnect,
                        });
                    }
                    let _ = event_task.send(AgentEvent::SessionEnded {
                        reason: "Disconnected".into(),
                    });
                }
            }
        }
    });

    let handle = AgentHandle { cmd_tx };

    if config.role == Role::Admin {
        handle.create_session();
    } else if let Some(code) = config.session_code {
        handle.join_session(code);
    }

    Ok((handle, event_rx))
}

async fn begin_key_exchange(state: &Arc<Mutex<SharedState>>, client: &Arc<SignalingClient>) {
    let mut s = state.lock().await;
    if s.crypto.is_some() {
        return;
    }
    let (crypto, pubkey) = SessionCrypto::initiator();
    s.crypto = Some(Arc::new(Mutex::new(crypto)));
    client.send(SignalingMessage::KeyExchange { public_key: pubkey });
    let _ = &mut s;
}

async fn handle_incoming(
    tx: mpsc::UnboundedSender<AgentEvent>,
    state: Arc<Mutex<SharedState>>,
    msg: SignalingMessage,
) -> Result<()> {
    match msg {
        SignalingMessage::SessionCreated { session_code } => {
            let mut s = state.lock().await;
            s.code = Some(session_code.clone());
            let _ = tx.send(AgentEvent::SessionCreated { code: session_code });
        }
        SignalingMessage::SessionLinked { session_code } => {
            {
                let mut s = state.lock().await;
                s.code = Some(session_code.clone());
            }
            let _ = tx.send(AgentEvent::SessionLinked {
                code: session_code.clone(),
            });
            let s = state.lock().await;
            if s.role == Role::Admin {
                if let Some(cl) = s.client.clone() {
                    drop(s);
                    begin_key_exchange(&state, &cl).await;
                }
            }
        }
        SignalingMessage::LinkError { message } => {
            let _ = tx.send(AgentEvent::LinkError { message });
        }
        SignalingMessage::ConsentRequest { .. } => {
            let _ = tx.send(AgentEvent::ConsentRequired);
        }
        SignalingMessage::KeyExchange { public_key } => {
            let mut s = state.lock().await;
            if let Ok((crypto, ack)) = SessionCrypto::responder(&public_key) {
                s.crypto = Some(Arc::new(Mutex::new(crypto)));
                s.crypto_ready = true;
                if let Some(cl) = s.client.clone() {
                    cl.send(SignalingMessage::KeyExchangeAck { public_key: ack });
                    try_start_capture(&mut s, cl);
                }
            }
        }
        SignalingMessage::KeyExchangeAck { public_key } => {
            let mut s = state.lock().await;
            if let Some(crypto) = &s.crypto {
                let _ = crypto.lock().await.complete_handshake(&public_key);
                s.crypto_ready = true;
            }
            if let Some(cl) = s.client.clone() {
                try_start_capture(&mut s, cl);
            }
        }
        SignalingMessage::MediaFrame {
            frame,
            width,
            height,
            ..
        } => {
            let jpeg = if let Some(crypto) = &state.lock().await.crypto {
                crypto.lock().await.decrypt(&frame).unwrap_or(frame)
            } else {
                frame
            };
            let _ = tx.send(AgentEvent::FrameReceived {
                jpeg,
                width,
                height,
            });
        }
        SignalingMessage::ControlAction { action, .. } => {
            handle_control(&tx, &state, action).await?;
        }
        SignalingMessage::SessionTerminated { reason } => {
            let _ = tx.send(AgentEvent::SessionEnded { reason });
        }
        _ => {}
    }
    Ok(())
}

async fn handle_control(
    tx: &mpsc::UnboundedSender<AgentEvent>,
    state: &Arc<Mutex<SharedState>>,
    action: ControlAction,
) -> Result<()> {
    match &action {
        ControlAction::FileBrowseRes { path, items, error } => {
            let _ = tx.send(AgentEvent::FileBrowseRes {
                path: path.clone(),
                items: items.clone(),
                error: error.clone(),
            });
            return Ok(());
        }
        ControlAction::FileReadRes { path, content, error } => {
            let _ = tx.send(AgentEvent::FileReadRes {
                path: path.clone(),
                content: content.clone(),
                error: error.clone(),
            });
            return Ok(());
        }
        ControlAction::TerminalData { chunk } => {
            let _ = tx.send(AgentEvent::TerminalData {
                chunk: chunk.clone(),
            });
            return Ok(());
        }
        ControlAction::TerminalEnd => {
            let _ = tx.send(AgentEvent::TerminalEnd);
            return Ok(());
        }
        ControlAction::SystemStatsRes { stats } => {
            let _ = tx.send(AgentEvent::SystemStatsRes {
                stats: stats.clone(),
            });
            return Ok(());
        }
        ControlAction::ChatMessage { text, from } => {
            let _ = tx.send(AgentEvent::Chat {
                text: text.clone(),
                from: from.clone(),
            });
            return Ok(());
        }
        ControlAction::ScreenResolution { width, height } => {
            let _ = tx.send(AgentEvent::ScreenResolution {
                width: *width,
                height: *height,
            });
            return Ok(());
        }
        _ => {}
    }

    let mut s = state.lock().await;
    if !s.consent {
        return Ok(());
    }

    match action {
        ControlAction::MouseMove { .. }
        | ControlAction::MouseClick { .. }
        | ControlAction::TypeText { .. }
        | ControlAction::PressKey { .. } => {
            if s.injector.is_none() {
                s.injector = create_injector().ok();
            }
            if let Some(inj) = s.injector.as_mut() {
                inj.apply(&action)?;
            }
        }
        ControlAction::FileBrowse { path } => {
            let code = s.code.clone();
            let cl = s.client.clone();
            let res = fs_ops::browse(&path);
            if let (Some(code), Some(cl)) = (code, cl) {
                let (items, error) = match res {
                    Ok(items) => (items, None),
                    Err(e) => (vec![], Some(e.to_string())),
                };
                cl.send(SignalingMessage::ControlAction {
                    session_code: code,
                    action: ControlAction::FileBrowseRes {
                        path,
                        items,
                        error,
                    },
                });
            }
        }
        ControlAction::FileRead { path } => {
            let code = s.code.clone();
            let cl = s.client.clone();
            let res = fs_ops::read_file(&path);
            if let (Some(code), Some(cl)) = (code, cl) {
                let (content, error) = match res {
                    Ok(c) => (Some(c), None),
                    Err(e) => (None, Some(e.to_string())),
                };
                cl.send(SignalingMessage::ControlAction {
                    session_code: code,
                    action: ControlAction::FileReadRes {
                        path,
                        content,
                        error,
                    },
                });
            }
        }
        ControlAction::FileWrite { path, content } => {
            let _ = fs_ops::write_file(&path, &content);
        }
        ControlAction::ClipboardSet { text } => {
            let _ = arboard::Clipboard::new().and_then(|mut c| c.set_text(text));
        }
        ControlAction::ClipboardGet => {
            if let (Some(code), Some(cl)) = (s.code.clone(), s.client.clone()) {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    if let Ok(text) = cb.get_text() {
                        cl.send(SignalingMessage::ControlAction {
                            session_code: code,
                            action: ControlAction::ClipboardData { text },
                        });
                    }
                }
            }
        }
        ControlAction::SystemStatsRequest => {
            let stats = system_stats();
            if let (Some(code), Some(cl)) = (s.code.clone(), s.client.clone()) {
                cl.send(SignalingMessage::ControlAction {
                    session_code: code,
                    action: ControlAction::SystemStatsRes { stats },
                });
            }
        }
        ControlAction::TerminalCmd { command } => {
            let code = s.code.clone();
            let cl = s.client.clone();
            drop(s);
            if let (Some(code), Some(cl)) = (code, cl) {
                run_shell_command(&cl, &code, &command).await;
            }
            return Ok(());
        }
        _ => {}
    }
    Ok(())
}

fn try_start_capture(s: &mut SharedState, client: Arc<SignalingClient>) {
    if s.role != Role::Joiner || !s.consent || !s.crypto_ready || s.capture_running {
        return;
    }
    s.capture_running = true;
    let cap = s.capture.take();
    let code = s.code.clone();
    let crypto = s.crypto.clone();
    if let (Some(mut capture), Some(code)) = (cap, code) {
        tokio::spawn(async move {
            run_capture_loop(client, code, crypto, &mut capture).await;
        });
    }
}

async fn run_shell_command(client: &SignalingClient, code: &str, command: &str) {
    #[cfg(windows)]
    let output = tokio::process::Command::new("cmd")
        .args(["/C", command])
        .output()
        .await;
    #[cfg(not(windows))]
    let output = tokio::process::Command::new("sh")
        .args(["-c", command])
        .output()
        .await;
    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout).to_string()
            + &String::from_utf8_lossy(&out.stderr);
        client.send(SignalingMessage::ControlAction {
            session_code: code.to_string(),
            action: ControlAction::TerminalData { chunk: text },
        });
        client.send(SignalingMessage::ControlAction {
            session_code: code.to_string(),
            action: ControlAction::TerminalEnd,
        });
    }
}

async fn run_capture_loop(
    client: Arc<SignalingClient>,
    code: String,
    crypto: Option<Arc<Mutex<SessionCrypto>>>,
    capture: &mut Box<dyn ScreenCapture>,
) {
    if let Ok(frame) = capture.capture_frame() {
        client.send(SignalingMessage::ControlAction {
            session_code: code.clone(),
            action: ControlAction::ScreenResolution {
                width: frame.width,
                height: frame.height,
            },
        });
    }
    let mut seq = 0u64;
    loop {
        tokio::time::sleep(Duration::from_millis(66)).await;
        match capture.capture_frame() {
            Ok(CapturedFrame {
                width,
                height,
                jpeg,
            }) => {
                let frame = if let Some(c) = &crypto {
                    c.lock().await.encrypt(&jpeg)
                } else {
                    jpeg
                };
                client.send(SignalingMessage::MediaFrame {
                    session_code: code.clone(),
                    frame,
                    width,
                    height,
                    sequence: seq,
                });
                seq += 1;
            }
            Err(e) => {
                error!("capture: {e}");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

impl AgentHandle {
    pub fn create_session(&self) {
        let _ = self.cmd_tx.send(AgentCommand::CreateSession);
    }

    pub fn join_session(&self, code: String) {
        let _ = self.cmd_tx.send(AgentCommand::JoinSession { code });
    }

    pub fn grant_consent(&self, accepted: bool) {
        let _ = self.cmd_tx.send(AgentCommand::ConsentResponse { accepted });
    }

    pub fn send_control(&self, action: ControlAction) {
        let _ = self.cmd_tx.send(AgentCommand::SendControl(action));
    }

    pub fn disconnect(&self) {
        let _ = self.cmd_tx.send(AgentCommand::Disconnect);
    }
}

pub fn system_stats() -> SystemStats {
    let mut sys = System::new_all();
    sys.refresh_all();
    let total = sys.total_memory();
    let used = sys.used_memory();
    SystemStats {
        os_type: System::name().unwrap_or_else(|| std::env::consts::OS.into()),
        hostname: System::host_name().unwrap_or_else(|| "unknown".into()),
        cpu_usage: sys.global_cpu_usage() as u32,
        memory_usage: if total > 0 {
            ((used * 100) / total) as u32
        } else {
            0
        },
        uptime_secs: System::uptime(),
        total_memory: total,
        free_memory: sys.free_memory(),
    }
}
