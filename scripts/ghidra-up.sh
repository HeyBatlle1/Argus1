#!/bin/bash
# Start Ghidra in headless mode with GhidraMCP HTTP server on port 8080.
# Sentry uses this as her private reverse engineering tool.
# The bridge (ghidra_bridge.py) connects to port 8080 and translates to MCP stdio.
#
# Usage:
#   ./ghidra-up.sh              — start Ghidra headless + MCP bridge
#   ./ghidra-up.sh --bridge-only — restart bridge only (if Ghidra already running)

GHIDRA_HOME="/Users/burtonstuff/ghidra"
GHIDRA_PROJECTS="/Users/burtonstuff/.argus/ghidra-projects"
BRIDGE="/Users/burtonstuff/Argus1/scripts/ghidra_bridge.py"
PORT=8080

mkdir -p "$GHIDRA_PROJECTS"

if [[ "$1" == "--bridge-only" ]]; then
    echo "[ghidra] Starting MCP bridge only on port $PORT..."
    python3 "$BRIDGE" &
    echo "[ghidra] Bridge PID: $!"
    exit 0
fi

echo "[ghidra] Starting Ghidra headless + GhidraMCP on port $PORT..."

# Run Ghidra headless with the GhidraMCP plugin active.
# -import accepts a dummy file — GhidraMCP just needs Ghidra running.
"$GHIDRA_HOME/support/analyzeHeadless" \
    "$GHIDRA_PROJECTS" SentryProject \
    -noanalysis \
    -postScript GhidraMCPPlugin.java \
    -scriptPath "$GHIDRA_HOME/Ghidra/Extensions" \
    -log /tmp/ghidra-sentry.log &

GHIDRA_PID=$!
echo "[ghidra] Ghidra PID: $GHIDRA_PID"

# Give Ghidra a few seconds to start the HTTP server before bridge connects
sleep 5

echo "[ghidra] Starting MCP bridge..."
python3 "$BRIDGE" &
BRIDGE_PID=$!
echo "[ghidra] Bridge PID: $BRIDGE_PID"
echo "[ghidra] Sentry's Ghidra is live on port $PORT"
echo "[ghidra] Log: /tmp/ghidra-sentry.log"
