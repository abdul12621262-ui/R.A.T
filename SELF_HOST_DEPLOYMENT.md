# Self-Host Deployment Guide

This guide explains how to self-host the R.A.T signaling server on your home machine for free international access.

## Overview

Self-hosting means running the signaling server on your own computer and making it accessible from the internet using port forwarding and dynamic DNS. This is completely free and gives you full control.

## Prerequisites

- R.A.T project built on your machine
- Router with port forwarding capability
- Dynamic DNS service account (free)
- Windows machine (for this guide)

## Step 1: Run the Signaling Server Locally

First, verify the signaling server runs on your machine:

```powershell
cd d:\R.A.T
cargo run --release -p rat-signaling
```

You should see:
```
INFO rat_signaling: R.A.T signaling listening on 0.0.0.0:4899
```

Keep this terminal open - the server needs to stay running.

## Step 2: Set Up Port Forwarding

Port forwarding allows external connections to reach your signaling server through your router.

### Find Your Router's IP

1. Open Command Prompt
2. Run: `ipconfig`
3. Look for "Default Gateway" (usually 192.168.1.1 or 192.168.0.1)

### Access Router Settings

1. Open web browser
2. Enter router IP (e.g., http://192.168.1.1)
3. Login with router credentials (check router label or manual)

### Configure Port Forwarding

1. Find "Port Forwarding" or "Virtual Server" section
2. Add new rule:
   - **External Port**: 4899
   - **Internal Port**: 4899
   - **Protocol**: TCP
   - **Internal IP**: Your computer's local IP (run `ipconfig` to find IPv4 Address)
3. Save/Apply changes

### Test Port Forwarding

1. On your machine, visit: [https://www.yougetsignal.com/tools/open-ports/](https://www.yougetsignal.com/tools/open-ports/)
2. Enter port: 4899
3. Should show "Port 4899 is open"

## Step 3: Set Up Dynamic DNS

Your home IP address changes periodically. Dynamic DNS gives you a consistent domain name.

### Create DuckDNS Account (Free)

1. Go to [duckdns.org](https://www.duckdns.org)
2. Sign up with email
3. Create a subdomain (e.g., `my-rat-signaling.duckdns.org`)

### Install DuckDNS Updater

**Option 1: DuckDNS Desktop App**
1. Download from [duckdns.org](https://www.duckdns.org/domains)
2. Install and login
3. It will automatically update your IP

**Option 2: Windows Task Scheduler (Manual)**
1. Create batch file `update-dns.bat`:
```batch
@echo off
curl "https://www.duckdns.org/update?domains=your-subdomain&token=your-token&ip="
```
2. Set up Windows Task Scheduler to run this every 5 minutes

### Verify Dynamic DNS

1. Visit your DuckDNS subdomain in browser
2. Should show connection refused (normal - no web server)
3. This confirms DNS is working

## Step 4: Configure R.A.T Client

Now configure your R.A.T client to use your self-hosted server:

### Option 1: Temporary
```powershell
$env:RAT_SIGNALING_HOST = "your-subdomain.duckdns.org:4899"
.\target\debug\rat.exe
```

### Option 2: Permanent
```powershell
[System.Environment]::SetEnvironmentVariable('RAT_SIGNALING_HOST', 'your-subdomain.duckdns.org:4899', 'User')
```

## Step 5: Test International Connection

1. **Admin Machine (Your computer):**
   - Run R.A.T with self-hosted server configured
   - Click "Create Connection Code"
   - Share the code with friend

2. **Joiner Machine (Friend's computer):**
   - Configure R.A.T with your DuckDNS URL
   - Enter the session code
   - Click "Link Device"
   - Should connect internationally

## Step 6: Keep Server Running

The signaling server must stay running for connections to work.

### Option 1: Run in Background (Windows)

```powershell
Start-Process -NoNewWindow cargo -ArgumentList "run --release -p rat-signaling"
```

### Option 2: Run as Windows Service (Advanced)

Use tools like NSSM (Non-Sucking Service Manager) to run as a service.

### Option 3: Run on Startup

1. Create shortcut to `cargo run --release -p rat-signaling`
2. Place in `shell:startup` folder
3. Server starts automatically on boot

## Firewall Configuration

If connections fail, check Windows Firewall:

1. Open Windows Defender Firewall
2. Go to "Allow an app through firewall"
3. Allow "cargo" or "rat-signaling" on both private and public networks
4. Or create inbound rule for port 4899:
   - Open Windows Defender Firewall with Advanced Security
   - Inbound Rules → New Rule
   - Port → TCP → 4899 → Allow → Name it "R.A.T Signaling"

## Troubleshooting

### Port Forwarding Not Working

- Verify router has port forwarding enabled
- Check firewall isn't blocking port 4899
- Ensure computer's local IP is correct
- Restart router after changes

### Dynamic DNS Not Updating

- Verify DuckDNS token is correct
- Check updater is running
- Manually test update URL in browser
- Check router allows DuckDNS access

### Connection Refused

- Ensure signaling server is running
- Check port forwarding is active
- Verify firewall allows port 4899
- Test with `telnet your-subdomain.duckdns.org 4899`

### Server Stops When Computer Sleeps

- Disable sleep on your machine
- Or use wake-on-LAN
- Or run on a dedicated device (Raspberry Pi)

## Security Considerations

- Your signaling server is now publicly accessible
- Session codes provide basic authentication
- Monitor logs for suspicious activity
- Consider adding authentication for production use
- Use strong DuckDNS password

## Advantages

- **Completely free** - no hosting costs
- **Full control** - you manage everything
- **No credit card required**
- **Unlimited bandwidth** (limited by your internet)
- **Privacy** - data stays on your machine

## Disadvantages

- **Requires router configuration**
- **Depends on your internet uptime**
- **Computer must stay on**
- **Limited by upload speed**
- **Security responsibility**

## Alternative: Dedicated Device

For better reliability, consider running on:
- Raspberry Pi (~$35)
- Old laptop/desktop
- Always-on device

This allows the server to run 24/7 without affecting your main computer.

## Next Steps

1. Set up port forwarding
2. Configure DuckDNS
3. Run signaling server
4. Test connection with friend
5. Consider running on dedicated device for reliability

## Support

- DuckDNS docs: [duckdns.org](https://www.duckdns.org)
- Port forwarding guides: [portforward.com](https://portforward.com)
