/**
 * AetherMod - FRONTEND CORE CONTROLLER
 * Orchestrates: room pairing, local loopback OS actions, WebRTC video feeds,
 * coordinate translation systems, terminal consoles, and physical folder browses.
 */

// Connection state nodes
let socketCentral = null; // Sockets connecting Admin and Client over the network
let socketLocal = null;   // Sockets connecting Client to its own local OS bridge server
let rtcPeer = null;       // WebRTC Peer connection
let localMediaStream = null;

// Session details
let myRole = ''; // 'admin' or 'client'
let activeSessionCode = '';
let activeTargetIP = '';

// Viewport mapping parameters (Synchronized from Client during handshakes)
let clientScreenResolution = { width: 1920, height: 1080 };
let isRemoteInteractionActive = true;
let isAnnotationModeActive = false;

// Shared Drawing Context (Annotations)
let isDrawing = false;
let lastDrawPoint = { x: 0, y: 0 };

// Cached physical path
let currentFilePath = 'C:\\';

// Document Elements
const el = {
  landing: document.getElementById('screen-landing'),
  admin: document.getElementById('screen-admin'),
  client: document.getElementById('screen-client'),

  btnCreateSession: document.getElementById('btn-create-session'),
  adminPairingHub: document.getElementById('admin-pairing-hub'),
  txtSessionCode: document.getElementById('txt-session-code'),
  btnCopyCode: document.getElementById('btn-copy-code'),

  btnJoinSession: document.getElementById('btn-join-session'),
  inputTargetIP: document.getElementById('input-target-ip'),
  inputSessionCode: document.getElementById('input-session-code'),
  txtLinkError: document.getElementById('txt-link-error'),

  adminPing: document.getElementById('admin-ping'),
  adminClientName: document.getElementById('admin-client-name'),
  adminCodeIndicator: document.getElementById('admin-code-indicator'),
  btnAdminDisconnect: document.getElementById('btn-admin-disconnect'),

  btnToggleControl: document.getElementById('btn-toggle-control'),
  btnAdminDraw: document.getElementById('btn-admin-draw'),
  btnAdminClearDraw: document.getElementById('btn-admin-clear-draw'),

  video: document.getElementById('remote-video'),
  canvas: document.getElementById('viewport-canvas'),
  virtualCursor: document.getElementById('admin-virtual-cursor'),
  viewportWrapper: document.getElementById('viewport-wrapper'),
  viewportFallback: document.getElementById('viewport-fallback'),

  terminalOutput: document.getElementById('terminal-output'),
  terminalInput: document.getElementById('terminal-input'),

  btnFileUp: document.getElementById('btn-file-up'),
  btnFileRefresh: document.getElementById('btn-file-refresh'),
  txtCurrentPath: document.getElementById('txt-current-path'),
  explorerFileList: document.getElementById('explorer-file-list'),
  explorerEditorBox: document.getElementById('explorer-editor-box'),
  editorFilename: document.getElementById('editor-filename'),
  btnSaveFile: document.getElementById('btn-save-file'),
  btnCloseEditor: document.getElementById('btn-close-editor'),
  explorerTextarea: document.getElementById('explorer-textarea'),

  gaugeCpuCircle: document.getElementById('gauge-cpu-circle'),
  gaugeCpuVal: document.getElementById('gauge-cpu-val'),
  gaugeMemCircle: document.getElementById('gauge-mem-circle'),
  gaugeMemVal: document.getElementById('gauge-mem-val'),

  statHostname: document.getElementById('stat-hostname'),
  statOs: document.getElementById('stat-os'),
  statArch: document.getElementById('stat-arch'),
  statCpu: document.getElementById('stat-cpu'),
  statCores: document.getElementById('stat-cores'),
  statRam: document.getElementById('stat-ram'),
  statUptime: document.getElementById('stat-uptime'),

  txtClipboardInput: document.getElementById('txt-clipboard-input'),
  btnClipRead: document.getElementById('btn-clip-read'),
  btnClipWrite: document.getElementById('btn-clip-write'),
  chatMessagesBox: document.getElementById('chat-messages-box'),
  chatInput: document.getElementById('chat-input'),
  btnSendMessage: document.getElementById('btn-send-message'),

  clientCodeIndicator: document.getElementById('client-code-indicator'),
  clientChatMessages: document.getElementById('client-chat-messages'),
  clientChatInput: document.getElementById('client-chat-input'),
  btnClientSendMessage: document.getElementById('btn-client-send-message'),
  btnClientKill: document.getElementById('btn-client-kill'),
  clientActivityLog: document.getElementById('client-activity-log')
};

// ==========================================================================
// 1. APPLICATION VIEW SWITCHER
// ==========================================================================

function showScreen(screenType) {
  el.landing.classList.remove('active-screen');
  el.admin.classList.add('hide');
  el.client.classList.add('hide');

  if (screenType === 'landing') {
    el.landing.classList.add('active-screen');
  } else if (screenType === 'admin') {
    el.admin.classList.remove('hide');
    resizeCanvas();
  } else if (screenType === 'client') {
    el.client.classList.remove('hide');
  }
}

// ==========================================================================
// 2. CLIENT LOCAL CONNECTION (LOOPBACK TO LOCAL MACHINE OS AGENT)
// ==========================================================================

function connectLocalOSBridge() {
  if (socketLocal) return;

  console.log('[Local Bridge] Connecting to local OS-control loopback...');
  socketLocal = io('http://localhost:4899');

  socketLocal.on('connect', () => {
    console.log('[Local Bridge] Direct communication with local OS bridge active!');
    logClientActivity('Successfully bound loopback OS automation engine');
  });

  // Receive real terminal data streams
  socketLocal.on('local-os-terminal-data', (data) => {
    appendTerminalLine(data, 'stdout');
  });

  socketLocal.on('local-os-terminal-end', () => {
    el.terminalInput.disabled = false;
    el.terminalInput.focus();
  });

  // Receive directory index
  socketLocal.on('local-os-file-browse-res', (res) => {
    if (res.error) {
      alert(`Explorer Error: ${res.error}`);
      return;
    }
    renderFileList(res.items, res.path);
  });

  // Receive read file contents
  socketLocal.on('local-os-file-read-res', (res) => {
    if (res.error) {
      alert(`Read Error: ${res.error}`);
      return;
    }
    openEditor(res.filePath, res.content);
  });

  // Receive write file content confirmation
  socketLocal.on('local-os-file-write-res', (res) => {
    if (res.error) {
      alert(`Save Failed: ${res.error}`);
    } else {
      alert('File successfully saved remotely!');
    }
  });

  // Receive real CPU/RAM status metrics
  socketLocal.on('local-os-system-stats-res', (stats) => {
    // If we are currently sharing, relay these stats to our central socket (Admin)
    if (socketCentral && myRole === 'client') {
      socketCentral.emit('control-action', {
        sessionCode: activeSessionCode,
        action: 'system-stats-payload',
        payload: stats
      });
    }
  });
}

// Log actions on Client HUD console screen
function logClientActivity(msg, type = 'info') {
  const time = new Date().toLocaleTimeString();
  const line = document.createElement('div');
  line.className = `log-line ${type}`;
  line.innerHTML = `<span>[${time}]</span> ${msg}`;
  el.clientActivityLog.appendChild(line);
  el.clientActivityLog.scrollTop = el.clientActivityLog.scrollHeight;
}

// ==========================================================================
// 3. ADMIN PORTAL ACTIONS (NODE INITIALIZATION & CODE CREATION)
// ==========================================================================

el.btnCreateSession.addEventListener('click', () => {
  myRole = 'admin';
  activeTargetIP = 'localhost:4899';

  // Connect socket.central directly to localhost port 4899
  console.log('[Central] Spawning local signaling session...');
  socketCentral = io('http://localhost:4899');

  socketCentral.on('connect', () => {
    socketCentral.emit('create-session');
  });

  socketCentral.on('session-created', ({ sessionCode }) => {
    activeSessionCode = sessionCode;
    el.txtSessionCode.textContent = sessionCode;
    el.btnCreateSession.classList.add('hide');
    el.adminPairingHub.classList.remove('hide');
  });

  socketCentral.on('session-linked', () => {
    console.log('[Central] Pairing successful! Establishing signaling channels...');
    setupAdminWorkspace();
  });

  socketCentral.on('session-terminated', () => {
    alert('Client disconnected from session.');
    teardownSession();
  });

  socketCentral.on('signal', (data) => {
    handleIncomingSignal(data);
  });

  socketCentral.on('control-action', ({ action, payload }) => {
    handleAdminSideActions(action, payload);
  });
});

// Copy Code Clipboard helper
el.btnCopyCode.addEventListener('click', () => {
  const code = el.txtSessionCode.textContent;
  navigator.clipboard.writeText(code).then(() => {
    alert(`Code copied to clipboard: ${code}`);
  });
});

// ==========================================================================
// 4. CLIENT PORTAL ACTIONS (INPUT CODE & HANDSHAKE PIPELINE)
// ==========================================================================

el.btnJoinSession.addEventListener('click', () => {
  const targetIP = el.inputTargetIP.value.trim();
  const sessionCode = el.inputSessionCode.value.trim();

  if (!sessionCode) {
    alert('Please enter a secure connection Link PIN.');
    return;
  }

  myRole = 'client';
  activeSessionCode = sessionCode;
  activeTargetIP = targetIP;

  const signalingUrl = targetIP.startsWith('http') ? targetIP : `http://${targetIP}`;

  console.log(`[Central] Connecting to Admin signaling hub at: ${signalingUrl}`);
  el.btnJoinSession.disabled = true;
  el.btnJoinSession.querySelector('span').textContent = 'Connecting...';

  socketCentral = io(signalingUrl);

  socketCentral.on('connect', () => {
    socketCentral.emit('join-session', { sessionCode });
  });

  socketCentral.on('link-error', ({ message }) => {
    el.txtLinkError.textContent = message;
    el.txtLinkError.classList.remove('hide');
    el.btnJoinSession.disabled = false;
    el.btnJoinSession.querySelector('span').textContent = 'Link Device Terminal';
    socketCentral.disconnect();
    socketCentral = null;
  });

  socketCentral.on('session-linked', () => {
    // Connect to our own local OS bridge server
    connectLocalOSBridge();
    setupClientWorkspace();
  });

  socketCentral.on('session-terminated', () => {
    alert('Admin terminated connection.');
    teardownSession();
  });

  socketCentral.on('signal', (data) => {
    handleIncomingSignal(data);
  });

  socketCentral.on('control-action', ({ action, payload }) => {
    handleClientSideOSActions(action, payload);
  });
});

// ==========================================================================
// 5. WEBRTC STREAMING HANDSHAKE (DIRECT P2P MEDIA EXCHANGES)
// ==========================================================================

function handleIncomingSignal(data) {
  if (!rtcPeer) return;

  if (data.sdp) {
    rtcPeer.setRemoteDescription(new RTCSessionDescription(data.sdp))
      .then(() => {
        if (rtcPeer.remoteDescription.type === 'offer') {
          rtcPeer.createAnswer().then((answer) => {
            rtcPeer.setLocalDescription(answer).then(() => {
              socketCentral.emit('signal', {
                sessionCode: activeSessionCode,
                data: { sdp: rtcPeer.localDescription }
              });
            });
          });
        }
      });
  } else if (data.candidate) {
    rtcPeer.addIceCandidate(new RTCIceCandidate(data.candidate))
      .catch(e => console.error('[WebRTC] Error adding ICE Candidate:', e));
  }
}

// Client starts screen capturing and creates connection Offer
async function startClientWebRTC() {
  console.log('[WebRTC] Initiating media share handshakes...');
  logClientActivity('Capturing physical desktop screen stream...');

  try {
    // Call HTML5 Media API (opens Chrome screen share overlay)
    localMediaStream = await navigator.mediaDevices.getDisplayMedia({
      video: {
        cursor: "always",
        displaySurface: "monitor"
      },
      audio: false
    });

    logClientActivity('Screen captures active. Establishing Peer Connection...');

    // Extract actual width/height of the captured track
    const track = localMediaStream.getVideoTracks()[0];
    const settings = track.getSettings();

    clientScreenResolution = {
      width: settings.width || screen.width,
      height: settings.height || screen.height
    };

    console.log(`[WebRTC] Client Screen dimensions: ${clientScreenResolution.width}x${clientScreenResolution.height}`);

    // Relay screen dimension handshake to Admin
    socketCentral.emit('control-action', {
      sessionCode: activeSessionCode,
      action: 'resolution-handshake',
      payload: clientScreenResolution
    });

    // Create RTCPeerConnection
    rtcPeer = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
    });

    localMediaStream.getTracks().forEach(t => rtcPeer.addTrack(t, localMediaStream));

    rtcPeer.onicecandidate = (event) => {
      if (event.candidate) {
        socketCentral.emit('signal', {
          sessionCode: activeSessionCode,
          data: { candidate: event.candidate }
        });
      }
    };

    rtcPeer.createOffer().then((offer) => {
      rtcPeer.setLocalDescription(offer).then(() => {
        socketCentral.emit('signal', {
          sessionCode: activeSessionCode,
          data: { sdp: rtcPeer.localDescription }
        });
      });
    });

    logClientActivity('Tunneling media tracks to remote Admin window...');

    // Listen for screen share termination (user clicks "stop sharing" native button)
    track.onended = () => {
      logClientActivity('Screen share stream ended by host client.', 'error');
      teardownSession();
    };

  } catch (err) {
    console.error('[WebRTC] Screen capture failed:', err);
    logClientActivity(`Screen capture failed: ${err.message}`, 'error');
    alert(`Screen sharing is required for link: ${err.message}`);
    teardownSession();
  }
}

// ==========================================================================
// 6. ADMIN VIEWPORT MANAGEMENT & PRECISE MOUSE TRANSLATIONS
// ==========================================================================

function setupAdminWorkspace() {
  showScreen('admin');
  el.adminCodeIndicator.textContent = activeSessionCode;

  // Listeners for WebRTC incoming streams
  rtcPeer = new RTCPeerConnection({
    iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
  });

  rtcPeer.onicecandidate = (event) => {
    if (event.candidate) {
      socketCentral.emit('signal', {
        sessionCode: activeSessionCode,
        data: { candidate: event.candidate }
      });
    }
  };

  rtcPeer.ontrack = (event) => {
    console.log('[WebRTC] Stream track received! Binding to Admin viewport video...');
    el.video.srcObject = event.streams[0];

    // Hide loading prompt and show screen
    el.viewportFallback.classList.add('hide');
    el.video.classList.remove('hide');
  };

  // Configure viewport cursor coordinate captures
  setupViewportEvents();

  // Connect to local server just for clipboard or file editor options on Admin if needed
  connectLocalOSBridge();

  // Load drives in Explorer immediately
  requestLocalFolderListing('');
  // Poll system info
  requestSystemStats();
}

function setupViewportEvents() {
  const wrapper = el.viewportWrapper;

  // Scale coordinates: maps Admin viewport coordinates back to Client monitor grid
  function scaleCoordinates(clientX, clientY) {
    const video = el.video;
    const rect = video.getBoundingClientRect();

    // Width and height of video box
    const viewW = rect.width;
    const viewH = rect.height;

    // Offset from screen
    const offsetX = clientX - rect.left;
    const offsetY = clientY - rect.top;

    // Relative ratios (0.0 to 1.0)
    const xRatio = offsetX / viewW;
    const yRatio = offsetY / viewH;

    // Convert directly to Client physical screen dimensions!
    const targetX = Math.round(xRatio * clientScreenResolution.width);
    const targetY = Math.round(yRatio * clientScreenResolution.height);

    return { x: targetX, y: targetY };
  }

  // Mouse Movement relative to video
  wrapper.addEventListener('mousemove', (e) => {
    if (!isRemoteInteractionActive) return;

    const coords = scaleCoordinates(e.clientX, e.clientY);

    // Boundary check
    if (coords.x >= 0 && coords.x <= clientScreenResolution.width && coords.y >= 0 && coords.y <= clientScreenResolution.height) {

      // Update local virtual cursor highlight
      el.virtualCursor.style.left = `${e.clientX - wrapper.getBoundingClientRect().left}px`;
      el.virtualCursor.style.top = `${e.clientY - wrapper.getBoundingClientRect().top}px`;
      el.virtualCursor.style.display = 'block';

      if (isAnnotationModeActive) {
        if (isDrawing) {
          sendDrawLine(coords.x, coords.y);
        }
      } else {
        // Send coordinate to Client
        socketCentral.emit('control-action', {
          sessionCode: activeSessionCode,
          action: 'mouse-move',
          payload: coords
        });
      }
    } else {
      el.virtualCursor.style.display = 'none';
    }
  });

  wrapper.addEventListener('mouseleave', () => {
    el.virtualCursor.style.display = 'none';
    isDrawing = false;
  });

  // Mouse down/up triggers click ripples
  wrapper.addEventListener('mousedown', (e) => {
    if (!isRemoteInteractionActive) return;
    const coords = scaleCoordinates(e.clientX, e.clientY);

    // Limit actions within video boundaries
    if (coords.x < 0 || coords.x > clientScreenResolution.width || coords.y < 0 || coords.y > clientScreenResolution.height) {
      return;
    }

    if (isAnnotationModeActive) {
      isDrawing = true;
      lastDrawPoint = coords;
    } else {
      // Direct left click ripple
      createClickRipple(e.clientX - wrapper.getBoundingClientRect().left, e.clientY - wrapper.getBoundingClientRect().top);

      const buttonType = e.button === 2 ? 'right' : 'left';

      socketCentral.emit('control-action', {
        sessionCode: activeSessionCode,
        action: 'mouse-click',
        payload: {
          x: coords.x,
          y: coords.y,
          button: buttonType,
          double: e.detail === 2
        }
      });
    }
  });

  wrapper.addEventListener('mouseup', () => {
    isDrawing = false;
  });

  // Block default right click menu inside remote view
  wrapper.addEventListener('contextmenu', (e) => e.preventDefault());

  // Physical keyboard captures when viewport focused
  window.addEventListener('keydown', (e) => {
    if (!isRemoteInteractionActive || isAnnotationModeActive) return;

    // Check if mouse is hovering over the viewport wrapper (means viewport is focused)
    const rect = wrapper.getBoundingClientRect();
    const isHovered = (
      window.mousePos &&
      window.mousePos.x >= rect.left && window.mousePos.x <= rect.right &&
      window.mousePos.y >= rect.top && window.mousePos.y <= rect.bottom
    );

    // If focused on an input box, skip capture
    if (document.activeElement.tagName === 'INPUT' || document.activeElement.tagName === 'TEXTAREA') {
      return;
    }

    // Capture standard functional keys
    const targetKeys = ['Enter', 'Backspace', 'Escape', 'Tab', 'Space', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Delete'];
    const key = e.key;

    if (targetKeys.includes(key)) {
      e.preventDefault();

      const sendKey = key.replace('Arrow', '').toLowerCase();
      socketCentral.emit('control-action', {
        sessionCode: activeSessionCode,
        action: 'key-press',
        payload: { key: sendKey }
      });
    } else if (key.length === 1 && !e.ctrlKey && !e.altKey) {
      // Standard typing characters
      e.preventDefault();
      socketCentral.emit('control-action', {
        sessionCode: activeSessionCode,
        action: 'text-type',
        payload: { text: key }
      });
    }
  });
}

// Global mouse tracker for keyboard focus
window.addEventListener('mousemove', (e) => {
  window.mousePos = { x: e.clientX, y: e.clientY };
});

// Viewport click ripple animation creator
function createClickRipple(x, y) {
  const ripple = document.createElement('div');
  ripple.className = 'viewport-click-ripple';
  ripple.style.left = `${x}px`;
  ripple.style.top = `${y}px`;

  el.viewportWrapper.appendChild(ripple);

  // Cleanup after animation completes
  setTimeout(() => ripple.remove(), 600);
}

// Canvas Drawings Setup (Admin overlay annotations)
function sendDrawLine(x, y) {
  socketCentral.emit('control-action', {
    sessionCode: activeSessionCode,
    action: 'draw-line',
    payload: {
      x1: lastDrawPoint.x,
      y1: lastDrawPoint.y,
      x2: x,
      y2: y
    }
  });

  // Local drawing on admin canvas
  drawLocalLine(lastDrawPoint, { x, y });
  lastDrawPoint = { x, y };
}

function drawLocalLine(p1, p2) {
  const canvas = el.canvas;
  const ctx = canvas.getContext('2d');

  // Scale points to fit canvas size
  const scaleX = canvas.width / clientScreenResolution.width;
  const scaleY = canvas.height / clientScreenResolution.height;

  ctx.strokeStyle = 'hsl(190, 100%, 50%)'; // glowing cyan
  ctx.lineWidth = 3;
  ctx.lineCap = 'round';
  ctx.shadowColor = 'rgba(0, 242, 254, 0.5)';
  ctx.shadowBlur = 8;

  ctx.beginPath();
  ctx.moveTo(p1.x * scaleX, p1.y * scaleY);
  ctx.lineTo(p2.x * scaleX, p2.y * scaleY);
  ctx.stroke();

  ctx.shadowBlur = 0; // reset
}

function resizeCanvas() {
  const canvas = el.canvas;
  const rect = el.video.getBoundingClientRect();
  canvas.width = rect.width || 800;
  canvas.height = rect.height || 600;
}

window.addEventListener('resize', resizeCanvas);
el.video.addEventListener('loadedmetadata', resizeCanvas);

// Viewport Actions bindings
el.btnToggleControl.addEventListener('click', () => {
  isRemoteInteractionActive = !isRemoteInteractionActive;
  el.btnToggleControl.classList.toggle('active', isRemoteInteractionActive);
});

el.btnAdminDraw.addEventListener('click', () => {
  isAnnotationModeActive = !isAnnotationModeActive;
  el.btnAdminDraw.classList.toggle('active', isAnnotationModeActive);
  if (isAnnotationModeActive) {
    el.viewportWrapper.style.cursor = 'crosshair';
  } else {
    el.viewportWrapper.style.cursor = 'default';
  }
});

el.btnAdminClearDraw.addEventListener('click', () => {
  const canvas = el.canvas;
  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Relay clear to client
  socketCentral.emit('control-action', {
    sessionCode: activeSessionCode,
    action: 'clear-drawings'
  });
});

// ==========================================================================
// 7. CLIENT SYSTEM WORKSPACE (STREAMING STATE HANDLERS)
// ==========================================================================

function setupClientWorkspace() {
  showScreen('client');
  el.clientCodeIndicator.textContent = activeSessionCode;

  // Establish screen stream via WebRTC
  startClientWebRTC();

  // Send initial local stats to Admin
  setTimeout(() => {
    if (socketLocal) socketLocal.emit('local-os-system-stats');
  }, 2000);

  // Poll system statistics every 4 seconds
  setInterval(() => {
    if (socketCentral && socketLocal && myRole === 'client') {
      socketLocal.emit('local-os-system-stats');
    }
  }, 4000);
}

// Handlers for Admin actions receiving at Client Browser Renderer
function handleClientSideOSActions(action, payload) {
  if (!socketLocal) return;

  switch (action) {
    case 'mouse-move':
      socketLocal.emit('local-os-move-mouse', payload);
      // Limit cursor movement log spam on UI, but register visually
      break;

    case 'mouse-click':
      socketLocal.emit('local-os-click-mouse', payload);
      logClientActivity(`Admin executed remote ${payload.button.toUpperCase()} click at (${payload.x}, ${payload.y})`);
      break;

    case 'text-type':
      socketLocal.emit('local-os-type-text', payload);
      logClientActivity(`Admin inputted keystroke: ${payload.text}`);
      break;

    case 'key-press':
      socketLocal.emit('local-os-press-key', payload);
      logClientActivity(`Admin pressed functional key: ${payload.key.toUpperCase()}`);
      break;

    case 'draw-line':
      // Render line drawing locally on Client's visual log screen overlay if they have it
      drawClientOverlayLine(payload);
      break;

    case 'clear-drawings':
      clearClientOverlayCanvas();
      break;

    case 'terminal-cmd':
      logClientActivity(`Admin executing shell command: ${payload.command}`, 'system');
      socketLocal.emit('local-os-terminal', payload);
      break;

    case 'file-browse':
      logClientActivity(`Admin index directory path: ${payload.dirPath}`, 'system');
      socketLocal.emit('local-os-file-browse', payload);
      break;

    case 'file-read':
      logClientActivity(`Admin downloading file contents: ${payload.filePath}`);
      socketLocal.emit('local-os-file-read', payload);
      break;

    case 'file-write':
      logClientActivity(`Admin overwrote contents of file: ${payload.filePath}`, 'system');
      socketLocal.emit('local-os-file-write', payload);
      break;

    case 'system-clipboard-write':
      logClientActivity(`Admin synchronized system clipboard data: "${payload.text.substring(0, 15)}..."`);
      socketLocal.emit('local-os-type-text', { text: payload.text }); // Fallback paste or typing
      break;

    case 'chat-msg':
      appendChatMessage(payload.sender, payload.message, true);
      break;
  }
}

// Client canvas drawer overlays
function drawClientOverlayLine(coords) {
  const canvas = el.canvas;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');

  const scaleX = canvas.width / clientScreenResolution.width;
  const scaleY = canvas.height / clientScreenResolution.height;

  ctx.strokeStyle = 'hsl(272, 92%, 60%)'; // Indy violet
  ctx.lineWidth = 3;
  ctx.lineCap = 'round';

  ctx.beginPath();
  ctx.moveTo(coords.x1 * scaleX, coords.y1 * scaleY);
  ctx.lineTo(coords.x2 * scaleX, coords.y2 * scaleY);
  ctx.stroke();
}

function clearClientOverlayCanvas() {
  const canvas = el.canvas;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, canvas.width, canvas.height);
}

// ==========================================================================
// 8. HANDLERS FOR CENTRAL EVENTS ON ADMIN SIDE (RECEIVING DATA)
// ==========================================================================

function handleAdminSideActions(action, payload) {
  switch (action) {
    case 'resolution-handshake':
      clientScreenResolution = payload;
      console.log(`[Handshake] Synchronized Remote monitor scale dimensions to: ${payload.width}x${payload.height}`);
      resizeCanvas();
      break;

    case 'system-stats-payload':
      updateStatsPanel(payload);
      break;

    case 'chat-msg':
      appendChatMessage(payload.sender, payload.message, false);
      break;

    case 'file-browse-res':
      renderFileList(payload.items, payload.path);
      break;

    case 'file-read-res':
      openEditor(payload.filePath, payload.content);
      break;

    case 'file-write-res':
      if (payload.success) alert('File successfully saved on host computer!');
      break;

    case 'terminal-data-stream':
      appendTerminalLine(payload.text, payload.streamType);
      break;
  }
}

// ==========================================================================
// 9. POWERSHELL TERMINAL ENGINE CONTROLLER
// ==========================================================================

el.terminalInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') {
    const cmd = el.terminalInput.value.trim();
    if (!cmd) return;

    // Append to console list locally
    appendTerminalLine(`PS C:\\> ${cmd}`, 'prompt-row');
    el.terminalInput.value = '';

    // Clear console helper
    if (cmd.toLowerCase() === 'clear' || cmd.toLowerCase() === 'cls') {
      el.terminalOutput.innerHTML = '';
      return;
    }

    el.terminalInput.disabled = true;

    // Check if we are client or admin
    if (myRole === 'admin') {
      // Send command over to Client
      socketCentral.emit('control-action', {
        sessionCode: activeSessionCode,
        action: 'terminal-cmd',
        payload: { command: cmd }
      });
    } else {
      // If client running locally, execute on own local loopback
      socketLocal.emit('local-os-terminal', { command: cmd });
    }
  }
});

function appendTerminalLine(text, streamType) {
  const line = document.createElement('div');
  line.className = `term-line ${streamType}`;
  line.textContent = text;

  el.terminalOutput.appendChild(line);
  el.terminalOutput.scrollTop = el.terminalOutput.scrollHeight;
}

// ==========================================================================
// 10. FILE EXPLORER CONTROLLER
// ==========================================================================

function requestLocalFolderListing(dirPath) {
  if (myRole === 'admin') {
    socketCentral.emit('control-action', {
      sessionCode: activeSessionCode,
      action: 'file-browse',
      payload: { dirPath }
    });
  } else {
    if (socketLocal) socketLocal.emit('local-os-file-browse', { dirPath });
  }
}

function renderFileList(items, currentPath) {
  currentFilePath = currentPath;
  el.txtCurrentPath.textContent = currentPath;
  el.explorerFileList.innerHTML = '';

  if (!items || items.length === 0) {
    el.explorerFileList.innerHTML = '<div class="file-item">Empty Directory</div>';
    return;
  }

  items.forEach(item => {
    const row = document.createElement('div');
    row.className = `file-item ${item.isDirectory ? 'directory' : 'file'}`;

    const icon = item.isDirectory
      ? `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>`
      : `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>`;

    const size = item.isDirectory ? '' : formatBytes(item.size);

    row.innerHTML = `
      ${icon}
      <span class="file-name">${item.name}</span>
      <span class="file-size">${size}</span>
    `;

    row.addEventListener('dblclick', () => {
      if (item.isDirectory) {
        requestLocalFolderListing(item.path);
      } else {
        // Request reading file content
        requestFileContent(item.path);
      }
    });

    el.explorerFileList.appendChild(row);
  });
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// Navigation directories up
el.btnFileUp.addEventListener('click', () => {
  // Simple windows path separation
  const parts = currentFilePath.split('\\');
  // Remove last item
  if (parts.length > 1) {
    // If it ends with blank or double slash, clear
    let newPath = parts.slice(0, -1).join('\\');
    if (newPath.endsWith(':')) newPath += '\\';
    requestLocalFolderListing(newPath);
  } else {
    requestLocalFolderListing('');
  }
});

el.btnFileRefresh.addEventListener('click', () => {
  requestLocalFolderListing(currentFilePath);
});

// Files editors operations
function requestFileContent(filePath) {
  if (myRole === 'admin') {
    socketCentral.emit('control-action', {
      sessionCode: activeSessionCode,
      action: 'file-read',
      payload: { filePath }
    });
  } else {
    if (socketLocal) socketLocal.emit('local-os-file-read', { filePath });
  }
}

function openEditor(filePath, content) {
  el.editorFilename.textContent = filePath.split('\\').pop();
  el.editorFilename.dataset.filepath = filePath;
  el.explorerTextarea.value = content;

  el.explorerFileList.classList.add('hide');
  el.explorerEditorBox.classList.remove('hide');
}

el.btnCloseEditor.addEventListener('click', () => {
  el.explorerEditorBox.classList.add('hide');
  el.explorerFileList.classList.remove('hide');
});

el.btnSaveFile.addEventListener('click', () => {
  const filePath = el.editorFilename.dataset.filepath;
  const content = el.explorerTextarea.value;

  if (myRole === 'admin') {
    socketCentral.emit('control-action', {
      sessionCode: activeSessionCode,
      action: 'file-write',
      payload: { filePath, content }
    });
  } else {
    if (socketLocal) {
      socketLocal.emit('local-os-file-write', { filePath, content });
    }
  }
});

// ==========================================================================
// 11. SYSTEM STATISTICS GAUGES UPDATES
// ==========================================================================

function requestSystemStats() {
  setInterval(() => {
    if (socketCentral && myRole === 'admin') {
      // Central server queries client to pull stats which fires system-stats-payload
    }
  }, 4000);
}

function updateStatsPanel(stats) {
  // Update CPU gauges
  const cpuOffset = 251 - (251 * stats.cpuUsage) / 100;
  el.gaugeCpuCircle.style.strokeDashoffset = cpuOffset;
  el.gaugeCpuVal.textContent = `${stats.cpuUsage}%`;

  // Update memory gauges
  const memOffset = 251 - (251 * stats.memoryUsage) / 100;
  el.gaugeMemCircle.style.strokeDashoffset = memOffset;
  el.gaugeMemVal.textContent = `${stats.memoryUsage}%`;

  // Text metrics lists
  el.statHostname.textContent = stats.hostname;
  el.statOs.textContent = `${stats.osType} (${stats.osPlatform} ${stats.osRelease})`;
  el.statArch.textContent = stats.arch;
  el.statCpu.textContent = stats.cpuModel;
  el.statCores.textContent = stats.cpuCores;
  el.statRam.textContent = (stats.totalMemory / (1024 ** 3)).toFixed(1);

  // Format uptime
  const uptimeHours = Math.floor(stats.uptime / 3600);
  const uptimeMin = Math.floor((stats.uptime % 3600) / 60);
  el.statUptime.textContent = `${uptimeHours}h ${uptimeMin}m`;
}

// ==========================================================================
// 12. CHAT & SYSTEM CLIPBOARD SYSTEM
// ==========================================================================

// Chat tabs button navigations
const tabs = document.querySelectorAll('.tab-btn');
tabs.forEach(tab => {
  tab.addEventListener('click', () => {
    tabs.forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));

    tab.classList.add('active');
    document.getElementById(tab.dataset.tab).classList.add('active');
  });
});

// Sending Chat
function triggerSendChat() {
  const input = myRole === 'admin' ? el.chatInput : el.clientChatInput;
  const msgText = input.value.trim();
  if (!msgText) return;

  socketCentral.emit('control-action', {
    sessionCode: activeSessionCode,
    action: 'chat-msg',
    payload: {
      sender: myRole,
      message: msgText
    }
  });

  appendChatMessage(myRole, msgText, myRole === 'client');
  input.value = '';
}

el.btnSendMessage.addEventListener('click', triggerSendChat);
el.chatInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') triggerSendChat();
});

el.btnClientSendMessage.addEventListener('click', triggerSendChat);
el.clientChatInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') triggerSendChat();
});

function appendChatMessage(sender, message, isClientHUD) {
  const box = isClientHUD ? el.clientChatMessages : el.chatMessagesBox;
  const msg = document.createElement('div');

  const isOutgoing = sender === myRole;
  msg.className = `msg ${isOutgoing ? 'outgoing' : 'incoming'}`;
  msg.textContent = message;

  box.appendChild(msg);
  box.scrollTop = box.scrollHeight;
}

// System Clipboard handlers
el.btnClipRead.addEventListener('click', () => {
  navigator.clipboard.readText().then(text => {
    el.txtClipboardInput.value = text;
    alert('Loaded clipboard data from Admin OS!');
  }).catch(() => {
    alert('Please grant clipboard reading permissions to copy physical data.');
  });
});

el.btnClipWrite.addEventListener('click', () => {
  const text = el.txtClipboardInput.value;
  if (!text) {
    alert('Please enter text to synchronize.');
    return;
  }

  // Relay clipboard paste to client
  socketCentral.emit('control-action', {
    sessionCode: activeSessionCode,
    action: 'system-clipboard-write',
    payload: { text }
  });

  alert('Dispatched synced clipboard commands to Client OS!');
});

// ==========================================================================
// 13. SAFETY SESSION TERMINATION & KILL SWITCH
// ==========================================================================

function teardownSession() {
  console.log('[Teardown] Closing peer streams and severing network pipes...');

  if (localMediaStream) {
    localMediaStream.getTracks().forEach(t => t.stop());
    localMediaStream = null;
  }

  if (rtcPeer) {
    rtcPeer.close();
    rtcPeer = null;
  }

  if (socketCentral) {
    socketCentral.disconnect();
    socketCentral = null;
  }

  // Visual cleanups
  el.video.srcObject = null;
  el.video.classList.add('hide');
  el.viewportFallback.classList.remove('hide');
  el.txtLinkError.classList.add('hide');

  el.btnCreateSession.classList.remove('hide');
  el.adminPairingHub.classList.add('hide');
  el.btnJoinSession.disabled = false;
  el.btnJoinSession.querySelector('span').textContent = 'Link Device Terminal';
  el.inputSessionCode.value = '';

  showScreen('landing');
}

el.btnAdminDisconnect.addEventListener('click', teardownSession);
el.btnClientKill.addEventListener('click', () => {
  logClientActivity('KILL SWITCH ACTIVATED. Severing physical desktop streams immediately.', 'error');
  teardownSession();
});

// Bind global ESC key on Client dashboard to force immediate disconnection
window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && myRole === 'client') {
    logClientActivity('KILL SWITCH TRIGGERED VIA ESC KEYBOARD SHORTCUT.', 'error');
    teardownSession();
  }
});
