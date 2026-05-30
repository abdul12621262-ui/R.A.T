//! R.A.T native desktop UI (Slint, no Electron).

slint::include_modules!();

mod agent_bridge;

use agent_bridge::UiAgent;
use anyhow::Result;
use slint::{Image, Rgb8Pixel, SharedPixelBuffer};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let rt = tokio::runtime::Runtime::new()?;
    let handle = rt.handle().clone();

    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();
    let agent: Arc<Mutex<UiAgent>> = Arc::new(Mutex::new(UiAgent::new()));
    let remote_screen: Arc<StdMutex<(u32, u32)>> = Arc::new(StdMutex::new((1920, 1080)));

    ui.on_create_session({
        let agent = agent.clone();
        let handle = handle.clone();
        move || {
            let agent = agent.clone();
            handle.spawn(async move {
                if let Err(e) = agent.lock().await.start_admin().await {
                    tracing::error!("admin start: {e}");
                }
            });
        }
    });

    ui.on_join_session({
        let agent = agent.clone();
        let ui = ui_weak.clone();
        let handle = handle.clone();
        move || {
            let code = match ui.upgrade() {
                Some(u) => u.get_join_code().to_string(),
                None => return,
            };
            let agent = agent.clone();
            handle.spawn(async move {
                if let Err(e) = agent.lock().await.start_joiner(code).await {
                    tracing::error!("joiner start: {e}");
                }
            });
        }
    });

    ui.on_copy_code({
        let ui = ui_weak.clone();
        move || {
            if let Some(u) = ui.upgrade() {
                let code = u.get_session_code().to_string();
                let _ = arboard::Clipboard::new().and_then(|mut c| c.set_text(code));
            }
        }
    });

    ui.on_disconnect({
        let agent = agent.clone();
        let ui = ui_weak.clone();
        let handle = handle.clone();
        move || {
            let agent = agent.clone();
            handle.spawn(async move {
                agent.lock().await.disconnect();
            });
            if let Some(u) = ui.upgrade() {
                u.set_screen(0);
                u.set_status_text("Disconnected".into());
                u.set_remote_active(false);
            }
        }
    });

    ui.on_consent_accept({
        let agent = agent.clone();
        let ui = ui_weak.clone();
        let handle = handle.clone();
        move || {
            let agent = agent.clone();
            handle.spawn(async move {
                agent.lock().await.grant_consent(true);
            });
            if let Some(u) = ui.upgrade() {
                u.set_screen(3);
                u.set_status_text("Sharing screen — remote control active".into());
            }
        }
    });

    ui.on_consent_deny({
        let agent = agent.clone();
        let ui = ui_weak.clone();
        let handle = handle.clone();
        move || {
            let agent = agent.clone();
            handle.spawn(async move {
                let mut a = agent.lock().await;
                a.grant_consent(false);
                a.disconnect();
            });
            if let Some(u) = ui.upgrade() {
                u.set_screen(0);
            }
        }
    });

    ui.on_send_chat({
        let agent = agent.clone();
        let ui = ui_weak.clone();
        let handle = handle.clone();
        move || {
            if let Some(u) = ui.upgrade() {
                let text = u.get_chat_input().to_string();
                if !text.is_empty() {
                    let agent = agent.clone();
                    let text_c = text.clone();
                    handle.spawn(async move {
                        agent.lock().await.send_chat(&text_c);
                    });
                    let log = format!("{}\nYou: {}", u.get_chat_log(), text);
                    u.set_chat_log(log.into());
                    u.set_chat_input("".into());
                }
            }
        }
    });

    ui.on_send_terminal({
        let agent = agent.clone();
        let ui = ui_weak.clone();
        let handle = handle.clone();
        move || {
            if let Some(u) = ui.upgrade() {
                let cmd = u.get_terminal_input().to_string();
                if !cmd.is_empty() {
                    let agent = agent.clone();
                    handle.spawn(async move {
                        agent.lock().await.send_terminal(&cmd);
                    });
                    u.set_terminal_input("".into());
                }
            }
        }
    });

    ui.on_browse_files({
        let agent = agent.clone();
        let ui = ui_weak.clone();
        let handle = handle.clone();
        move || {
            if let Some(u) = ui.upgrade() {
                let path = u.get_file_path().to_string();
                let agent = agent.clone();
                handle.spawn(async move {
                    agent.lock().await.browse_files(&path);
                });
            }
        }
    });

    ui.on_refresh_stats({
        let agent = agent.clone();
        let handle = handle.clone();
        move || {
            let agent = agent.clone();
            handle.spawn(async move {
                agent.lock().await.refresh_stats();
            });
        }
    });

    ui.on_mouse_move({
        let agent = agent.clone();
        let handle = handle.clone();
        let remote_screen = remote_screen.clone();
        move |x, y| {
            let (rw, rh) = *remote_screen.lock().unwrap();
            let agent = agent.clone();
            // TODO: Get actual viewport dimensions from Slint for perfect scaling.
            // For now, assume viewport is ~1920x1080 (or scale gracefully).
            // True scaling: (x / viewport_width) * remote_width, (y / viewport_height) * remote_height
            let scaled_x = if rw > 0 { ((x as u32 * rw) / 1920).max(0) as i32 } else { x as i32 };
            let scaled_y = if rh > 0 { ((y as u32 * rh) / 1080).max(0) as i32 } else { y as i32 };
            handle.spawn(async move {
                agent.lock().await.mouse_move(scaled_x, scaled_y);
            });
        }
    });

    ui.on_mouse_click({
        let agent = agent.clone();
        let handle = handle.clone();
        move |btn, dbl| {
            let agent = agent.clone();
            let btn = btn.to_string();
            handle.spawn(async move {
                agent.lock().await.mouse_click(&btn, dbl);
            });
        }
    });

    let ui_poll = ui_weak.clone();
    let agent_poll = agent.clone();
    let remote_poll = remote_screen.clone();
    std::thread::spawn(move || {
        rt.block_on(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let ev = agent_poll.lock().await.poll_event();
                if let Some(ev) = ev {
                    let ui = ui_poll.clone();
                    let remote_poll = remote_poll.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(u) = ui.upgrade() {
                            apply_event(&u, ev, &remote_poll);
                        }
                    });
                }
            }
        });
    });

    ui.run()?;
    Ok(())
}

fn apply_event(ui: &AppWindow, ev: rat_agent::AgentEvent, remote_screen: &StdMutex<(u32, u32)>) {
    use rat_agent::AgentEvent;
    match ev {
        AgentEvent::ScreenResolution { width, height } => {
            *remote_screen.lock().unwrap() = (width, height);
        }
        AgentEvent::SessionCreated { code } => {
            ui.set_session_code(code.into());
            ui.set_screen(1);
            ui.set_status_text("Waiting for joiner...".into());
        }
        AgentEvent::SessionLinked { code } => {
            ui.set_session_code(code.into());
            ui.set_screen(2);
            ui.set_status_text("Connected".into());
            ui.set_remote_active(false);
        }
        AgentEvent::LinkError { message } => {
            ui.set_status_text(message.into());
            ui.set_screen(0);
        }
        AgentEvent::FrameReceived { jpeg, .. } => {
            if let Some(img) = jpeg_to_slint_image(&jpeg) {
                ui.set_remote_frame(img);
                ui.set_remote_active(true);
            }
        }
        AgentEvent::ConsentRequired => {
            ui.set_screen(4);
        }
        AgentEvent::SessionEnded { reason } => {
            ui.set_status_text(reason.into());
            ui.set_screen(0);
            ui.set_remote_active(false);
        }
        AgentEvent::Chat { text, from } => {
            let log = format!("{}\n{}: {}", ui.get_chat_log(), from, text);
            ui.set_chat_log(log.into());
        }
        AgentEvent::TerminalData { chunk } => {
            let log = format!("{}{}", ui.get_terminal_log(), chunk);
            ui.set_terminal_log(log.into());
        }
        AgentEvent::TerminalEnd => {
            let log = format!("{}\n---\n", ui.get_terminal_log());
            ui.set_terminal_log(log.into());
        }
        AgentEvent::FileBrowseRes { path, items, error } => {
            if let Some(e) = error {
                ui.set_file_listing(format!("Error: {e}").into());
            } else {
                let lines: Vec<String> = items
                    .iter()
                    .map(|i| {
                        let tag = if i.is_directory { "[DIR]" } else { "[FILE]" };
                        format!("{tag} {}  {}", i.name, i.path)
                    })
                    .collect();
                ui.set_file_path(path.into());
                ui.set_file_listing(lines.join("\n").into());
            }
        }
        AgentEvent::FileReadRes { path, content, error } => {
            if let Some(e) = error {
                ui.set_file_listing(format!("Read error: {e}").into());
            } else if let Some(c) = content {
                ui.set_file_listing(format!("--- {path} ---\n{c}").into());
            }
        }
        AgentEvent::SystemStatsRes { stats } => {
            ui.set_stats_text(
                format!(
                    "Host: {}\nOS: {}\nCPU: {}%\nMemory: {}%\nUptime: {}s\nRAM: {} / {} MB",
                    stats.hostname,
                    stats.os_type,
                    stats.cpu_usage,
                    stats.memory_usage,
                    stats.uptime_secs,
                    (stats.total_memory - stats.free_memory) / 1_048_576,
                    stats.total_memory / 1_048_576,
                )
                .into(),
            );
        }
    }
}

pub fn jpeg_to_slint_image(jpeg: &[u8]) -> Option<Image> {
    let img = image::ImageReader::new(std::io::Cursor::new(jpeg))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let buf = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(rgb.as_raw(), w, h);
    Some(Image::from_rgb8(buf))
}
