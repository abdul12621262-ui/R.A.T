# Railway Deployment Tutorial for R.A.T Signaling Server

This guide explains how to deploy the R.A.T signaling server to Railway for free international access.

## Prerequisites

- Railway account (sign up at [railway.app](https://railway.app))
- Git repository with your R.A.T code
- Railway CLI (optional, but recommended)

## Step 1: Prepare Your Repository

The deployment files are already created in `server/signaling/`:
- `Dockerfile` - Container configuration
- `railway.toml` - Railway-specific settings
- `.dockerignore` - Files to exclude from Docker build

## Step 2: Deploy to Railway

### Option A: Via Railway CLI (Recommended)

1. Install Railway CLI:
```bash
npm install -g @railway/cli
```

2. Login to Railway:
```bash
railway login
```

3. Initialize Railway project:
```bash
cd d:\R.A.T
railway init
```

4. Deploy from the signaling directory:
```bash
cd server/signaling
railway up
```

5. Railway will build and deploy your signaling server. Once complete, you'll get a public URL like:
   `https://your-project-name.railway.app`

### Option B: Via Railway Dashboard

1. Go to [railway.app](https://railway.app) and login
2. Click "New Project" → "Deploy from GitHub repo"
3. Select your R.A.T repository
4. Railway will detect the Dockerfile in `server/signaling/`
5. Click "Deploy"
6. Wait for build to complete (2-5 minutes)

## Step 3: Configure Your R.A.T Client

Once deployed, get your Railway URL and configure the client:

### Option 1: Environment Variable (Recommended)
```powershell
$env:RAT_SIGNALING_HOST = "your-project-name.railway.app"
.\target\debug\rat.exe
```

### Option 2: Permanent Environment Variable
```powershell
[System.Environment]::SetEnvironmentVariable('RAT_SIGNALING_HOST', 'your-project-name.railway.app', 'User')
```

### Option 3: Update Code
Edit `apps/ui/src/agent_bridge.rs` lines 33 and 50, replace the placeholder with your Railway URL.

## Step 4: Test the Connection

1. Start the signaling server on Railway (it auto-starts on deploy)
2. Run R.A.T on your admin machine
3. Click "Create Connection Code"
4. Share the code with your joiner
5. Joiner enters only the code (no address needed)
6. Connection should work internationally

## Railway Free Tier Details

- **Free credit**: $5/month (enough for small signaling server)
- **Builds**: 500 hours/month
- **CPU**: 512 MB RAM
- **Storage**: 1 GB
- **Bandwidth**: 100 GB/month (sufficient for signaling)

## Troubleshooting

### Build Fails
- Check Railway logs in the dashboard
- Ensure Dockerfile is in `server/signaling/` directory
- Verify Cargo.toml dependencies are correct

### Connection Refused
- Ensure port 4899 is exposed (configured in railway.toml)
- Check Railway service is running (green status in dashboard)
- Verify firewall allows outbound connections from your client

### Health Check Failing
- The signaling server has a `/health` endpoint
- Railway checks this to ensure service is running
- If failing, check logs for startup errors

## Custom Domain (Optional)

To use a custom domain instead of Railway's default:

1. In Railway dashboard, go to your project
2. Click "Settings" → "Networking"
3. Add your custom domain (e.g., `signaling.yourdomain.com`)
4. Update DNS records as instructed by Railway
5. Update `RAT_SIGNALING_HOST` to your custom domain

## Monitoring

- View logs in Railway dashboard under "Logs" tab
- Monitor usage under "Usage" tab
- Set up alerts in "Settings" → "Notifications"

## Scaling

If you need more capacity:
- Upgrade to Railway Pro plan ($20/month)
- Or deploy to Oracle Cloud Free Tier (see main README)

## Security Notes

- Railway provides HTTPS automatically
- The signaling server uses WebSocket over HTTPS (wss://)
- Session codes are temporary (10 minutes TTL)
- End-to-end encryption is handled by the agent layer

## Support

For Railway-specific issues:
- Railway docs: [docs.railway.app](https://docs.railway.app)
- Railway Discord: [discord.gg/railway](https://discord.gg/railway)

For R.A.T-specific issues:
- Check the main README.md
- Review signaling server logs in Railway dashboard
