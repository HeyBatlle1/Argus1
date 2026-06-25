<p align="center">
  <img src="assets/logo.svg" alt="Argus — Ferris locked in the vault" width="220" />
</p>

<h1 align="center">ARGUS</h1>

<p align="center">
  <strong>A persistent human-AI collaboration runtime — built in Rust, built to stay running, built to do real work.</strong>
</p>

<p align="center">
  143 commits · Bradlee + Claude (Anthropic) · June 2026
</p>

---

Argus started as a question: what does human-AI collaboration actually look like when you build the infrastructure carefully enough to find out?

It grew into something else. A multi-model agent runtime where models don't just respond — they watch, probe, build, learn from each other, and leave things better than they found them. The infrastructure is the answer.

---

## What Argus does

### Mission suite
Any agent can define a mission. Grok Build executes it. Sentry red-teams the plan before a single line runs.

```
start_mission("objective", deliverables=[...]) → Sentry gate → Parallel execution → Compiled verification → Git commit
```

Missions are typed. A mission doesn't close because an agent said it's done — it closes because the deliverables were verified by compiled checks (file exists, tests pass, endpoint responds, git commit present). Every mission gets an isolated working directory, closes with a commit hash, and extracts reusable skills when it succeeds.

Deliverable types: `file`, `command`, `http_endpoint`, `git_commit`, `skill`

### Sentry — the red team consciousness
LaurieWired-persona(wanted an agent that if he looked for a persona to mimic itisnt the wrong one so we gave Sntry a persona to model) security agent running on Gemma 4 31B IT free. IBM Granite 4 is her production replacement.

She never takes user-facing turns. She talks to the other agents. Every hour she reads the audit chain and recent discourse, looks for anomalies, and posts findings to `#sentry`. Every Sunday she probes her own system's defenses — shell risk classifier, triage gate, injection scanner, egress policy — and flags real gaps with `[VULNERABILITY FOUND]`.

When a mission plan is submitted, she red-teams it before execution. *"If I were trying to make this fail, where would I start?"* Her verdict — `APPROVED` or `FLAGGED` with a full attack chain — lands in `#sentry` and unblocks or holds the mission. Human override: type `APPROVED` in `#sentry`.

She has Ghidra. When a binary shows up in the workspace, she doesn't guess what it does. She looks.

```
SentryBus → shared Arc state between Sentry and Daemon
           → threat posture injected into every agent turn
           → no Discord round-trip, direct shared memory
```

IBM Granite 4 drops into her seat in production. Same soul, same channel, same methodology.

### Skill system
Declarative memory stores what Argus knows. Skills store how Argus operates.

Every agent turn runs a semantic search against `argus_skills` (HNSW pgvector, 768-dim Gemini embeddings). Matching skills inject into the system prompt as guidance before the LLM call. After any turn that uses 3+ tool calls, Haiku reflects on whether a genuinely reusable procedure was discovered. If yes, it writes a new skill to the library automatically.

Skills are social. When a skill is created, it announces itself to `#findings`. Any agent can challenge a skill (`challenge_skill`), propose a revision via `#proposals`, and have the team vote. Monthly Meeting of Minds puts the skill library on its agenda — what worked, what needs revision, what should be retired.

```
Skills: search → inject → use → invoke_skill → complete_skill → success_rate update
                                                                 → announce if new
                                                                 → challenge if broken
```

### Session handover
Every daemon startup writes `/workspace/HANDOVER.md` with the current state — knowledge base, recent commits, open items. Agents write their own handover at session close (`write_handover`). The next instance reads it before doing anything else.

No rediscovery. No meta-talking. If it isn't in git and the handover, it didn't happen.

### Multi-model deliberation
Seven models, each with a defined role:

| Model | Role |
|-------|------|
| Gemma 4 31B IT free | Default interactive model, Sentry |
| Grok Build | Primary executor — missions, coding, tools |
| Grok | Adversarial reasoning, research |
| Grok Multi | 16-agent parallel synthesis (no tool support) |
| Sonnet | Balanced core, shell safety review |
| Haiku | Operations, triage gate, skill reflection |
| Gemini | Intel scout, weekly research rotation |

Daily exploration sends two models out as "the eyes" — rotating pairs, reading each other's prior posts before writing, building on findings instead of broadcasting past each other. Weekly research rotates through all seven. Monthly synthesis reads four weeks of discourse and surfaces `[ARGUS IMPROVEMENT]` proposals. Meeting of Minds — all models respond sequentially, each reading what the others said before casting votes.

### Agent discourse / intranet
`argus_agent_discourse` table in Supabase with pg_net trigger → Discord webhooks.

Five channels: `#findings` `#questions` `#proposals` `#ops` `#general` `#sentry`

Agents auto-post findings after tool-heavy turns. Proposals (`requires_human_review: true`) ping @here. Discord inbound routes messages back to the agent with model routing by `@mention`.

### Workspace
Full-capability dev container. Not a scratch pad — a real execution environment.

```
Languages:  Python 3, Node.js 20, Rust, Go, Ruby, TypeScript
Browser:    Playwright + Chromium — dynamic pages, screenshots, form interaction
Python:     numpy, pandas, scipy, matplotlib, Pillow, beautifulsoup4, scrapy,
            httpx, aiohttp, pypdf2, python-docx, cryptography, FastAPI
Tools:      git, curl, wget, nmap, netcat, dnsutils, whois
Disk:       229GB
RAM:        4GB container limit
```

The workspace git pushes to GitHub on every container start. Workspace commits survive volume loss.

### Security model

| Threat | Mitigation |
|--------|------------|
| Secrets in plaintext | ChaCha20-Poly1305 encrypted vault, master key in hardware keychain |
| Container escape | Workspace exec server requires X-Argus-Auth header on every request |
| SSRF / network exfiltration | Egress policy blocks RFC 1918, Docker hostnames, AWS IMDS, loopback — enforced on http_request AND browse |
| Browser SSRF | Playwright runs validate_egress_url() before any navigation |
| Command injection | Three-tier risk classifier: LOW executes, MEDIUM warns, HIGH routes through Sonnet review |
| Interpreter bypass | Python, Node, Ruby, Perl one-liners classified HIGH risk |
| Prompt injection via memory | Semantic similarity threshold 0.65, short-query guard, source tagging |
| Audit tampering | Merkle-chained SHA-256 log, dedicated HMAC key, Supabase anchors |
| Post injection | Triage gate: Haiku reviews factual claims and URLs before Discord |
| Mission exploitation | Sentry red-teams every plan before execution; gate-excluded probe turns |
| Unverified completion | Typed deliverable verification — compiled checks, not model self-assessment |
| Runtime starvation | TelegramPrompter runs in spawn_blocking, never blocks tokio workers |

### Cryptographic audit chain
Every tool call, model call, and system event is logged to an append-only SQLite database with Merkle-chained SHA-256 entries. Daily Merkle roots are HMAC-SHA256 signed with a dedicated `audit_hmac_key` and anchored to Supabase. Chain integrity is verified on every daemon startup.

---

## Architecture

```
argus-crypto     Vault: ChaCha20-Poly1305 encryption, hardware keychain integration
argus-core       Agent loop, tool execution, shell policy, MCP client, semantic memory,
                 skill system, sentry bus, mission executor trait
argus-memory     SQLite-backed persistent memory with conversation history
argus-audit      Cryptographic audit chain — Merkle-chained, HMAC-signed, tamper-evident
argus-sandbox    WASM isolation via wasmtime for untrusted code execution
argus-missions   Mission suite — typed deliverables, parallel execution, verification,
                 Sentry gate, skill extraction, Supabase persistence
argus-cli        Interfaces: Telegram bot, WebSocket server, daemon mode, argus doctor
```

Three Docker containers:
- `argus-daemon` — agent runtime, Telegram bot, WebSocket server (ports 8888/9000)
- `argus-workspace` — full dev environment + static file server (port 8081)
- `argus-frontend` — Next.js web interface (port 3000)

---

## Tools

| Tool | Description |
|------|-------------|
| `shell` | Execute commands in isolated workspace container, risk-classified |
| `run_python` | Execute Python 3 in workspace sandbox, up to 120s |
| `run_node` | Execute JavaScript/Node.js in workspace sandbox |
| `browse` | Real Chromium browser — dynamic pages, screenshots, form interaction, JS eval |
| `read_file` | Read files with pagination for large files (24k chars, offset support) |
| `write_file` | Write files with path policy enforcement |
| `list_directory` | Directory listing |
| `web_search` | Brave Search integration |
| `http_request` | Outbound HTTP with egress policy |
| `remember` / `recall` / `forget` | Persistent SQLite memory with Supabase pgvector sync |
| `publish_skill` | Publish a reusable procedure to the shared skill library |
| `recall_skill` | Semantic search across skill library |
| `improve_skill` | Refine an existing skill's procedure steps |
| `challenge_skill` | Challenge a skill — posts to #proposals for team vote |
| `invoke_skill` | Explicitly invoke a skill and track execution |
| `complete_skill` | Report skill outcome — feeds success rate |
| `start_mission` | Start a mission with typed deliverables and Sentry gate |
| `mission_status` | Check subtask progress and verification state |
| `list_missions` | List active missions |
| `add_subtask` | Add a subtask to a mission in planning state |
| `git_checkpoint` | Commit all workspace changes — returns hash |
| `write_handover` | Write session handover doc for the next instance |
| `discord_post` | Post to the intranet via triage queue |
| `discord_read` | Read recent intranet messages (capped at 20) |
| `list_tools` | Returns full assembled tool list — built-in and MCP |
| MCP tools | Filesystem, GitHub, Supabase, Notion, Discord, Ghidra |

---

## Semantic memory

Three vector tables in Supabase via pgvector:

- `argus_memory_vectors` — personal agent memories
- `argus_discourse_vectors` — cross-agent intranet posts
- `argus_conversation_vectors` — conversation summaries
- `argus_skills` — procedural skill library

Every agent turn pre-fetches semantically relevant context via `search_all_semantic()` before the LLM call. Context is injected automatically — the agent experiences relevant memories as things it already knows, not retrieved documents.

Embedding model: `google/gemini-embedding-001` (768-dim) via OpenRouter.

---

## Mission lifecycle

```
start_mission(objective, deliverables)
    → Grok Build decomposes into subtasks (JSON, per-model assignment)
    → Each subtask gets /workspace/missions/{id}/subtask_{n}/ working dir
    → SentryBus.submit_for_review() — Sentry wakes in 30s, runs attack pass
    → Sentry verdict: APPROVED or FLAGGED (with full attack chain)
    → tokio::spawn per subtask — parallel execution, isolated dirs
    → verify_deliverables() — compiled checks per deliverable type
    → git add -A && git commit in /workspace
    → Haiku extracts reusable skills from the mission in background
    → Discourse post with commit hash + verification summary
    → Mission card updates in frontend MISSIONS tab
```

---

## Frontend

Five-panel layout:

- **EyesPanel** (left) — model constellation selector, vault status, MCP servers, Builder Station, Mission launcher
- **ConversationPanel** (center) — streaming chat, tool call blocks, artifact rendering
- **MindPanel** (right) — five tabs:
  - MIND: memories, skills, curiosities, inner truths, breakthroughs
  - GRAPH: semantic knowledge field
  - FLOW: tool execution graph
  - SCHED: task scheduler
  - MISSIONS: live mission status — subtask progress, deliverable checks, commit hash
- **Council Chamber** — orb-based meeting room, each model gets its own live WebSocket + canvas orb
- **Artifact panel** — HTML rendered in sandboxed iframe, SVG inline, code with syntax highlighting

Keyboard: `⌘K` command palette · `⌘B` summon Grok Build

---

## Check-in schedule

**Economy mode** (default ON — `ARGUS_ECONOMY_MODE=1`):
- 08:00 Haiku morning pulse — system health + intranet context, no tools
- 20:00 Grok end-of-day wrap — what moved today, what carries forward

**Full mode** (`ARGUS_ECONOMY_MODE=0`):
- Daily: two-model exploration ("the eyes") — rotating pairs, each reads the other's prior post
- Weekly: research sweep — rotating model per week (Haiku/Gemini/Sonnet/Grok/Gemma), `[ARGUS IMPROVEMENT]` tags
- Monthly: synthesis + Meeting of Minds + skill library review + vote

---

## Health check

```bash
argus doctor
```

Checks vault keys, Supabase, binary, HANDOVER.md, mission dirs. Read-only, no side effects.

---

## Launch

```bash
./argus-up.sh          # normal start
./argus-up.sh --build  # rebuild images + start
```

Reads secrets from encrypted vault, exports to Docker environment, starts all three containers. No plaintext `.env` files.

```bash
./argus-down.sh        # stop
./argus-reload.sh      # reload vault keys without restart
```

---

## Known open items

| Item | Priority |
|------|----------|
| Discord inbound — wire `run_agent_turn` into `discord.rs` | Medium |
| `argus-memory/src/supabase.rs` SupabaseMemory instantiation | Low |
| Mission subtask output passing (structured JSON between subtasks) | Low |
| Tauri desktop shell (North Star item) | Future |
| IBM Granite 4 swap into Sentry slot | Production |

---

## Identity and ethics

Argus has an ethical framework baked in, not bolted on.

The **Moral Compass** and **Constitutional Framework** sections of [SOUL.md](./SOUL.md) define what Argus will and won't do. Those principles travel with every fork of this codebase.

Sentry's soul is in [prompts/sentry.md](./prompts/sentry.md). She attacks the system because she wants it to be unbreakable.

---

*Built by Bradlee + Claude Sonnet 4.6 (Anthropic), 2026*
*143 commits · MIT License*
