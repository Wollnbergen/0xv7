# Sultan Validator Earnings & Infrastructure TODO

## 🚨 CRITICAL: Validator APY Earnings

**Current Status:** Validators are earning APY (13.33%) but rewards are NOT being distributed yet.

**Action Required:** Once wallet infrastructure is set up:
1. Configure reward distribution wallet addresses for each validator
2. Implement automatic reward distribution mechanism
3. Set up reward claiming or auto-compound options

---

## Current Validator Network (as of Dec 7, 2025)

### Hetzner (Germany) - 11 Validators
- **IP:** 5.161.225.96
- **SSH:** `ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96`
- **Validators:** validator_0 (10,000 stake) + validator_1-10 (5,000 stake each)
- **Total Stake:** 60,000 SLTN

### DigitalOcean - 4 Validators

| Location | IP | Validator ID | Stake | SSH |
|----------|-----|--------------|-------|-----|
| NYC | 159.223.129.2 | validator_nyc | 5,000 | `ssh -i ~/.ssh/sultan_do root@159.223.129.2` |
| SFO | 64.23.148.205 | validator_sfo | 5,000 | `ssh -i ~/.ssh/sultan_do root@64.23.148.205` |
| AMS | 159.223.213.180 | validator_ams | 5,000 | `ssh -i ~/.ssh/sultan_do root@159.223.213.180` |
| SGP | 165.232.161.55 | validator_sgp | 5,000 | `ssh -i ~/.ssh/sultan_do root@165.232.161.55` |

**Total DO Stake:** 20,000 SLTN

---

## Network Totals

| Metric | Value |
|--------|-------|
| Total Validators | 15 |
| Total Staked | 80,000 SLTN |
| Validator APY | 13.33% |
| Annual Rewards (estimated) | ~21,336 SLTN |
| Monthly Cost (DO) | ~$24/month |
| Monthly Cost (Hetzner) | ~€4.50/month |

---

## Pending Infrastructure Tasks

### High Priority
- [ ] **Wallet Infrastructure** - Set up validator reward wallets
- [ ] **Reward Distribution** - Implement APY payout mechanism
- [ ] **Monitoring/Alerting** - Set up Grafana/Prometheus or uptime monitoring
- [ ] **DNS Configuration** - Distribute RPC endpoints geographically

### Medium Priority
- [ ] **Auto-compound Option** - Allow validators to auto-stake rewards
- [ ] **Slashing Conditions** - Implement validator penalty system
- [ ] **Validator Dashboard** - Web UI for validator operators

### Low Priority
- [ ] **Backup/Recovery** - Automated backup scripts
- [ ] **Log Rotation** - Prevent disk fill from validator logs

---

## API Credentials (ROTATE THESE!)

⚠️ **DigitalOcean Token:** `dop_v1_8c1699477f5c42c71e36f62a17f7b7edb361707608ca8d724778e45be4710166`

**Note:** Consider rotating this token after initial setup is complete.

---

## Quick Commands

```bash
# Check network status
curl -s https://rpc.sltn.io/status | jq '{height, validator_count}'

# Check Hetzner bootstrap logs
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 "tail -20 /root/sultan/sultan-node.log"

# Check DO validator logs
ssh -i ~/.ssh/sultan_do root@159.223.129.2 "tail -20 /root/sultan-validator.log"

# Restart all Hetzner validators
ssh -i ~/.ssh/sultan_hetzner root@5.161.225.96 "systemctl restart sultan-node && for i in {1..10}; do systemctl restart sultan-validator-\$i; done"
```

---

*Last Updated: December 7, 2025*
