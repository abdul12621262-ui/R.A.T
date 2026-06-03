# Koyeb Deployment Guide

This guide explains how to deploy the R.A.T signaling server to Koyeb for free international access.

## Why Koyeb?

- **Free tier without credit card** ✓
- **Supports long-running services** ✓
- **WebSocket support** ✓
- **Global edge network** ✓
- **No sleep/spin-down** (always-on) ✓
- **Docker container support** ✓

## Prerequisites

- Koyeb account (sign up at [koyeb.com](https://koyeb.com))
- Git repository with your R.A.T code
- GitHub account (Koyeb connects via GitHub)

## Koyeb Free Tier Details

- **512 MB RAM**
- **1 vCPU**
- **Persistent storage**
- **Always-on service**
- **Global edge network**
- **No credit card required**

## Step 1: Prepare Your Repository

The deployment file is already created in the project root:
- `koyeb.yaml` - Koyeb-specific configuration
- `Dockerfile` - Docker build configuration

## Step 2: Deploy to Koyeb

### Via Koyeb Dashboard (Recommended)

1. Go to [koyeb.com](https://koyeb.com) and login
2. Click "Create App" or "New Service"
3. Connect your GitHub account
4. Select your R.A.T repository
5. Koyeb will auto-detect the `koyeb.yaml` file
6. Click "Create App"
7. Wait for build to complete (2-5 minutes)

### Configuration Details

Koyeb will use the settings from `koyeb.yaml`:
- **Service Name**: rat-signaling
- **Type**: Web Service
- **Port**: 4899 (TCP)
- **Environment**: PORT=4899
- **Build**: Dockerfile
- **Instance Type**: nano (free tier)
- **Region**: Washington D.C. (was)
- **Health Check**: /health on port 4899

## Step 3: Get Your Koyeb URL

Once deployed, Koyeb will provide a URL like:
`https://rat-signaling-xxx.koyeb.app`

## Step 4: Configure Your R.A.T Client

### Option 1: Environment Variable (Temporary)
```powershell
$env:RAT_SIGNALING_HOST = "rat-signaling-xxx.koyeb.app"
.\target\debug\rat.exe
```

### Option 2: Permanent Environment Variable
```powershell
[System.Environment]::SetEnvironmentVariable('RAT_SIGNALING_HOST', 'rat-signaling-xxx.koyeb.app', 'User')
```

### Option 3: Linux/Mac
```bash
export RAT_SIGNALING_HOST="rat-signaling-xxx.koyeb.app"
./target/debug/rat
```

## Step 5: Test Connection

1. Run R.A.T on Machine A (admin)
2. Click "Create Connection Code" → share the code
3. Run R.A.T on Machine B (joiner)
4. Enter only the code → click "Link Device"
5. Should connect internationally

## Alternative: Koyeb CLI

If you prefer command-line deployment:

1. Install Koyeb CLI:
```bash
# Windows (via scoop)
scoop install koyeb

# Or download from https://github.com/koyeb/koyeb-cli
```

2. Login:
```bash
koyeb login
```

3. Deploy:
```bash
koyeb app init rat-signaling
koyeb service create signaling --dockerfile Dockerfile --ports 4899:tcp --env PORT=4899
```

## Troubleshooting

### Build Fails
- Check Koyeb logs in the dashboard
- Ensure koyeb.yaml is in the project root
- Verify Dockerfile is correct
- Check Cargo.toml dependencies

### Connection Refused
- Ensure port 4899 is exposed in koyeb.yaml
- Check service is running (green status in dashboard)
- Verify health check is passing
- Check logs for startup errors

### Health Check Failing
- Ensure /health endpoint exists in signaling server
- Check health check path in koyeb.yaml
- Verify port configuration
- Check logs for errors

### Service Not Starting
- Check instance type is "nano" (free tier)
- Verify Dockerfile builds successfully locally
- Check Koyeb logs for errors
- Ensure Rust version is compatible

## Advantages Over Other Platforms

**vs Railway:**
- No credit card required
- No sleep/spin-down
- More generous free tier
- Better WebSocket support

**vs Render:**
- No credit card required
- Always-on service
- Global edge network
- Better for signaling servers

**vs Self-host:**
- No router configuration
- No port forwarding
- No dynamic DNS needed
- Professional hosting
- Global availability

## Monitoring

Koyeb provides built-in monitoring:
- CPU usage
- Memory usage
- Network traffic
- Request logs
- Error logs

Access these in the Koyeb dashboard under your service.

## Scaling

If you need more resources (paid):
- Upgrade instance type (micro, small, medium, etc.)
- Add more instances
- Enable auto-scaling
- Choose different regions

## Security

- Koyeb provides HTTPS automatically
- Your signaling server is protected
- Session codes provide authentication
- Monitor logs for suspicious activity
- Consider adding authentication for production

## Next Steps

1. Deploy to Koyeb using the steps above
2. Test the connection between two devices
3. Monitor the service in Koyeb dashboard
4. Consider upgrading if you need more resources

## Support

- Koyeb docs: [koyeb.com/docs](https://koyeb.com/docs)
- Docker on Koyeb: [koyeb.com/docs/docker](https://koyeb.com/docs/docker)
- GitHub integration: [koyeb.com/docs/github](https://koyeb.com/docs/github)

## Comparison with Other Options

| Platform | Free Tier | Credit Card | Always-on | WebSocket | Best For |
|----------|-----------|-------------|------------|-----------|----------|
| **Koyeb** | ✓ | No | ✓ | ✓ | Signaling servers |
| Railway | ✓ | Yes | No | ✓ | Quick prototypes |
| Render | ✓ | Yes | No | ✓ | Web apps |
| Vercel | ✓ | Yes | No | ✗ | Static sites |
| Self-host | ✓ | No | ✓ | ✓ | Advanced users |

**Koyeb is the best choice for R.A.T signaling server** because it supports WebSockets, is always-on, and doesn't require a credit card.
