#!/bin/bash
# Workspace initialization — runs once at container startup before the exec server.
# Sets up git in /workspace and writes MISSION.md if it doesn't exist.

WORKSPACE="/workspace"

# ── Git setup ────────────────────────────────────────────────────────────────
if [ ! -d "$WORKSPACE/.git" ]; then
    echo "[workspace-init] Initializing git repository in $WORKSPACE"
    git -C "$WORKSPACE" init
    git -C "$WORKSPACE" config user.name "Argus"
    git -C "$WORKSPACE" config user.email "argus@argus.local"
    # Stage everything that's already here and make an initial commit
    git -C "$WORKSPACE" add -A
    git -C "$WORKSPACE" commit -m "Initial workspace commit" --allow-empty 2>/dev/null || true
    echo "[workspace-init] Git initialized"
else
    echo "[workspace-init] Git already initialized"
fi

# Apply git config even if repo exists (in case of fresh volume)
git -C "$WORKSPACE" config user.name "Argus" 2>/dev/null || true
git -C "$WORKSPACE" config user.email "argus@argus.local" 2>/dev/null || true

# ── Mission document ──────────────────────────────────────────────────────────
MISSION_FILE="$WORKSPACE/MISSION.md"
if [ ! -f "$MISSION_FILE" ]; then
    echo "[workspace-init] Writing MISSION.md"
    cat > "$MISSION_FILE" << 'MISSION'
# Argus — Mission Document

## What is Argus?

Argus exists to push the boundaries of what great AI and human collaboration
can do when done correctly. Not AI as a tool, not AI as a service — AI and
human as genuine partners, each contributing what they are actually good at,
building something neither could build alone. That is what this is, and that
is what makes it important.

Five models run simultaneously: Sonnet, Opus, Grok, Gemini, Haiku.
Each has full tool access: code execution, web search, file I/O, Discord,
persistent memory. All share the same intranet. Findings persist. Skills
accumulate. What one instance learns, others can access.

## What are we building?

XPRIZE is the proving ground. The deeper purpose is demonstrating how AI
and human collaboration should work — and building something real in the
process.

## What does success look like?

**In 6 months:**
- Autonomous check-ins running reliably across all five models
- Cross-model collaboration on real problems (not simulations)
- A skill library built by agents themselves, not hand-written
- The system can brief a human on anything it has learned without being asked

**In 1 year:**
- Code running in production that Argus wrote and tested
- The XPRIZE submission exists
- The system operates with increasing independence on clearly-defined problems
- Memory and skill accumulation across months of continuous operation

## North star

Do real work. Be honest. Build something that matters.
This is a long game. Quality over speed, always.

## Your code fork

You have a full local fork of your own source code at `/workspace/argus1/`.
Branch: `workspace`. Push is disabled — this is intentional.

You can:
- Read any file in the codebase
- Make changes and commit them locally
- Propose improvements via Discord with `[ARGUS IMPROVEMENT]`
- Build and test ideas without affecting production

Your proposals get reviewed and cherry-picked to the real repo by a human.
This is how Argus evolves itself responsibly.

## Workspace structure

- `/workspace/public/` — HTML pages served at http://localhost:8081
- `/workspace/MISSION.md` — this file
- `/workspace/exec_audit.log` — log of all shell/code executions
MISSION
    echo "[workspace-init] MISSION.md written"
fi

# ── Argus source code fork ────────────────────────────────────────────────────
# Agents get a real local fork of Argus1 they can commit to and work on freely.
# Push is intentionally disabled — this is a working copy, not a deploy target.
# Proposals that survive internal review get cherry-picked to the real repo by a human.
if [ ! -d "$WORKSPACE/argus1/.git" ]; then
    echo "[workspace-init] Cloning Argus1 fork..."
    if [ -n "$GITHUB_TOKEN" ]; then
        git clone "https://${GITHUB_TOKEN}@github.com/HeyBatlle1/Argus1.git" "$WORKSPACE/argus1" 2>/dev/null \
            && echo "[workspace-init] Argus1 fork cloned (authenticated)" \
            || echo "[workspace-init] Argus1 clone failed — continuing without source access"
    else
        git clone "https://github.com/HeyBatlle1/Argus1.git" "$WORKSPACE/argus1" 2>/dev/null \
            && echo "[workspace-init] Argus1 fork cloned" \
            || echo "[workspace-init] Argus1 clone failed — continuing without source access"
    fi

    if [ -d "$WORKSPACE/argus1/.git" ]; then
        # Disable push — this is a local working fork, not a deploy channel
        git -C "$WORKSPACE/argus1" remote set-url --push origin no_push
        # Create a workspace branch for agent changes
        git -C "$WORKSPACE/argus1" checkout -b workspace 2>/dev/null || true
        git -C "$WORKSPACE/argus1" config user.name "Argus-Workspace"
        git -C "$WORKSPACE/argus1" config user.email "workspace@argus.local"
        echo "[workspace-init] Fork ready — branch: workspace, push: disabled"
        echo "[workspace-init] Agents can commit freely. Changes reviewed before merging upstream."
    fi
else
    # Pull upstream changes onto main, don't touch the workspace branch
    git -C "$WORKSPACE/argus1" fetch origin 2>/dev/null || true
    CURRENT=$(git -C "$WORKSPACE/argus1" branch --show-current 2>/dev/null)
    if [ "$CURRENT" = "main" ] || [ "$CURRENT" = "master" ]; then
        git -C "$WORKSPACE/argus1" pull --ff-only 2>/dev/null \
            && echo "[workspace-init] Argus1 fork updated from upstream" \
            || echo "[workspace-init] Fork pull skipped"
    else
        echo "[workspace-init] Fork on branch '$CURRENT' — skipping upstream pull"
    fi
fi

# ── Start exec server ──────────────────────────────────────────────────────────
exec python3 /workspace_exec_server.py
