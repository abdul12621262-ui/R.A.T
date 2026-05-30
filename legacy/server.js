const express = require('express');
const http = require('http');
const socketIo = require('socket.io');
const path = require('path');
const OSController = require('./src/os-controller');
const { launchBrowser } = require('./launcher');

const app = express();
const server = http.createServer(app);

// Configure Socket.io with broad CORS for cross-device networking
const io = socketIo(server, {
  cors: {
    origin: '*',
    methods: ['GET', 'POST']
  }
});

const PORT = 4899;

// Serve public static assets
app.use(express.static(path.join(__dirname, 'public')));

// Registry for active peer-to-peer session rooms
const activeRooms = {};

io.on('connection', (socket) => {
  console.log(`[Socket Server] New socket connection: ${socket.id}`);

  // 1. ADMIN ACTIONS: Generate Connection Code
  socket.on('create-session', () => {
    // Generate a secure, user-friendly 6-digit connection code (e.g. 583-921)
    const rawCode = Math.floor(100000 + Math.random() * 900000).toString();
    const sessionCode = `${rawCode.slice(0, 3)}-${rawCode.slice(3, 6)}`;

    activeRooms[sessionCode] = {
      adminId: socket.id,
      clientId: null,
      created: new Date()
    };

    socket.join(sessionCode);
    socket.emit('session-created', { sessionCode });
    console.log(`[Socket Server] Created Room Session: ${sessionCode} for Admin: ${socket.id}`);
  });

  // 2. CLIENT ACTIONS: Connect Using Code
  socket.on('join-session', ({ sessionCode }) => {
    const room = activeRooms[sessionCode];
    if (room) {
      if (room.clientId) {
        socket.emit('link-error', { message: 'This session is already in use by another client.' });
        return;
      }

      room.clientId = socket.id;
      socket.join(sessionCode);

      // Notify both Admin and Client that pairing is successful!
      io.to(sessionCode).emit('session-linked', { sessionCode });
      console.log(`[Socket Server] Client: ${socket.id} linked to Session Room: ${sessionCode}`);
    } else {
      socket.emit('link-error', { message: 'Invalid or expired connection code.' });
    }
  });

  // 3. WEBRTC SIGNALING RELAY: Pipes Offers, Answers, and ICE Candidates
  socket.on('signal', ({ sessionCode, data }) => {
    // Broadcast signaling message to other party in the room
    socket.to(sessionCode).emit('signal', data);
  });

  // 4. REMOTE CONTROL EVENTS PIPE: Relays real-time inputs
  socket.on('control-action', ({ sessionCode, action, payload }) => {
    // Relays actions like 'mouse-move', 'click', 'keypress', 'terminal-cmd', 'file-browse'
    socket.to(sessionCode).emit('control-action', { action, payload });
  });

  // 5. LOCAL LOOPBACK OS EXECUTION
  // Listeners for the client's local renderer page connecting to its own localhost process
  socket.on('local-os-move-mouse', ({ x, y }) => {
    OSController.moveMouse(x, y);
  });

  socket.on('local-os-click-mouse', ({ button, doubleClick }) => {
    OSController.clickMouse(button, doubleClick);
  });

  socket.on('local-os-type-text', ({ text }) => {
    OSController.typeText(text);
  });

  socket.on('local-os-press-key', ({ key }) => {
    OSController.pressKey(key);
  });

  socket.on('local-os-terminal', ({ command }) => {
    OSController.runTerminalCommand(
      command,
      (chunk) => {
        socket.emit('local-os-terminal-data', chunk);
      },
      (exitMsg) => {
        socket.emit('local-os-terminal-data', exitMsg);
        socket.emit('local-os-terminal-end');
      }
    );
  });

  socket.on('local-os-file-browse', ({ dirPath }) => {
    OSController.browseDirectory(dirPath, (err, items) => {
      if (err) {
        socket.emit('local-os-file-browse-res', { error: err.message, path: dirPath });
      } else {
        socket.emit('local-os-file-browse-res', { items, path: dirPath });
      }
    });
  });

  socket.on('local-os-file-read', ({ filePath }) => {
    OSController.readFileContent(filePath, (err, content) => {
      if (err) {
        socket.emit('local-os-file-read-res', { error: err.message, filePath });
      } else {
        socket.emit('local-os-file-read-res', { content, filePath });
      }
    });
  });

  socket.on('local-os-file-write', ({ filePath, content }) => {
    OSController.writeFileContent(filePath, content, (err) => {
      if (err) {
        socket.emit('local-os-file-write-res', { error: err.message, filePath });
      } else {
        socket.emit('local-os-file-write-res', { success: true, filePath });
      }
    });
  });

  socket.on('local-os-system-stats', () => {
    OSController.getSystemStats((err, stats) => {
      if (!err) {
        socket.emit('local-os-system-stats-res', stats);
      }
    });
  });

  // Clean disconnects
  socket.on('disconnect', () => {
    console.log(`[Socket Server] Socket disconnected: ${socket.id}`);

    // Find rooms owned by or linked to this socket
    for (const code in activeRooms) {
      const room = activeRooms[code];
      if (room.adminId === socket.id || room.clientId === socket.id) {
        io.to(code).emit('session-terminated', { reason: 'Peer disconnected' });
        delete activeRooms[code];
        console.log(`[Socket Server] Terminated and cleaned Room Session: ${code}`);
      }
    }
  });
});

// Start listening and auto-launch App window
server.listen(PORT, '0.0.0.0', () => {
  const localUrl = `http://localhost:${PORT}`;
  console.log('\n======================================================');
  console.log(`  AetherMod Desktop Active: Running on ${localUrl}`);
  console.log('======================================================\n');

  // Launch Chrome/Edge borderless window container
  launchBrowser(localUrl);
});

// Listen for process termination to safely shutdown OS bridge
process.on('SIGINT', () => {
  OSController.shutdown();
  process.exit(0);
});

process.on('SIGTERM', () => {
  OSController.shutdown();
  process.exit(0);
});
