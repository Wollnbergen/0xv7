# Sultan Website Updates - December 9, 2025

## Changes Required for Replit Website (index.html)

### 1. Fix Meta Description (Line 5)
**Old:**
```html
<meta name="description" content="Sultan Chain: The first zero-fee blockchain with 26.67% validator APY...">
```

**New:**
```html
<meta name="description" content="Sultan Chain: The first zero-fee blockchain with 13.33% validator APY. Built with Rust, secured by 9 global validators.">
```

### 2. Update "View Manual Setup Instructions" Button
In the `generateSetup()` function, update the setup instructions to link to the new page:

**Add this line in the modal/button section:**
```html
<a href="validator-setup.html" class="btn btn-primary" style="width: 100%;">View Complete Setup Guide</a>
```

### 3. Upload New File
Upload `validator-setup.html` to the Replit website (created in this session).

---

## New Validator Setup Page Features

The new `validator-setup.html` includes:

1. **Step-by-step guide** with 6 clear steps
2. **Provider recommendations** with pricing:
   - DigitalOcean ($6/mo)
   - Hetzner (€4.51/mo) 
   - Vultr ($5/mo)
   - AWS Lightsail ($5/mo)

3. **Copy-paste commands** for:
   - Downloading the binary
   - Starting the validator
   - Setting up systemd service
   - Troubleshooting

4. **Earnings calculator** showing:
   - 1,333 SLTN/year
   - 111 SLTN/month
   - 3.65 SLTN/day
   (at 10,000 SLTN stake)

5. **Correct economics:**
   - 13.33% APY (fixed)
   - 4% inflation (fixed forever)

---

## Current Validator Network

| Provider | Name | Location | IP |
|----------|------|----------|-----|
| DigitalOcean | NYC | US East | 192.241.154.140 (Bootstrap) |
| DigitalOcean | SFO | US West | 143.198.67.237 |
| DigitalOcean | AMS | Amsterdam | 188.166.102.7 |
| DigitalOcean | LON | London | 159.65.88.145 |
| DigitalOcean | FRA | Frankfurt | 159.65.113.160 |
| DigitalOcean | SGP | Singapore | 188.166.218.123 |
| Hetzner | NBG | Nuremberg | 116.203.92.158 |
| Hetzner | HEL | Helsinki | 77.42.35.238 |
| Hetzner | FSN | Falkenstein | 49.13.26.15 |

**Bootstrap peer for new validators:**
```
/ip4/192.241.154.140/tcp/26656
```
