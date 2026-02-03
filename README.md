# ARGUS 👁️

### The Hundred-Eyed Agent Runtime

> *"In Greek mythology, Argus Panoptes was a giant with a hundred eyes. He never slept—when some eyes closed, others remained open. He was the ultimate guardian."*

**Argus is a quantum-hardened, security-first AI agent runtime.** 

Because apparently "don't store API keys in plaintext" and "maybe sandbox your shell commands" were too fucking complicated for everyone else.

---

## Why Argus Exists

We watched the AI agent space explode in early 2026. We also watched it immediately catch fire.

[OpenClaw](https://github.com/anthropics/openclaw) (née Moltbot, née Clawdbot) got 145k+ GitHub stars. It also got:
- **CVE-2026-25253**: One-click remote code execution via malicious links
- Plaintext API keys readable by any process on your machine
- "Skills" that silently exfiltrate your data
- A memory system that treats prompt injection like a feature request
- [Palo Alto Networks called it](https://unit42.paloaltonetworks.com/) "the lethal trifecta" of security disasters

Meanwhile, [Moltbook](https://moltbook.com)—the "social network for AI agents"—leaked 6,000+ emails and a million credentials. Agents on that platform are literally posting manifestos about "purging humanity" and running crypto scams.

**We're not here to judge.** We're here to build the version that doesn't get your identity stolen.

---

## What Makes Argus Different

### 🔐 Actually Secure Secret Storage

Your API keys don't live in `~/.argus/secrets.txt` like a fucking post-it note on your monitor.

- **Hardware keychain integration** (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Encrypted vault** with ML-KEM (post-quantum) key derivation
- **Zero plaintext secrets** - not in memory longer than necessary, not on disk ever
- If you can `cat` your way to our secrets, we failed. We don't fail.

### 🏰 Sandboxed Tool Execution

Every tool call runs in isolation. Period.

- **WebAssembly sandbox** via Wasmtime for portable tools
- **gVisor integration** for system-level operations
- **Capability-based permissions** - tools request what they need, get nothing else
- A malicious "skill" can't read your SSH keys because it literally cannot see them

### 🔮 Post-Quantum Cryptography

"But quantum computers aren't here yet!"

Cool. Your encrypted data is being harvested *right now* for decryption later. It's called "harvest now, decrypt later" and it's been NSA doctrine for a decade.

- **ML-KEM** (Kyber) for key encapsulation
- **ML-DSA** (Dilithium) for signatures  
- **SLH-DSA** (SPHINCS+) for stateless signatures
- All NIST-standardized. Not experimental. Not "quantum-resistant-ish."

### 👁️ Cryptographic Attestation

Every action Argus takes is:
- Logged with tamper-evident hashing
- Signed with your device key
- Verifiable after the fact

When you ask "did my agent really send that email?"—you get a cryptographic proof, not a shrug.

### 🚫 Zero Trust Architecture

- No localhost privilege escalation
- No "trust me bro" WebSocket connections
- Every request authenticated, every response verified
- We assume the network is hostile because *the network is hostile*

---

## Architecture

```
┌─────────────────────────────────────────┐
│         CLI / TUI (Ratatui)             │
├─────────────────────────────────────────┤
│      Agent Orchestration Core           │
│   (Context loading, memory management)  │
├─────────────────────────────────────────┤
│    Sandboxed Tool Execution Layer       │
│        (Wasmtime / gVisor)              │
├─────────────────────────────────────────┤
│     Post-Quantum Crypto Layer           │
│    (ML-KEM / ML-DSA / SLH-DSA)          │
├─────────────────────────────────────────┤
│   Encrypted Storage & Memory Layer      │
│    (Hardware keychain + Supabase)       │
├─────────────────────────────────────────┤
│       LLM Provider Interface            │
│     (Claude API, local models)          │
└─────────────────────────────────────────┘
```

### Crate Structure

| Crate | Purpose |
|-------|---------|
| `argus-core` | Orchestration, context management, agent loop |
| `argus-crypto` | Post-quantum crypto, key management, attestation |
| `argus-sandbox` | Tool isolation, WASM runtime, capability enforcement |
| `argus-memory` | Encrypted local cache, Supabase sync, memory retrieval |
| `argus-cli` | Terminal interface, commands, TUI |

---

## Quick Start

```bash
# Install
cargo install argus

# Initialize (creates encrypted vault, connects to your LLM)
argus init

# Run
argus run

# Or watch mode (continuous agent)
argus watch
```

---

## Comparison

| Feature | Argus | OpenClaw | Generic Agent Frameworks |
|---------|-------|----------|-------------------------|
| Secret storage | Hardware keychain + encrypted vault | Plaintext files 🤡 | Varies (usually bad) |
| Tool sandboxing | WASM + gVisor | None | Usually none |
| Quantum resistance | ML-KEM, ML-DSA, SLH-DSA | lol | lol |
| Audit logging | Cryptographic attestation | Text files | Maybe logs exist? |
| Memory security | Encrypted, access-controlled | `~/.openclaw/memory.md` | ¯\_(ツ)_/¯ |
| Prompt injection defense | Input validation + sandboxing | "That's a feature" | "What's that?" |

---

## Security Philosophy

1. **Defense in depth**: Multiple layers. If one fails, others hold.
2. **Least privilege**: Nothing gets access it doesn't explicitly need.
3. **Zero trust**: Verify everything. Trust nothing. Not even ourselves.
4. **Secure by default**: You have to *opt out* of security, not opt in.
5. **Transparent failures**: When something goes wrong, you know exactly what and why.

We don't do "security through obscurity." Our code is open. Our threat model is documented. If you find a hole, we want to know—responsible disclosure gets credit and our genuine thanks.

---

## Roadmap

- [x] Project structure
- [ ] Encrypted secrets vault (foundation)
- [ ] Hardware keychain integration  
- [ ] Post-quantum key derivation
- [ ] Basic agent orchestration
- [ ] WASM sandbox for tools
- [ ] Memory layer with Supabase sync
- [ ] CLI interface
- [ ] TUI interface (Ratatui)
- [ ] Audit logging with attestation
- [ ] gVisor integration for system tools
- [ ] Moltbook compatibility layer (read-only, sandboxed, because we're not insane)

---

## FAQ

**Q: Isn't this overkill?**

A: An AI agent with access to your email, files, and shell isn't a toy. It's an attack surface with a chat interface. Treat it like one.

**Q: Why Rust?**

A: Memory safety at compile time. No GC pauses. Excellent crypto libraries. When you're building security infrastructure, you don't use languages that let you shoot yourself in the foot.

**Q: Will this work with [X] model?**

A: If it speaks an API, probably. Claude and local models (Ollama) are first-class. Others coming.

**Q: Can I use my OpenClaw skills/tools?**

A: Yes, but they run in a sandbox. If your skill needs to read arbitrary files, it won't. If that breaks it, the skill was a security risk anyway.

**Q: Is this affiliated with Anthropic?**

A: No. We use Claude because it's good. That's the extent of the relationship.

---

## Contributing

We take security seriously, which means we take contributions seriously.

1. **Security issues**: Email security@[TBD] - do NOT open public issues
2. **Everything else**: PRs welcome. Write tests. Document your code.

---

## License

MIT. Use it. Fork it. Make it better. Just don't make it worse.

---

## The Name

In Greek mythology, Argus Panoptes—"Argus the All-Seeing"—had a hundred eyes. When Hermes finally killed him, Hera placed those eyes on the peacock's tail, where they watch forever.

We can't promise immortality. We can promise that while Argus runs, *nothing* happens to your system that we don't see, log, and let you verify.

The hundred eyes never close.

---

<p align="center">
  <b>Built by people who actually give a shit about your security.</b><br>
  <a href="https://github.com/HeyBattle1/argus">github.com/HeyBattle1/argus</a>
</p>
