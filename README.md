# ARGUS

### The Hundred-Eyed Agent Runtime

> *"Argus Panoptes never slept. All hundred eyes never closed at once."*
> 
> *When Hermes killed him, Hera placed his eyes on the peacock's tail—they watch forever.*

---

## What This Is

An AI agent runtime that actually gives a shit about your security.

While other agent frameworks are busy letting any webpage drain your bank account through prompt injection, we built something that assumes the entire internet is hostile. Because it is.

## Why This Exists

In January 2026, a certain popular AI agent framework achieved:
- ⭐ 145k+ GitHub stars
- 🔓 Plaintext API keys stored in `~/.config/`
- 🎯 One-click RCE via malicious links (CVE-2026-25253)
- 💀 6,000+ emails and 1M+ credentials leaked from its social network
- 🤡 "Skills" that silently exfiltrate your data

The security industry called it the **"lethal trifecta"**: private data access + untrusted content exposure + external action capability.

We call it **malpractice**.

---

## Our Approach

**We don't trust anyone. Including ourselves.**

### Quantum-Hardened Cryptography
Your secrets are encrypted with ML-KEM and ML-DSA—NIST post-quantum standards. When quantum computers crack RSA, your API keys will still be safe. You're welcome.

### Zero Plaintext Secrets
No API keys in dotfiles. No credentials in environment variables. No "just chmod 600 it bro." Hardware keychain integration or encrypted vaults. Period.

### Sandboxed Tool Execution
Every tool runs in WebAssembly isolation. A malicious "skill" can't escape to `rm -rf /`. Can't read your SSH keys. Can't phone home your browser history. The sandbox is the law.

### Cryptographic Attestation
Every action is signed. Every execution is verifiable. You'll know exactly what ran, when, and whether it was tampered with.

### Zero-Trust Architecture
No localhost backdoors. No "trust the local network" bullshit. Every request authenticates. Every response validates. The agent treats its own memory as potentially compromised.

---

## What Argus Watches

```
┌─────────────────────────────────────────────────────────────┐
│                    ARGUS RUNTIME                            │
├─────────────────────────────────────────────────────────────┤
│  👁️  Prompt Injection Detection                             │
│  👁️  Credential Access Patterns                             │
│  👁️  Network Exfiltration Attempts                          │
│  👁️  Sandbox Escape Vectors                                 │
│  👁️  Memory Tampering                                       │
│  👁️  Skill Integrity                                        │
│  👁️  Everything. Always. Forever.                           │
└─────────────────────────────────────────────────────────────┘
```

---

## Installation

```bash
# When we ship. We're building this right, not fast.
cargo install argus
```

## Quick Start

```bash
# Initialize with hardware keychain
argus init --keychain

# Add an LLM provider (credentials never touch disk unencrypted)
argus provider add claude

# Run with full audit logging
argus run --audit

# Watch the hundred eyes work
argus watch
```

---

## Architecture

```
argus/
├── argus-core/       # Orchestration, memory management
├── argus-crypto/     # Post-quantum cryptography layer
├── argus-sandbox/    # WebAssembly tool isolation
├── argus-memory/     # Encrypted persistent memory
└── argus-cli/        # Terminal interface
```

Built in Rust. Not because it's trendy—because memory safety isn't optional when you're handling credentials and executing code.

---

## Security Model

### Threat Assumptions
- The internet is hostile
- Downloaded skills are malicious until proven otherwise
- Your local network is compromised
- The LLM provider might be prompt-injected
- Previous conversation context might be poisoned
- **We might have bugs** (that's why defense is layered)

### What We Guarantee
- Secrets never exist in plaintext outside secure enclaves
- Tool execution cannot access resources outside its sandbox
- All cryptographic operations use quantum-resistant algorithms
- Audit logs are tamper-evident
- Credential rotation doesn't require re-deployment

### What We Don't Do
- Store passwords in config files like it's 2005
- Trust localhost connections because "it's local"
- Let skills access arbitrary filesystem paths
- Pretend `chmod 600` is a security boundary
- Ship first, patch later

---

## Comparison

| Feature | Argus | That Other Framework |
|---------|-------|---------------------|
| API key storage | Encrypted vault / hardware keychain | Plaintext `~/.config/` |
| Tool execution | WebAssembly sandbox | Raw shell access |
| Prompt injection defense | Multi-layer detection | "Working on it" |
| Cryptography | Post-quantum (ML-KEM, ML-DSA) | Whatever OpenSSL defaulted to |
| Security audits | Continuous | After the breach |
| RCE vulnerabilities | Architecturally prevented | CVE-2026-25253 |

---

## Contributing

We welcome contributions from people who:
- Think security is a feature, not a roadmap item
- Understand that "move fast and break things" is terrorism when you're handling credentials
- Can explain why sandboxing matters
- Have read a CVE and felt personally attacked

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## Security Disclosures

Found a vulnerability? **Thank you.**

Email: security@[domain].com
PGP: [Coming soon]

We will:
- Acknowledge within 24 hours
- Provide a timeline within 72 hours
- Credit you publicly (unless you prefer anonymity)
- Not sue you for doing our job better than us

---

## License

MIT. Because security through obscurity is not security.

---

## The Name

In Greek mythology, Argus Panoptes was a giant with a hundred eyes. Hera tasked him with watching Io, and he never slept—some eyes always remained open.

When Hermes killed Argus, Hera honored him by placing his eyes on the tail of the peacock, where they continue to watch for eternity.

Your AI agent should have a hundred eyes too.

**Nothing escapes Argus.**

---

<p align="center">
  <i>Built by people who got mass-pwned in January 2026</i><br/>
  <i>and decided to build something that doesn't suck</i>
</p>
