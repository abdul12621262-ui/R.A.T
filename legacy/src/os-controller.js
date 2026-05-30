const { spawn, exec } = require('child_process');
const fs = require('fs');
const path = require('path');

// Initialize persistent PowerShell background process
let ps = null;

function initPowerShell() {
  if (ps) return;
  
  console.log('[OS Bridge] Initializing persistent PowerShell process...');
  ps = spawn('powershell.exe', ['-NoExit', '-Command', '-'], {
    stdio: ['pipe', 'pipe', 'pipe']
  });

  // Load essential WinForms Assemblies for automation
  ps.stdin.write(`
    Add-Type -AssemblyName System.Windows.Forms
    Add-Type -AssemblyName System.Drawing
    Add-Type -MemberDefinition '
      [DllImport("user32.dll")]
      public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo);
    ' -Name User32 -Namespace Win32API
  \n`);

  ps.stdout.on('data', (data) => {
    // Keep internal logs clean, but can be enabled for debugging
  });

  ps.stderr.on('data', (data) => {
    console.error(`[OS Bridge PowerShell Error] ${data.toString()}`);
  });

  ps.on('close', (code) => {
    console.log(`[OS Bridge] Persistent PowerShell process exited with code ${code}`);
    ps = null;
  });
}

// Helper to escape special characters inside SendKeys API
function escapeSendKeys(text) {
  const specials = ['{', '}', '[', ']', '(', ')', '+', '^', '%', '~'];
  let result = '';
  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    if (specials.includes(char)) {
      result += `{${char}}`;
    } else {
      result += char;
    }
  }
  return result;
}

// Native OS automation tools
const OSController = {
  // Move Windows cursor to coordinates
  moveMouse(x, y) {
    if (!ps) initPowerShell();
    const cmd = `[System.Windows.Forms.Cursor]::Position = [System.Drawing.Point]::new(${x}, ${y})\n`;
    ps.stdin.write(cmd);
  },

  // Perform physical click
  clickMouse(button, doubleClick) {
    if (!ps) initPowerShell();
    if (button === 'left') {
      if (doubleClick) {
        ps.stdin.write(`
          [Win32API.User32]::mouse_event(0x0002, 0, 0, 0, 0)
          [Win32API.User32]::mouse_event(0x0004, 0, 0, 0, 0)
          Start-Sleep -m 50
          [Win32API.User32]::mouse_event(0x0002, 0, 0, 0, 0)
          [Win32API.User32]::mouse_event(0x0004, 0, 0, 0, 0)
        \n`);
      } else {
        ps.stdin.write(`
          [Win32API.User32]::mouse_event(0x0002, 0, 0, 0, 0)
          [Win32API.User32]::mouse_event(0x0004, 0, 0, 0, 0)
        \n`);
      }
    } else if (button === 'right') {
      ps.stdin.write(`
        [Win32API.User32]::mouse_event(0x0008, 0, 0, 0, 0)
        [Win32API.User32]::mouse_event(0x0010, 0, 0, 0, 0)
      \n`);
    }
  },

  // Simulate global OS typing
  typeText(text) {
    if (!ps) initPowerShell();
    const escaped = escapeSendKeys(text);
    ps.stdin.write(`[System.Windows.Forms.SendKeys]::SendWait("${escaped}")\n`);
  },

  // Simulate special keys (Enter, Backspace, Arrow keys, etc.)
  pressKey(key) {
    if (!ps) initPowerShell();
    const keyMap = {
      'enter': '{ENTER}',
      'backspace': '{BACKSPACE}',
      'escape': '{ESC}',
      'tab': '{TAB}',
      'space': ' ',
      'up': '{UP}',
      'down': '{DOWN}',
      'left': '{LEFT}',
      'right': '{RIGHT}',
      'delete': '{DELETE}'
    };

    const mapped = keyMap[key.toLowerCase()];
    if (mapped) {
      ps.stdin.write(`[System.Windows.Forms.SendKeys]::SendWait("${mapped}")\n`);
    }
  },

  // Execute actual PowerShell command and stream output
  runTerminalCommand(command, onData, onEnd) {
    console.log(`[OS Bridge Terminal] Executing: ${command}`);
    
    // Spawn a separate PowerShell task for terminal commands to avoid blocking input
    const terminalTask = spawn('powershell.exe', ['-Command', command]);

    terminalTask.stdout.on('data', (data) => {
      onData(data.toString());
    });

    terminalTask.stderr.on('data', (data) => {
      onData(`ERROR: ${data.toString()}`);
    });

    terminalTask.on('close', (code) => {
      onEnd(`\n[Process exited with code ${code}]\n`);
    });
  },

  // Map logical hard drives on Windows
  getLogicalDrives(callback) {
    exec('wmic logicaldisk get name', (err, stdout) => {
      if (err) {
        // Fallback to standard C:\ drive if wmic fails
        return callback(null, ['C:\\']);
      }
      
      const drives = stdout
        .split('\r\n')
        .map(line => line.trim())
        .filter(line => line.match(/^[A-Z]:$/))
        .map(drive => drive + '\\');

      if (drives.length === 0) {
        drives.push('C:\\');
      }
      
      callback(null, drives);
    });
  },

  // Browse local files and folder lists
  browseDirectory(dirPath, callback) {
    // If no path specified, list drives first
    if (!dirPath || dirPath === '/' || dirPath === '') {
      return this.getLogicalDrives((err, drives) => {
        if (err) return callback(err);
        
        const items = drives.map(drive => ({
          name: drive,
          path: drive,
          isDirectory: true,
          size: 0,
          modified: new Date()
        }));
        
        callback(null, items);
      });
    }

    const targetPath = path.resolve(dirPath);

    fs.readdir(targetPath, { withFileTypes: true }, (err, files) => {
      if (err) {
        return callback(err);
      }

      const items = [];
      
      files.forEach(file => {
        try {
          const itemPath = path.join(targetPath, file.name);
          const stats = fs.statSync(itemPath);
          
          items.push({
            name: file.name,
            path: itemPath,
            isDirectory: file.isDirectory(),
            size: stats.size,
            modified: stats.mtime
          });
        } catch (statErr) {
          // Skip files that fail to stat (permissions issues, system locks, etc.)
        }
      });

      // Sort: Directories first, then alphabetical names
      items.sort((a, b) => {
        if (a.isDirectory && !b.isDirectory) return -1;
        if (!a.isDirectory && b.isDirectory) return 1;
        return a.name.localeCompare(b.name);
      });

      callback(null, items);
    });
  },

  // Read raw contents of a text file
  readFileContent(filePath, callback) {
    fs.readFile(filePath, 'utf-8', (err, data) => {
      if (err) return callback(err);
      callback(null, data);
    });
  },

  // Write content to a text file (edit files remotely)
  writeFileContent(filePath, content, callback) {
    fs.writeFile(filePath, content, 'utf-8', (err) => {
      if (err) return callback(err);
      callback(null);
    });
  },

  // Get remote system statistics (Mock system details using standard Node platform)
  getSystemStats(callback) {
    const os = require('os');
    
    // Quick cmd trigger to check current CPU load
    exec('wmic cpu get LoadPercentage', (err, stdout) => {
      let cpuLoad = 15; // default fallback
      if (!err) {
        const matches = stdout.match(/\d+/);
        if (matches) cpuLoad = parseInt(matches[0]);
      }

      const stats = {
        osType: os.type(),
        osPlatform: os.platform(),
        osRelease: os.release(),
        arch: os.arch(),
        hostname: os.hostname(),
        uptime: os.uptime(),
        cpuModel: os.cpus()[0]?.model || 'Generic CPU',
        cpuCores: os.cpus().length,
        cpuUsage: cpuLoad,
        totalMemory: os.totalmem(),
        freeMemory: os.freemem(),
        memoryUsage: Math.round(((os.totalmem() - os.freemem()) / os.totalmem()) * 100)
      };

      callback(null, stats);
    });
  },

  // Clean shutdown
  shutdown() {
    if (ps) {
      console.log('[OS Bridge] Shutting down persistent PowerShell process...');
      ps.stdin.write("exit\n");
      ps.kill();
      ps = null;
    }
  }
};

// Auto-initialize when file is imported
initPowerShell();

module.exports = OSController;
