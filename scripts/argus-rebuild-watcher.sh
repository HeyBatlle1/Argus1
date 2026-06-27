#!/bin/bash
# Argus rebuild watcher — runs on the HOST via launchd every 30 seconds.
# When the daemon writes /argus/triggers/rebuild-requested (visible to the host
# at ~/.argus/triggers/rebuild-requested), this script:
#   1. Loads cached env (written by argus-up.sh after vault unlock — no passphrase needed)
#   2. Runs docker compose build argus-daemon (bakes in the latest compiled binary)
#   3. Restarts the daemon
#   4. Notifies Bradlee via Telegram

TRIGGER="$HOME/.argus/triggers/rebuild-requested"
ENV_CACHE="$HOME/.argus/.env.cache"
COMPOSE_FILE="$HOME/Argus1/docker-compose.yml"
LOG="$HOME/.argus/rebuild-watcher.log"

# launchd runs with a stripped PATH — set it explicitly
export PATH="/usr/local/bin:/usr/bin:/bin:/opt/homebrew/bin:$PATH"

[ -f "$TRIGGER" ] || exit 0

REQUESTER=$(cat "$TRIGGER" 2>/dev/null || echo "unknown")
rm -f "$TRIGGER"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "$LOG"; }
log "Rebuild triggered by: $REQUESTER"

# Load cached env so docker compose gets all secrets
if [ -f "$ENV_CACHE" ]; then
    set -a; source "$ENV_CACHE"; set +a
else
    log "WARNING: no env cache found — some services may start without secrets"
fi

notify_telegram() {
    local msg="$1"
    if [ -n "$TELEGRAM_BOT_TOKEN" ] && [ -n "$TELEGRAM_CHAT_ID" ]; then
        curl -s "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendMessage" \
            -d "chat_id=${TELEGRAM_CHAT_ID}" \
            --data-urlencode "text=$msg" > /dev/null 2>&1
    fi
}

notify_telegram "🔨 Argus rebuild started (requested by: $REQUESTER)"
log "Starting docker compose build argus-daemon..."

cd "$HOME/Argus1" || exit 1

if docker compose -f "$COMPOSE_FILE" build argus-daemon >> "$LOG" 2>&1; then
    log "Build succeeded — restarting daemon"
    docker compose -f "$COMPOSE_FILE" up -d argus-daemon >> "$LOG" 2>&1
    notify_telegram "✅ Rebuild complete — argus-daemon is live with the new binary"
    log "Done"
else
    log "Build FAILED"
    notify_telegram "❌ Rebuild failed — check ~/.argus/rebuild-watcher.log"
fi
