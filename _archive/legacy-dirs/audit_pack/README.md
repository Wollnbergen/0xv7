# Sultan Chain Audit Pack (Draft)

This directory aggregates production readiness artifacts for external security and infrastructure auditors.

## Contents
- `genesis.json` (copy here if needed) – current chain genesis state.
- Hardening scripts: `scripts/harden_sultan_node.sh`, `scripts/ddos_hardening.sh`.
- Zero-fee tests: `scripts/zero_gas_full_test.sh` (CW20 store/instantiate/transfer/mint under zero fees).
- Validator lifecycle: `scripts/validator_lifecycle_test.sh` (second validator, delegation, unbond, redelegate).
- Backup & DR: `scripts/backup_snapshot.sh`, `scripts/backup_restore.sh`.
- Security scans: `security/reports/panic_scan.json`, `security/reports/secret_scan.json`.

## Remaining Items
- Formal threat model (P2P eclipse, RPC DDoS, contract resource exhaustion).
- Panic/unwrap remediation PRs (convert to Result error handling).
- Reverse proxy rate limiting (Nginx) and fail2ban in deployment manifests.
- Parameter set capture (slashing, staking, gov) for audit.

## Ops Runbook (order)
1. Start chain (`START_SULTAN_CLEAN.sh`).
2. Apply hardening (`scripts/harden_sultan_node.sh` + `scripts/ddos_hardening.sh`).
3. Run zero-gas test (`scripts/zero_gas_full_test.sh`).
4. Establish second validator (`scripts/validator_lifecycle_test.sh`).
5. Snapshot (`scripts/backup_snapshot.sh`).
6. Generate security reports (`scripts/security_scan_panic.sh`, `scripts/security_scan_secrets.sh`).

## Contact
Add security contact email and GPG key in final audit revision.
