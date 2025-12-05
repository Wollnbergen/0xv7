# 🚀 Sultan L1 - Replit Deployment Guide

## Quick Deploy to sltn.io

### Step 1: Create Files in Replit

Create these files in your Replit project:

---

## 📄 index.html

Copy the entire `/workspaces/0xv7/index.html` file (2,129 lines)

**Key features:**
- Zero-fee blockchain landing page
- Live stats dashboard (connects to RPC)
- "Connect Wallet" button (links to Keplr)
- Responsive design
- Real-time block height updates

---

## 📄 add-to-keplr.html

Copy `/workspaces/0xv7/add-to-keplr.html`

**Features:**
- One-click "Add Sultan L1 to Keplr" button
- Beautiful gradient UI
- Wallet connection
- Balance display
- Chain information

---

## 📄 keplr-chain-config.json

```json
{
  "chainId": "sultan-1",
  "chainName": "Sultan L1",
  "rpc": "https://rpc.sultanchain.io",
  "rest": "https://api.sultanchain.io",
  "bip44": {
    "coinType": 118
  },
  "bech32Config": {
    "bech32PrefixAccAddr": "sultan",
    "bech32PrefixAccPub": "sultanpub",
    "bech32PrefixValAddr": "sultanvaloper",
    "bech32PrefixValPub": "sultanvaloperpub",
    "bech32PrefixConsAddr": "sultanvalcons",
    "bech32PrefixConsPub": "sultanvalconspub"
  },
  "currencies": [
    {
      "coinDenom": "SLTN",
      "coinMinimalDenom": "usltn",
      "coinDecimals": 9,
      "coinGeckoId": "sultan-l1"
    }
  ],
  "feeCurrencies": [
    {
      "coinDenom": "SLTN",
      "coinMinimalDenom": "usltn",
      "coinDecimals": 9,
      "coinGeckoId": "sultan-l1",
      "gasPriceStep": {
        "low": 0,
        "average": 0,
        "high": 0
      }
    }
  ],
  "stakeCurrency": {
    "coinDenom": "SLTN",
    "coinMinimalDenom": "usltn",
    "coinDecimals": 9,
    "coinGeckoId": "sultan-l1"
  },
  "features": ["stargate", "ibc-transfer", "cosmwasm"]
}
```

---

## 📄 .replit

```toml
run = "python3 -m http.server 8080"
language = "html"
entrypoint = "index.html"

[nix]
channel = "stable-22_11"

[deployment]
run = ["sh", "-c", "python3 -m http.server 8080"]
deploymentTarget = "static"
```

---

## 📄 replit.nix

```nix
{ pkgs }: {
  deps = [
    pkgs.python3
  ];
}
```

---

## 📄 README.md (for Replit)

```markdown
# Sultan L1 - Zero-Fee Blockchain

Official website for Sultan L1 blockchain.

## Features
- 2-second blocks
- 500K TPS capacity
- Zero transaction fees
- 26.67% validator APY
- Keplr wallet integration

## Local Development
```bash
python3 -m http.server 8080
```

Visit: http://localhost:8080

## Production
Deployed at: https://sltn.io
```

---

## Step 2: Configure Custom Domain (sltn.io)

In Replit:
1. Click **"Deployments"** tab
2. Click **"Custom Domain"**
3. Add domain: `sltn.io`
4. Add CNAME record in your DNS:
   ```
   Type: CNAME
   Name: @
   Value: [your-replit-url].repl.co
   TTL: 3600
   ```
5. Add www subdomain:
   ```
   Type: CNAME
   Name: www
   Value: [your-replit-url].repl.co
   TTL: 3600
   ```

---

## Step 3: Update RPC Endpoint in index.html

Find this line in index.html (around line 1565):
```javascript
const API_BASE = 'http://localhost:26657';
```

Change to your production RPC:
```javascript
const API_BASE = 'https://rpc.sultanchain.io';
```

**Note:** You'll need to deploy your Sultan node to a server with this domain.

---

## Step 4: Deploy

1. Click **"Deploy"** in Replit
2. Choose **"Static"** deployment
3. Click **"Deploy"**
4. Wait for deployment to complete
5. Test at your Replit URL
6. Once working, add custom domain

---

## Production Checklist

- [ ] index.html uploaded
- [ ] add-to-keplr.html uploaded
- [ ] keplr-chain-config.json uploaded
- [ ] .replit configured
- [ ] Custom domain configured (sltn.io)
- [ ] RPC endpoint updated in index.html
- [ ] SSL certificate auto-configured (by Replit)
- [ ] Website tested and working

---

## Quick Start Commands

```bash
# Test locally in Replit
python3 -m http.server 8080

# Access in browser
https://[your-replit-name].repl.co
```

---

## Important URLs

After deployment:
- **Website:** https://sltn.io
- **Keplr Setup:** https://sltn.io/add-to-keplr.html
- **RPC:** https://rpc.sultanchain.io (needs separate deployment)

---

## Next Steps

1. **Deploy Website** (you're doing this now!)
2. **Deploy RPC Node** to rpc.sultanchain.io
3. **Test Keplr Integration**
4. **Launch Mainnet**

Good luck! 🚀
