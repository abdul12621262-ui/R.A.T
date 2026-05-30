# Render.com Deployment Guide

This guide explains how to deploy the R.A.T signaling server to Render.com for free international access.

## Prerequisites

- Render account (sign up at [render.com](https://render.com))
- Git repository with your R.A.T code
- GitHub account (Render connects via GitHub)

## Step 1: Prepare Your Repository

The deployment file is already created in the project root:
- `render.yaml` - Render-specific configuration

## Step 2: Deploy to Render

### Via Render Dashboard (Recommended)

1. Go to [render.com](https://render.com) and login
2. Click "New +" → "Web Service"
3. Connect your GitHub account
4. Select your R.A.T repository
5. Render will automatically detect the `render.yaml` file
6. Click "Create Web Service"
7. Wait for build to complete (2-5 minutes)

### Configuration Details

Render will use the settings from `render.yaml`:
- **Type**: Web Service
- **Environment**: Rust
- **Build Command**: `cargo build --release -p rat-signaling`
- **Start Command**: `./target/release/rat-signaling`
- **Port**: 4899
- **Plan**: Free

## Step 3: Get Your Render URL

Once deployed, Render will provide a URL like:
`https://rat-signaling.onrender.com`

## Step 4: Configure Your R.A.T Client

### Option 1: Environment Variable (Temporary)
```powershell
$env:RAT_SIGNALING_HOST = "rat-signaling.onrender.com"
.\target\debug\rat.exe
```

### Option 2: Permanent Environment Variable
```powershell
[System.Environment]::SetEnvironmentVariable('RAT_SIGNALING_HOST', 'rat-signaling.onrender.com', 'User')
```

### Option 3: Linux/Mac
```bash
export RAT_SIGNALING_HOST="rat-signaling.onrender.com"
./target/debug/rat
```

## Step 5: Test Connection

1. Run R.A.T on Machine A (admin)
2. Click "Create Connection Code" → share the code
3. Run R.A.T on Machine B (joiner)
4. Enter only the code → click "Link Device"
5. Should connect internationally

## Render Free Tier Details

- **Free credit**: Included with free tier
- **Builds**: Unlimited
- **CPU**: 512 MB RAM
- **Storage**: Not applicable for signaling server
- **Bandwidth**: 100 GB/month (sufficient for signaling)
- **Sleep**: Free services spin down after 15 min of inactivity
- **Wake up**: ~30 seconds when accessed

**Note**: Free services spin down after inactivity. For a signaling server, this means:
- First connection attempt may take ~30 seconds
- Service stays awake while active
- Spins down after 15 minutes of no connections

## Troubleshooting

### Build Fails
- Check Render logs in the dashboard
- Ensure render.yaml is in the project root
- Verify Cargo.toml dependencies are correct
- Make sure the build command matches your project structure

### Connection Refused
- Ensure port 4899 is exposed (configured in render.yaml)
- Check Render service is running (green status in dashboard)
- Verify firewall allows outbound connections from your client

### Service Spins Down
- This is normal for free tier
- First connection takes ~30 seconds to wake up
- Service stays awake while active
- Consider upgrading to paid tier for always-on service

### Health Check Failing
- Render doesn't require health checks for web services
- The signaling server has a `/health` endpoint
- If issues, check logs for startup errors

## Advantages Over Railway

- **More generous free tier**: Better for long-running services
- **No credit system**: No surprise charges
- **Simpler pricing**: Predictable costs
- **Better documentation**: Clearer setup process
- **GitHub integration**: Seamless deployment

## Security Notes

- The signaling server uses WebSocket for real-time communication
- Session codes provide basic authentication
- Consider adding authentication for production use
- Monitor logs for suspicious activity
- Use HTTPS (Render provides this automatically)

## Next Steps

1. Deploy to Render using the steps above
2. Test the connection between two devices
3. Monitor the service in Render dashboard
4. Consider upgrading to paid tier if needed for always-on service

## Support

- Render docs: [docs.render.com](https://docs.render.com)
- Rust on Render: [docs.render.com/docs/rust](https://docs.render.com/docs/rust)
