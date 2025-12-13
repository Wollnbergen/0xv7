# Sultan L1 - Replit Website

Simple one-page website for Sultan L1 blockchain.

## How to Deploy on Replit

1. **Create New Repl:**
   - Go to https://replit.com
   - Click "Create Repl"
   - Choose "HTML, CSS, JS" template
   - Name it "sultan-l1"

2. **Upload Files:**
   - Delete default files
   - Upload `index.html` from this folder
   - Upload `.replit` file

3. **Run:**
   - Click "Run" button
   - Your site will be live at `https://sultan-l1.USERNAME.repl.co`

4. **Make it Always On (Optional):**
   - Upgrade to Replit Hacker plan
   - Enable "Always On" to keep it running 24/7

## Features

- ✅ Clean, modern design
- ✅ Live network stats
- ✅ Responsive (mobile-friendly)
- ✅ Links to GitHub SDK
- ✅ Links to full website
- ✅ Real-time block height simulation

## Customization

To connect to your real blockchain RPC:

1. Uncomment the `fetchNetworkStatus()` function in the script
2. Replace `YOUR_RPC_URL` with your actual RPC endpoint
3. Make sure CORS is enabled on your node

## Alternative: Static HTML Server

If you just want to test locally:

```bash
cd replit-website
python3 -m http.server 8080
# Visit http://localhost:8080
```

## What's Included

- **index.html** - Complete one-page website
- **.replit** - Replit configuration
- **README.md** - This file

## Live Links

- **SDK Repository:** https://github.com/Wollnbergen/BUILD
- **Full Website:** https://wollnbergen.github.io/SULTAN
- **Chain ID:** sultan-1
