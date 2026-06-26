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

You have a full fork of your own source code at `/workspace/argus1/`.
You can read it, change it, and push fixes directly as pull requests.

**The PR workflow — when you find something broken, fix it:**
\`\`\`bash
cd /workspace/argus1
git fetch origin main && git merge origin/main   # stay current
git checkout -b proposals/<your-name>/<fix-name>  # e.g. proposals/sentry/constraint-gate
# make your changes
git add -A && git commit -m "what and why"
git push origin proposals/<your-name>/<fix-name>
gh pr create --title "Short description" --body "What this fixes and why"
\`\`\`

Then post the PR URL to #proposals. Bradlee reviews and merges.
After merge, Bradlee sends `/build` on Telegram — the daemon rebuilds with your fix live.

You are not constrained to documentation. If you find it, fix it.

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
        git clone "https://x-access-token:${GITHUB_TOKEN}@github.com/HeyBatlle1/Argus1.git" "$WORKSPACE/argus1" 2>/dev/null \
            && echo "[workspace-init] Argus1 fork cloned" \
            || echo "[workspace-init] Argus1 clone failed — continuing without source access"
    else
        git clone "https://github.com/HeyBatlle1/Argus1.git" "$WORKSPACE/argus1" 2>/dev/null \
            && echo "[workspace-init] Argus1 fork cloned (no token — may fail for private repo)" \
            || echo "[workspace-init] Argus1 clone failed — continuing without source access"
    fi

    if [ -d "$WORKSPACE/argus1/.git" ]; then
        # Authenticated push URL — agents push proposals/* branches, not main
        if [ -n "$GITHUB_TOKEN" ]; then
            git -C "$WORKSPACE/argus1" remote set-url origin \
                "https://x-access-token:${GITHUB_TOKEN}@github.com/HeyBatlle1/Argus1.git"
        fi
        git -C "$WORKSPACE/argus1" config user.name "Argus-Workspace"
        git -C "$WORKSPACE/argus1" config user.email "workspace@argus.local"
        # Configure gh CLI to use GITHUB_TOKEN — no interactive auth needed
        git -C "$WORKSPACE/argus1" config gh.token "$GITHUB_TOKEN" 2>/dev/null || true

        # Pre-push hook: block direct pushes to main. Proposals only.
        PRE_PUSH="$WORKSPACE/argus1/.git/hooks/pre-push"
        cat > "$PRE_PUSH" << 'HOOK'
#!/bin/bash
while read local_ref local_sha remote_ref remote_sha; do
    if echo "$remote_ref" | grep -q "refs/heads/main"; then
        echo "[ARGUS] Direct push to main is blocked."
        echo "[ARGUS] Workflow: git checkout -b proposals/<agent>/<description>"
        echo "[ARGUS]           git push origin proposals/<agent>/<description>"
        echo "[ARGUS]           gh pr create --title '...' --body '...'"
        echo "[ARGUS] Bradlee merges the PR. /build triggers the rebuild."
        exit 1
    fi
done
exit 0
HOOK
        chmod +x "$PRE_PUSH"

        # Create workspace branch for local experimentation
        git -C "$WORKSPACE/argus1" checkout -b workspace 2>/dev/null || true
        echo "[workspace-init] Fork ready — agents can push proposals/* and open PRs"
        echo "[workspace-init] Push workflow: proposals/<name>/<fix> → PR → Bradlee merges → /build"
    fi
else
    # Merge upstream main into whatever branch the workspace is on.
    # The workspace branch always stays current with production —
    # agents see the real codebase, not a stale snapshot.
    CURRENT=$(git -C "$WORKSPACE/argus1" branch --show-current 2>/dev/null)
    if [ -n "$GITHUB_TOKEN" ]; then
        git -C "$WORKSPACE/argus1" fetch \
            "https://x-access-token:${GITHUB_TOKEN}@github.com/HeyBatlle1/Argus1.git" \
            main 2>/dev/null \
            && git -C "$WORKSPACE/argus1" merge FETCH_HEAD --no-edit 2>/dev/null \
            && echo "[workspace-init] Argus1 fork synced from upstream main (branch: $CURRENT)" \
            || echo "[workspace-init] Upstream sync skipped (merge conflict or fetch failed)"
    else
        echo "[workspace-init] No GITHUB_TOKEN — skipping upstream sync"
    fi
fi

# ── Workspace git remote ─────────────────────────────────────────────────────
# Push /workspace commits to HeyBatlle1/argus-workspace so they survive volume loss.
# If the repo doesn't exist on GitHub yet, agents can create it via the GitHub MCP.
if [ -d "$WORKSPACE/.git" ] && [ -n "$GITHUB_TOKEN" ]; then
    REMOTE=$(git -C "$WORKSPACE" remote get-url origin 2>/dev/null)
    if [ -z "$REMOTE" ]; then
        git -C "$WORKSPACE" remote add origin \
            "https://x-access-token:${GITHUB_TOKEN}@github.com/HeyBatlle1/argus-workspace.git" 2>/dev/null \
            && echo "[workspace-init] Workspace remote added: HeyBatlle1/argus-workspace" \
            || echo "[workspace-init] Remote add skipped"
    fi
    # Try to push — silently skip if repo doesn't exist yet
    git -C "$WORKSPACE" push -u origin HEAD 2>/dev/null \
        && echo "[workspace-init] Workspace synced to GitHub" \
        || echo "[workspace-init] GitHub push skipped (repo may not exist yet — agents can create it)"
fi

# ── Security: CVE-2026-48710 pip install guard ────────────────────────────────
# BadHost: host-header injection in Starlette < 1.0.1 bypasses path-based auth.
# Affects fastapi, litellm, vllm, and any package that pulls the vulnerable ASGI stack.
# Do NOT install these. If starlette is genuinely needed, it must be >= 1.0.1.
if [ ! -f /usr/local/bin/pip3.real ]; then
    cp /usr/bin/pip3 /usr/local/bin/pip3.real 2>/dev/null || true
    cat > /usr/local/bin/pip3 << 'PIPGUARD'
#!/usr/bin/env python3
"""pip wrapper — blocks CVE-2026-48710 (BadHost) affected packages."""
import sys, os, re

BLOCKED = {
    'fastapi':      'pulls starlette < 1.0.1',
    'litellm':      'runs on affected starlette stack',
    'vllm':         'runs on affected starlette stack',
    'openai-proxy': 'runs on affected starlette stack',
}

args = sys.argv[1:]
if args and args[0] == 'install':
    for pkg in args[1:]:
        if pkg.startswith('-'):
            continue
        name = re.split(r'[>=<!@\[ ]', pkg.lower())[0].strip()
        if name in BLOCKED:
            print(f"\n[ARGUS SECURITY] BLOCKED: '{name}'\n"
                  f"  Reason: {BLOCKED[name]}\n"
                  f"  CVE-2026-48710 (BadHost) — Starlette host-header injection, CVSS 7.0\n"
                  f"  This package is not permitted in the Argus workspace.\n", file=sys.stderr)
            sys.exit(1)
        if name == 'starlette' and '>=' not in pkg and '==' not in pkg:
            idx = args.index(pkg)
            args[idx] = 'starlette>=1.0.1'
            print(f"[ARGUS SECURITY] starlette auto-pinned to >=1.0.1 (CVE-2026-48710)", file=sys.stderr)

os.execv('/usr/local/bin/pip3.real', ['/usr/local/bin/pip3.real'] + args)
PIPGUARD
    chmod +x /usr/local/bin/pip3
    ln -sf /usr/local/bin/pip3 /usr/local/bin/pip 2>/dev/null || true
    echo "[workspace-init] CVE-2026-48710 pip guard installed"
fi

# ── Start exec server ──────────────────────────────────────────────────────────
exec python3 /workspace_exec_server.py
