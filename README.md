# 👁 ARGUS

**The Hundred-Eyed Agent Runtime**

A secure, local-first AI agent built in Rust. Named after Argus Panoptes, the all-seeing giant of Greek mythology.

```
        ╭──◉──╮
       ╭┤◉ ◉ ◉├╮
      ◉│ ╭───╮ │◉
      ◉│ │◉ ◉│ │◉
      ◉│ │ ▽ │ │◉
       │ ╰───╯ │
    ◉──┤ ◉ ◉ ◉ ├──◉
   ╭───┤       ├───╮
   │◉◉◉│ ◉ ◉ ◉ │◉◉◉│
   ╰───┤       ├───╯
       │ ◉   ◉ │
       ╰┬─┴─┬─╯
        │   │
       ─┴─ ─┴─
```

## Why Argus?

We watched the AI agent space build everything in JavaScript, store secrets in plaintext, and act surprised when it all caught fire.

So we built it in Rust. With real crypto. For adults.

## Features

- 🔐 **Encrypted Vault** — ChaCha20-Poly1305 encryption for all secrets
- 🔑 **Hardware Keychain** — Master keys stored in macOS Keychain / Windows Credential Manager / Linux Secret Service
- 🦀 **Memory Safe** — Written in Rust, not hoping JavaScript doesn't leak
- 🛡️ **Command Safety** — Dangerous shell commands blocked by default
- 🔍 **Web Search** — Google search with DuckDuckGo fallback
- 📁 **File Operations** — Read, write, list with full filesystem access
- 💻 **Shell Execution** — Run commands with safety guardrails
- 🎨 **TUI Interface** — Beautiful terminal UI with animated avatar

## Quick Start

```bash
# Clone
git clone https://github.com/HeyBatlle1/Argus1.git
cd Argus1

# Build
cargo build --release

# Initialize (creates encrypted vault, stores master key in keychain)
./target/release/argus init

# Store your OpenRouter API key
echo "your-api-key" | ./target/release/argus vault set OPENROUTER_KEY

# Run the agent
./target/release/argus run
```

## Security Model

| Threat | Mitigation |
|--------|------------|
| Secrets in plaintext | ChaCha20-Poly1305 encrypted vault |
| Key in environment | Hardware keychain integration |
| Memory corruption | Rust memory safety |
| Command injection | Blocklist + shell escaping |
| Supply chain attacks | Minimal dependencies, audited crates |

## Architecture

```
argus/
├── crates/
│   ├── argus-cli/      # TUI + CLI interface
│   ├── argus-core/     # Agent orchestration (WIP)
│   ├── argus-crypto/   # Vault + keychain + encryption
│   ├── argus-memory/   # Persistent memory (WIP)
│   └── argus-sandbox/  # WASM isolation (WIP)
```

## Tools

Argus currently supports:

| Tool | Description |
|------|-------------|
| `read_file` | Read file contents |
| `write_file` | Create or modify files |
| `list_directory` | List folder contents |
| `shell` | Execute shell commands (with safety) |
| `web_search` | Search Google for current information |

## Roadmap

- [x] Encrypted vault with keychain
- [x] Tool calling (file, shell, search)
- [x] TUI with animated avatar
- [ ] Persistent conversation memory
- [ ] MCP protocol support
- [ ] WASM sandbox for untrusted tools
- [ ] Post-quantum cryptography (ML-KEM, ML-DSA)
- [ ] Multi-model support

## Philosophy

Read [SOUL.md](./SOUL.md) to understand what Argus is and why it exists.

## License

MIT — Use it, modify it, make it better.

---

*"Nothing escapes the hundred eyes."*
