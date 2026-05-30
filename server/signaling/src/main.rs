//! R.A.T signaling server — ports room registry from legacy server.js.

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use rat_protocol::{
    generate_session_code, SignalingMessage, DEFAULT_SIGNALING_PORT, SESSION_CODE_TTL_SECS,
};
use std::{net::SocketAddr, sync::Arc, time::{Duration, Instant}};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Clone)]
struct Room {
    admin: Uuid,
    joiner: Option<Uuid>,
    created: Instant,
}

#[derive(Clone)]
struct RegisteredDevice {
    peer: Uuid,
    name: String,
}

#[derive(Clone)]
struct AppState {
    rooms: Arc<DashMap<String, Room>>,
    peers: Arc<DashMap<Uuid, mpsc::UnboundedSender<String>>>,
    devices: Arc<DashMap<String, RegisteredDevice>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("rat_signaling=info".parse()?))
        .init();

    let state = AppState {
        rooms: Arc::new(DashMap::new()),
        peers: Arc::new(DashMap::new()),
        devices: Arc::new(DashMap::new()),
    };

    let rooms_clean = state.rooms.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let now = Instant::now();
            rooms_clean.retain(|code, room| {
                let expired = now.duration_since(room.created).as_secs() > SESSION_CODE_TTL_SECS;
                if expired {
                    warn!("expired session {code}");
                }
                !expired
            });
        }
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_SIGNALING_PORT));
    info!("R.A.T signaling listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let peer_id = Uuid::new_v4();
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    state.peers.insert(peer_id, tx);

    let forward = tokio::spawn(async move {
        while let Some(json) = rx.recv().await {
            if sink.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(Message::Text(text))) = stream.next().await {
        if let Ok(msg) = serde_json::from_str::<SignalingMessage>(&text) {
            process_message(peer_id, msg, &state);
        }
    }

    cleanup_peer(peer_id, &state);
    forward.abort();
    let _ = state.peers.remove(&peer_id);
}

fn send_to(peer: Uuid, msg: &SignalingMessage, state: &AppState) {
    if let Ok(json) = serde_json::to_string(msg) {
        if let Some(tx) = state.peers.get(&peer) {
            let _ = tx.send(json);
        }
    }
}

fn broadcast_room(code: &str, msg: &SignalingMessage, state: &AppState, exclude: Option<Uuid>) {
    if let Some(room) = state.rooms.get(code) {
        if exclude != Some(room.admin) {
            send_to(room.admin, msg, state);
        }
        if let Some(j) = room.joiner {
            if exclude != Some(j) {
                send_to(j, msg, state);
            }
        }
    }
}

fn process_message(peer_id: Uuid, msg: SignalingMessage, state: &AppState) {
    match msg {
        SignalingMessage::CreateSession => {
            let code = generate_session_code();
            state.rooms.insert(
                code.clone(),
                Room {
                    admin: peer_id,
                    joiner: None,
                    created: Instant::now(),
                },
            );
            send_to(
                peer_id,
                &SignalingMessage::SessionCreated {
                    session_code: code.clone(),
                },
                state,
            );
            info!("session created: {code} admin={peer_id}");
        }
        SignalingMessage::JoinSession { session_code } => {
            let Some(mut room) = state.rooms.get_mut(&session_code) else {
                send_to(
                    peer_id,
                    &SignalingMessage::LinkError {
                        message: "Invalid or expired connection code.".into(),
                    },
                    state,
                );
                return;
            };
            if room.created.elapsed().as_secs() > SESSION_CODE_TTL_SECS {
                state.rooms.remove(&session_code);
                send_to(
                    peer_id,
                    &SignalingMessage::LinkError {
                        message: "Invalid or expired connection code.".into(),
                    },
                    state,
                );
                return;
            }
            if room.joiner.is_some() {
                send_to(
                    peer_id,
                    &SignalingMessage::LinkError {
                        message: "This session is already in use by another client.".into(),
                    },
                    state,
                );
                return;
            }
            let admin_id = room.admin;
            room.joiner = Some(peer_id);
            drop(room);
            send_to(
                admin_id,
                &SignalingMessage::SessionLinked {
                    session_code: session_code.clone(),
                },
                state,
            );
            send_to(
                peer_id,
                &SignalingMessage::SessionLinked {
                    session_code: session_code.clone(),
                },
                state,
            );
            send_to(
                peer_id,
                &SignalingMessage::ConsentRequest {
                    admin_label: "Remote Admin".into(),
                },
                state,
            );
            info!("joiner {peer_id} linked to {session_code}");
        }
        SignalingMessage::RegisterDevice { device_id, name } => {
            state.devices.insert(
                device_id.clone(),
                RegisteredDevice {
                    peer: peer_id,
                    name: name.clone(),
                },
            );
            let id_log = device_id.clone();
            send_to(
                peer_id,
                &SignalingMessage::DeviceRegistered { device_id },
                state,
            );
            info!("device registered: {id_log} ({name})");
        }
        SignalingMessage::LookupDevice { device_id } => {
            let found = state.devices.get(&device_id);
            send_to(
                peer_id,
                &SignalingMessage::DeviceLookupRes {
                    found: found.is_some(),
                    device_name: found.map(|d| d.name.clone()),
                },
                state,
            );
        }
        SignalingMessage::Signal { session_code, data } => {
            let msg = SignalingMessage::Signal {
                session_code: session_code.clone(),
                data,
            };
            broadcast_room(&session_code, &msg, state, Some(peer_id));
        }
        SignalingMessage::ControlAction {
            session_code,
            action,
        } => {
            let msg = SignalingMessage::ControlAction {
                session_code: session_code.clone(),
                action,
            };
            broadcast_room(&session_code, &msg, state, Some(peer_id));
        }
        SignalingMessage::MediaFrame { .. }
        | SignalingMessage::KeyExchange { .. }
        | SignalingMessage::KeyExchangeAck { .. }
        | SignalingMessage::ConsentResponse { .. } => {
            if let Some(code) = find_room_for_peer(peer_id, state) {
                broadcast_room(&code, &msg, state, Some(peer_id));
            }
        }
        SignalingMessage::Ping => {
            send_to(peer_id, &SignalingMessage::Pong, state);
        }
        _ => {}
    }
}

fn find_room_for_peer(peer_id: Uuid, state: &AppState) -> Option<String> {
    for entry in state.rooms.iter() {
        let r = entry.value();
        if r.admin == peer_id || r.joiner == Some(peer_id) {
            return Some(entry.key().clone());
        }
    }
    None
}

fn cleanup_peer(peer_id: Uuid, state: &AppState) {
    state.devices.retain(|_, d| d.peer != peer_id);

    let mut dead_code = None;
    for entry in state.rooms.iter() {
        let r = entry.value();
        if r.admin == peer_id || r.joiner == Some(peer_id) {
            dead_code = Some(entry.key().clone());
            break;
        }
    }
    if let Some(code) = dead_code {
        state.rooms.remove(&code);
        broadcast_room(
            &code,
            &SignalingMessage::SessionTerminated {
                reason: "Peer disconnected".into(),
            },
            state,
            Some(peer_id),
        );
        warn!("session {code} terminated (peer {peer_id} left)");
    }
}
