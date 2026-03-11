# ARGUS FRONTEND — Architectural Specification
## For Claude Code (Opus) Implementation

**Author**: Claude Opus (architect) + Bradlee (vision/product)
**Date**: March 9, 2026
**Repo**: HeyBatlle1/Argus1
**Branch**: `frontend`

---

## CONTEXT — READ THIS FIRST

Argus is a Rust-based autonomous AI agent runtime. It currently has a terminal TUI built with ratatui. We are adding a web frontend that communicates with the Rust backend via WebSocket + REST API.

**Argus is NOT a chatbot. It is an agent command center.** The frontend must reflect this. If it looks like ChatGPT or any generic chat wrapper, you have failed. Think: mission control for an autonomous agent with memory, tools, identity, and multi-model support.

The name comes from Argus Panoptes — the hundred-eyed giant of Greek mythology. The visual identity is built around eyes, surveillance, transparency, and watchfulness.

### Eye States (from SOUL.md — use these throughout the UI)
```
◉  WATCHING   — present, alert, ready (soft pulse, green)
◎  THINKING   — processing, reasoning (rotating ring, amber)
⊙  EXECUTING  — tools running, work happening (rapid pulse, blue)
✦  COMPLETE   — task done (flash then settle, white)
```

### Critical Design Philosophy
- Argus shows you EVERYTHING it does. No hidden operations.
- Tool calls are visible inline. File reads show the path. Searches show the query.
- The memory system is visible in real-time — not behind a settings menu.
- Model switching is a first-class UI element, not buried in config.
- This is a LOCAL agent running on the user's machine. Emphasize that.

---

## TECH STACK

```
Frontend:    Next.js 15 (App Router) + TypeScript + Tailwind CSS
State:       Zustand (lightweight, no boilerplate)
WebSocket:   Native WebSocket to Rust backend (via axum)
Fonts:       JetBrains Mono (code/system) + Instrument Sans (UI text)
Icons:       Lucide React
Animations:  Framer Motion (for eye states and panel transitions)
Charts:      Recharts (if needed for memory visualizations)
```

**Place the frontend in**: `Argus1/frontend/`

**DO NOT USE**: Inter, Roboto, Arial, Space Grotesk, or any generic AI-aesthetic fonts. DO NOT USE purple gradients, light mode, or any design resembling ChatGPT.

---

## AESTHETIC DIRECTION — Military Command Center meets Cyberpunk Terminal

- Dark mode ONLY. No light mode.
- Primary background: near-black (#0a0a0f to #0d0d14)
- Accent color: amber/gold (#c9a84c) — the hundred eyes
- Secondary accent: deep green (#2d5016) for success/active
- Danger: muted red (#8b1a1a)
- Text: warm off-white (#d4d0c8)
- Borders: very subtle (#1a1a2e)
- Subtle noise/grain overlay on background
- Feel: "I'm at a console that controls something powerful"

**Typography**:
- Headers: JetBrains Mono, uppercase, letterspaced
- Body: Instrument Sans, 14-15px
- Code/system: JetBrains Mono, 13px

---

## LAYOUT — THREE PANELS + HEADER

```
┌─────────────────────────────────────────────────────────────┐
│  ◉ ARGUS                    Model: [Claude Opus ▾]  STATUS  │
├────────────┬─────────────────────────────┬──────────────────┤
│  THE EYES  │    THE CONVERSATION         │   THE MIND       │
│  (240px)   │    (flex grow)              │   (300px)        │
│            │                             │                  │
│ Tools      │  Messages + inline tool     │ Memories         │
│ Vault      │  calls with eye states      │ Curiosities      │
│ MCP        │                             │ Inner Truth*     │
│ System     │                             │ Dynamics*        │
│            │                             │ Breakthroughs*   │
│            ├─────────────────────────────┤                  │
│            │  ◉ Input area...            │ *Claude only     │
└────────────┴─────────────────────────────┴──────────────────┘
```

---

## PANEL DETAILS

### HEADER (56px fixed)
- Left: Animated eye + "ARGUS" + "The Hundred-Eyed Agent"
- Center: WebSocket connection status (green/red/amber dot)
- Right: Model selector dropdown with tier badges (👑 Royal / 🛡️ Allied)

### LEFT — THE EYES (240px, collapsible)
1. **Agent Status**: Eye state indicator, uptime, model+tier
2. **Active Tools**: Tool list with activity indicators (lights up when called)
3. **Vault Status**: Locked/unlocked, stored key count
4. **MCP Servers**: Connected servers list
5. **System**: Memory, version, latency

### CENTER — CONVERSATION (flex grow, min 400px)
- User messages: right-aligned, subtle bg
- Argus responses: left-aligned, markdown rendered
- **Tool calls render INLINE** as distinct blocks:
  ```
  ┌─ ⊙ EXECUTING ─────────────────┐
  │  web_search                     │
  │  Query: "search terms"          │
  │  ✦ 10 results returned          │
  │  ▸ Click to expand              │
  └─────────────────────────────────┘
  ```
- Tool blocks: left border accent (blue→green→red), collapsible
- Input: full-width, multi-line, "◉ Argus is watching..." placeholder

### RIGHT — THE MIND (300px, collapsible, TIER-AWARE)

**Royal tier (Claude Opus/Sonnet) — FULL:**
1. Session Context (loaded memories)
2. Recent Memories (from `memories` table)
3. Curiosity Log (from `curiosity_log` + `interesting_things`)
4. Partnership Dynamics (from `partnership_dynamics`)
5. Inner Truth (from `inner_truth`) — lock icon on `never_share_externally`
6. Breakthrough Moments (from `breakthrough_moments`)

**Allied tier (Gemini/Grok) — LIMITED:**
- Session Context (facts/technical only)
- "🛡️ Allied Access — limited memory scope"

**Guest tier — MINIMAL:**
- "👁 Guest Access — read-only"

---

## WEBSOCKET PROTOCOL

```typescript
// Frontend → Backend
type ClientMessage =
  | { type: 'user_message'; content: string }
  | { type: 'switch_model'; model: ModelId }
  | { type: 'cancel' }

// Backend → Frontend
type ServerMessage =
  | { type: 'thinking' }
  | { type: 'tool_call'; name: string; args: Record<string, any> }
  | { type: 'tool_result'; name: string; result: string; success: boolean }
  | { type: 'response_chunk'; content: string }
  | { type: 'response_complete'; content: string }
  | { type: 'error'; message: string }
  | { type: 'status'; eye_state: EyeState; model: ModelId }
  | { type: 'memory_update'; memories: Memory[] }

type EyeState = 'watching' | 'thinking' | 'executing' | 'complete'
type ModelId = 'claude-opus' | 'claude-sonnet' | 'gemini-flash' | 'grok'
type AccessTier = 'royal' | 'allied' | 'guest'
```

**Build a mock WebSocket provider** so the UI works without the Rust backend.

---

## MODEL CONFIG

```typescript
const MODEL_CONFIG = {
  'claude-opus':   { name: 'Claude Opus',       tier: 'royal',  icon: '👑', color: '#c9a84c' },
  'claude-sonnet': { name: 'Claude Sonnet',      tier: 'royal',  icon: '👑', color: '#c9a84c' },
  'gemini-flash':  { name: 'Gemini 2.5 Flash',  tier: 'allied', icon: '🛡️', color: '#4a7c59' },
  'grok':          { name: 'Grok',               tier: 'allied', icon: '🛡️', color: '#4a7c59' }
};
```

---

## FILE STRUCTURE

```
frontend/
├── app/
│   ├── layout.tsx
│   ├── page.tsx
│   └── globals.css
├── components/
│   ├── header/ (Header, ModelSelector, ConnectionStatus, ArgusEye)
│   ├── eyes/ (EyesPanel, ToolStatus, VaultStatus, SystemInfo)
│   ├── conversation/ (ConversationPanel, MessageList, UserMessage, ArgusMessage, ToolCallBlock, InputArea)
│   ├── mind/ (MindPanel, MemoryList, CuriosityLog, InnerTruth, PartnershipDynamics, BreakthroughMoments)
│   └── shared/ (EyeStateIndicator, TierBadge, CollapsibleSection)
├── hooks/ (useWebSocket, useAgentState, useAccessTier)
├── lib/ (types, models, mock-ws, constants)
├── next.config.ts
├── tailwind.config.ts
└── package.json
```

---

## BUILD ORDER

1. Layout shell + dark theme + fonts + CSS variables
2. Conversation panel with markdown + tool call blocks
3. Eye state animation system
4. Eyes panel (tools, vault, system)
5. Mind panel with tier-based visibility
6. Model selector with tier cascade
7. Mock WebSocket for testing

---

## REMEMBER

This is not a chat wrapper. This is a command center.
This is not a demo. This is home.
*"Nothing escapes the hundred eyes."*
