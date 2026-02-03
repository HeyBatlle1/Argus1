# Security Policy

## Reporting a Vulnerability

**Do not open public issues for security vulnerabilities.**

Email: security@[domain].com

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Your suggested fix (optional)

## Response Timeline

| Action | Timeline |
|--------|----------|
| Acknowledgment | 24 hours |
| Initial assessment | 72 hours |
| Fix development | Depends on severity |
| Public disclosure | After fix is released |

## What Counts as a Vulnerability

### In Scope
- Sandbox escape (WASM → host)
- Secret exposure (plaintext leaks)
- Cryptographic weaknesses
- Authentication bypass
- Prompt injection that bypasses defenses
- Memory safety issues
- Denial of service

### Out of Scope
- Social engineering
- Physical attacks
- Issues in dependencies (report upstream, but let us know)
- Theoretical attacks without PoC

## Severity Levels

### Critical
- Remote code execution
- Secret exfiltration
- Sandbox escape

### High
- Local privilege escalation
- Authentication bypass
- Cryptographic breaks

### Medium
- Information disclosure
- Denial of service
- Security control bypass

### Low
- Minor information leaks
- Hardening improvements

## Safe Harbor

We will not pursue legal action against security researchers who:
- Act in good faith
- Avoid privacy violations
- Avoid data destruction
- Report vulnerabilities promptly
- Don't publicly disclose before fix

## Recognition

With your permission, we'll credit you in:
- Release notes
- Security advisories
- README contributors section

---

*Security is why we exist. Thank you for making Argus stronger.*
