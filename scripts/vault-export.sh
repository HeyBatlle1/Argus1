#!/bin/bash
# Load Argus secrets from the encrypted host vault into the current shell.
# Usage: source scripts/vault-export.sh
# Requires: ARGUS_BINARY set to the release argus binary path.

_vault_get() {
  "$ARGUS_BINARY" vault get "$1" 2>/dev/null || echo ""
}

load_argus_secrets() {
  if [ -z "${ARGUS_BINARY:-}" ] || [ ! -f "$ARGUS_BINARY" ]; then
    echo "  [!] ARGUS_BINARY not set or missing — cannot load vault"
    return 1
  fi

  echo ""
  echo "  [*] Unlocking vault and exporting secrets..."

  export OPENROUTER_API_KEY=$(_vault_get openrouter_api_key)
  export TELEGRAM_BOT_TOKEN=$(_vault_get telegram_bot_token)
  export SUPABASE_ARGUS_URL=$(_vault_get supabase_argus_url)
  export SUPABASE_ARGUS_SERVICE_KEY=$(_vault_get supabase_argus_service_key)
  export TELEGRAM_CHAT_ID=$(_vault_get telegram_chat_id)
  export WORKSPACE_EXEC_TOKEN=$(_vault_get workspace_exec_token)
  export BRAVE_SEARCH_API_KEY=$(_vault_get brave_search_api_key)
  export DISCORD_BOT_TOKEN=$(_vault_get discord_bot_token)
  export DISCORD_CHANNEL_ID=$(_vault_get discord_channel_id)
  export GITHUB_TOKEN=$(_vault_get github_token)

  export ARGUS_TRIAGE_ACTIVE=$( [ -n "$SUPABASE_ARGUS_URL" ] && [ -n "$SUPABASE_ARGUS_SERVICE_KEY" ] && echo "1" || echo "0" )
  export ARGUS_DISCORD_ACTIVE=$( [ -n "$DISCORD_BOT_TOKEN" ] && echo "1" || echo "0" )

  if [ -z "$OPENROUTER_API_KEY" ]; then
    echo "  [!] OPENROUTER_API_KEY not found in vault — cannot start"
    echo "  [!] Store it:  $ARGUS_BINARY vault set openrouter_api_key <key>"
    return 1
  fi

  ARGUS_WS_TOKEN=$(_vault_get argus_ws_token)
  if [ -z "$ARGUS_WS_TOKEN" ]; then
    ARGUS_WS_TOKEN=$(openssl rand -hex 32)
    "$ARGUS_BINARY" vault set argus_ws_token "$ARGUS_WS_TOKEN" 2>/dev/null || true
    echo "  [+] Generated new WS auth token — stored in vault"
  fi
  export ARGUS_WS_TOKEN

  echo "  [+] OpenRouter   ✓"
  [ -n "$TELEGRAM_BOT_TOKEN" ]       && echo "  [+] Telegram     ✓" || echo "  [-] Telegram     not set (optional)"
  [ -n "$DISCORD_BOT_TOKEN" ]        && echo "  [+] Discord      ✓" || echo "  [-] Discord      not set (optional)"
  [ -n "$SUPABASE_ARGUS_URL" ]       && echo "  [+] Supabase     ✓" || echo "  [-] Supabase     not set (optional)"
  [ -n "$BRAVE_SEARCH_API_KEY" ]     && echo "  [+] Brave Search ✓" || echo "  [-] Brave Search not set (optional)"
  [ -n "$GITHUB_TOKEN" ]             && echo "  [+] GitHub       ✓" || echo "  [-] GitHub       not set (optional)"
  echo "  [+] WS token     ✓ (runtime — no frontend rebuild needed)"

  # Cache env for the rebuild watcher — lets /build trigger rebuild without
  # re-entering the vault passphrase. File is owner-read-only (600).
  mkdir -p ~/.argus/triggers && chmod 700 ~/.argus
  {
    echo "OPENROUTER_API_KEY=$OPENROUTER_API_KEY"
    echo "TELEGRAM_BOT_TOKEN=$TELEGRAM_BOT_TOKEN"
    echo "SUPABASE_ARGUS_URL=$SUPABASE_ARGUS_URL"
    echo "SUPABASE_ARGUS_SERVICE_KEY=$SUPABASE_ARGUS_SERVICE_KEY"
    echo "TELEGRAM_CHAT_ID=$TELEGRAM_CHAT_ID"
    echo "WORKSPACE_EXEC_TOKEN=$WORKSPACE_EXEC_TOKEN"
    echo "BRAVE_SEARCH_API_KEY=$BRAVE_SEARCH_API_KEY"
    echo "DISCORD_BOT_TOKEN=$DISCORD_BOT_TOKEN"
    echo "DISCORD_CHANNEL_ID=$DISCORD_CHANNEL_ID"
    echo "GITHUB_TOKEN=$GITHUB_TOKEN"
    echo "ARGUS_WS_TOKEN=$ARGUS_WS_TOKEN"
    echo "ARGUS_ECONOMY_MODE=${ARGUS_ECONOMY_MODE:-1}"
  } > ~/.argus/.env.cache
  chmod 600 ~/.argus/.env.cache
  echo "  [+] Env cached   ✓ (rebuild watcher ready)"

  return 0
}