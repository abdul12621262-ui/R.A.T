const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

function launchBrowser(url) {
  const chromePaths = [
    'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
    'C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe',
    path.join(process.env.LOCALAPPDATA || '', 'Google\\Chrome\\Application\\chrome.exe')
  ];

  const edgePaths = [
    'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
    'C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe'
  ];

  let browserPath = null;

  // Search Google Chrome first for maximum compatibility
  for (const p of chromePaths) {
    if (fs.existsSync(p)) {
      browserPath = p;
      break;
    }
  }

  // Fallback to Microsoft Edge
  if (!browserPath) {
    for (const p of edgePaths) {
      if (fs.existsSync(p)) {
        browserPath = p;
        break;
      }
    }
  }

  if (browserPath) {
    console.log(`[Launcher] Launching standalone App Mode window using browser: ${browserPath}`);
    // Run the browser in borderless desktop app mode!
    const cmd = `"${browserPath}" --app="${url}" --start-maximized --no-first-run --no-default-browser-check`;
    
    exec(cmd, (err) => {
      if (err) {
        console.error('[Launcher] Failed to launch browser in app mode, falling back:', err);
        exec(`start ${url}`);
      }
    });
  } else {
    console.log(`[Launcher] Chrome/Edge app executables not detected. Defaulting to standard system browser: ${url}`);
    // Fallback: Open URL in default system browser using standard OS command
    exec(`start ${url}`);
  }
}

module.exports = { launchBrowser };
