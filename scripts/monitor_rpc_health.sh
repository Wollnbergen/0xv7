#!/bin/bash
# =============================================================================
# RPC Health Monitor - Runs as cron job to detect issues proactively
# =============================================================================
# Add to crontab: */5 * * * * /path/to/scripts/monitor_rpc_health.sh
# =============================================================================

set -e

LOG_FILE="/var/log/sultan-rpc-health.log"
ALERT_FILE="/tmp/sultan-rpc-alert-sent"
SLACK_WEBHOOK="${SULTAN_SLACK_WEBHOOK:-}"  # Set in environment if using Slack

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

alert() {
    log "🚨 ALERT: $1"
    
    # Slack notification (if configured)
    if [ -n "$SLACK_WEBHOOK" ]; then
        curl -s -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"🚨 Sultan RPC Alert: $1\"}" \
            "$SLACK_WEBHOOK" >/dev/null
    fi
    
    # Avoid spam - only alert once per hour
    touch "$ALERT_FILE"
}

should_alert() {
    if [ ! -f "$ALERT_FILE" ]; then
        return 0
    fi
    # Alert file older than 1 hour?
    if [ $(find "$ALERT_FILE" -mmin +60 2>/dev/null | wc -l) -gt 0 ]; then
        rm -f "$ALERT_FILE"
        return 0
    fi
    return 1
}

# =============================================================================
# Check 1: CORS Headers (duplicate header issue)
# =============================================================================
check_cors() {
    log "Checking CORS headers..."
    
    HEADERS=$(curl -s -D - "https://rpc.sltn.io/status" -H "Origin: https://sltn.io" -o /dev/null 2>&1)
    CORS_COUNT=$(echo "$HEADERS" | grep -ci "access-control-allow-origin" || true)
    
    if [ "$CORS_COUNT" -eq 0 ]; then
        if should_alert; then
            alert "No CORS headers - browsers will fail"
        fi
        return 1
    elif [ "$CORS_COUNT" -gt 1 ]; then
        if should_alert; then
            alert "Duplicate CORS headers ($CORS_COUNT) - run ./scripts/fix_rpc_cors.sh"
        fi
        return 1
    fi
    
    log "  ✅ CORS OK (single header)"
    return 0
}

# =============================================================================
# Check 2: RPC Responding
# =============================================================================
check_rpc_response() {
    log "Checking RPC response..."
    
    RESPONSE=$(curl -s --connect-timeout 5 "https://rpc.sltn.io/status" 2>&1)
    
    if [ -z "$RESPONSE" ]; then
        if should_alert; then
            alert "RPC not responding at all"
        fi
        return 1
    fi
    
    HEIGHT=$(echo "$RESPONSE" | jq -r '.height // empty' 2>/dev/null)
    
    if [ -z "$HEIGHT" ]; then
        if should_alert; then
            alert "RPC returning invalid JSON"
        fi
        return 1
    fi
    
    log "  ✅ RPC OK (height: $HEIGHT)"
    return 0
}

# =============================================================================
# Check 3: Block Production (chain not stalled)
# =============================================================================
check_block_production() {
    log "Checking block production..."
    
    STATE_FILE="/tmp/sultan-last-height"
    CURRENT_HEIGHT=$(curl -s "https://rpc.sltn.io/status" | jq -r '.height' 2>/dev/null)
    
    if [ -f "$STATE_FILE" ]; then
        LAST_HEIGHT=$(cat "$STATE_FILE")
        LAST_TIME=$(stat -c %Y "$STATE_FILE" 2>/dev/null || stat -f %m "$STATE_FILE")
        NOW=$(date +%s)
        ELAPSED=$((NOW - LAST_TIME))
        
        # If 5+ minutes passed and height unchanged, chain is stalled
        if [ "$ELAPSED" -gt 300 ] && [ "$CURRENT_HEIGHT" = "$LAST_HEIGHT" ]; then
            if should_alert; then
                alert "Chain stalled at height $CURRENT_HEIGHT for ${ELAPSED}s"
            fi
            return 1
        fi
    fi
    
    echo "$CURRENT_HEIGHT" > "$STATE_FILE"
    log "  ✅ Blocks progressing (height: $CURRENT_HEIGHT)"
    return 0
}

# =============================================================================
# Check 4: SSL Certificate Expiry
# =============================================================================
check_ssl() {
    log "Checking SSL certificate..."
    
    EXPIRY=$(echo | openssl s_client -servername rpc.sltn.io -connect rpc.sltn.io:443 2>/dev/null | \
             openssl x509 -noout -enddate 2>/dev/null | cut -d= -f2)
    
    if [ -z "$EXPIRY" ]; then
        log "  ⚠️ Could not check SSL"
        return 0
    fi
    
    EXPIRY_EPOCH=$(date -d "$EXPIRY" +%s 2>/dev/null || date -j -f "%b %d %T %Y %Z" "$EXPIRY" +%s 2>/dev/null)
    NOW=$(date +%s)
    DAYS_LEFT=$(( (EXPIRY_EPOCH - NOW) / 86400 ))
    
    if [ "$DAYS_LEFT" -lt 7 ]; then
        if should_alert; then
            alert "SSL certificate expires in $DAYS_LEFT days!"
        fi
        return 1
    fi
    
    log "  ✅ SSL OK (expires in $DAYS_LEFT days)"
    return 0
}

# =============================================================================
# Main
# =============================================================================
log "========== RPC Health Check =========="

FAILURES=0

check_cors || FAILURES=$((FAILURES + 1))
check_rpc_response || FAILURES=$((FAILURES + 1))
check_block_production || FAILURES=$((FAILURES + 1))
check_ssl || FAILURES=$((FAILURES + 1))

if [ "$FAILURES" -eq 0 ]; then
    log "✅ All checks passed"
    rm -f "$ALERT_FILE"  # Clear alert state on success
else
    log "❌ $FAILURES check(s) failed"
fi

log "======================================"
