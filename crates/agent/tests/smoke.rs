use anyhow::Result;
use rat_agent::{AgentConfig, AgentEvent, Role};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_full_lan_smoke_flow() -> Result<()> {
    // 1. Start the Admin session. Connect to our signaling server running on port 4899.
    let signaling_url = "ws://127.0.0.1:4899".to_string();
    println!("Connecting admin to signaling server...");
    
    let (admin_handle, mut admin_rx) = rat_agent::session::spawn_agent(AgentConfig {
        role: Role::Admin,
        signaling_url: signaling_url.clone(),
        session_code: None,
    })
    .await?;

    // 2. Wait for SessionCreated event on Admin side to get the connection code.
    println!("Waiting for session creation on Admin...");
    let code = match timeout(Duration::from_secs(5), admin_rx.recv()).await? {
        Some(AgentEvent::SessionCreated { code }) => {
            println!("Admin SessionCreated successfully with code: {}", code);
            code
        }
        other => panic!("Expected SessionCreated event, got: {:?}", other),
    };

    // 3. Start the Joiner session using the session code.
    println!("Connecting joiner with code {}...", code);
    let (joiner_handle, mut joiner_rx) = rat_agent::session::spawn_agent(AgentConfig {
        role: Role::Joiner,
        signaling_url: signaling_url.clone(),
        session_code: Some(code.clone()),
    })
    .await?;

    // 4. Wait for SessionLinked on both sides.
    println!("Waiting for SessionLinked on Joiner...");
    match timeout(Duration::from_secs(5), joiner_rx.recv()).await? {
        Some(AgentEvent::SessionLinked { code: joiner_code }) => {
            assert_eq!(joiner_code, code);
            println!("Joiner SessionLinked successfully.");
        }
        other => panic!("Expected SessionLinked on Joiner, got: {:?}", other),
    }

    println!("Waiting for SessionLinked on Admin...");
    match timeout(Duration::from_secs(5), admin_rx.recv()).await? {
        Some(AgentEvent::SessionLinked { code: admin_code }) => {
            assert_eq!(admin_code, code);
            println!("Admin SessionLinked successfully.");
        }
        other => panic!("Expected SessionLinked on Admin, got: {:?}", other),
    }

    // 5. Wait for ConsentRequired on Joiner side, and accept it.
    println!("Waiting for ConsentRequired on Joiner...");
    match timeout(Duration::from_secs(5), joiner_rx.recv()).await? {
        Some(AgentEvent::ConsentRequired) => {
            println!("ConsentRequired received on Joiner. Granting consent...");
            joiner_handle.grant_consent(true);
        }
        other => panic!("Expected ConsentRequired on Joiner, got: {:?}", other),
    }

    // 6. Wait for cryptographic handshake to complete and capture to send first frame.
    // The Joiner should start capture, send the ScreenResolution, and then send MediaFrames.
    // The Admin should receive ScreenResolution first.
    println!("Waiting for ScreenResolution on Admin...");
    match timeout(Duration::from_secs(5), admin_rx.recv()).await? {
        Some(AgentEvent::ScreenResolution { width, height }) => {
            println!("Admin received ScreenResolution: {}x{}", width, height);
            assert!(width > 0);
            assert!(height > 0);
        }
        other => panic!("Expected ScreenResolution on Admin, got: {:?}", other),
    }

    // The Admin should receive at least one MediaFrame (FrameReceived).
    println!("Waiting for FrameReceived on Admin...");
    match timeout(Duration::from_secs(5), admin_rx.recv()).await? {
        Some(AgentEvent::FrameReceived { jpeg, width, height }) => {
            println!(
                "Admin FrameReceived successfully! Image bytes: {}, size: {}x{}",
                jpeg.len(),
                width,
                height
            );
            assert!(!jpeg.is_empty());
        }
        other => panic!("Expected FrameReceived on Admin, got: {:?}", other),
    }

    // 7. Verify control commands: Admin sends a chat message.
    println!("Admin sending chat message...");
    admin_handle.send_control(rat_protocol::ControlAction::ChatMessage {
        text: "Hello from admin!".to_string(),
        from: "Admin".to_string(),
    });

    println!("Waiting for Chat event on Joiner...");
    match timeout(Duration::from_secs(5), joiner_rx.recv()).await? {
        Some(AgentEvent::Chat { text, from }) => {
            println!("Joiner received chat: from={}, text={}", from, text);
            assert_eq!(from, "Admin");
            assert_eq!(text, "Hello from admin!");
        }
        other => panic!("Expected Chat on Joiner, got: {:?}", other),
    }

    // 8. Disconnect from Admin.
    println!("Admin disconnecting...");
    admin_handle.disconnect();

    // Verify SessionEnded on Joiner.
    println!("Waiting for SessionEnded on Joiner...");
    match timeout(Duration::from_secs(5), joiner_rx.recv()).await? {
        Some(AgentEvent::SessionEnded { reason }) => {
            println!("Joiner session ended. Reason: {}", reason);
        }
        other => panic!("Expected SessionEnded on Joiner, got: {:?}", other),
    }

    println!("All LAN smoke flow checks passed successfully!");
    Ok(())
}
