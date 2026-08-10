# Brain-Log — the agentic-session-survival methodology

Vani's build runs across many AI sessions; a session dies with its context. To make the project restart from **"done"** instead of **"zero"**, every decision, bug, milestone, and proof is appended to a dated brain-log. The full log lives in `Vani-Protocol-brain/log.md` (23 entries). Representative excerpts:

```
## [2026-08-09] update | ADR-009 committed + pushed (07d179a → origin/main)
- Codebase commit 07d179a ... includes: turnkey.rs (create_wallet shape fix + sign_transaction,
  removed dead sol_send_transaction), rpc.rs (send_transaction), execute.rs (sign→broadcast both
  paths), server.rs (tool desc), e2e_week3.sh (regex + funding fallback). 46 tests, clippy clean.

## [2026-08-09] proof | FIRST LIVE BROADCAST
- 0.001 SOL sent, sig 2pJ5bNh6…Af4jK on devnet. Balances confirm: sender 1.0→0.998995 SOL,
  recipient +0.001. Weeks 3–3.5 now *signed + broadcast on devnet*. This is the ADR-008 gate —
  swap build is now unlocked.

## [2026-08-08] decision | ADR-008 — defer swap behind a real broadcast
- Jupiter swap-instructions → v0-message + address-lookup-table assembly cannot be validated
  offline. Given this session already caught two bugs from unverifiable code (malformed Token
  program id; unrecognized-token→SOL fallthrough), defer the swap until the signing layer is
  proven by a real broadcast.
```

**The proof it works:** a full session was lost mid-project (context gone, decisions gone). The next session read the brain-log and was back to full speed in minutes — including the exact chain signatures and ADR state. That one-file recovery is why Vani ships fast: no session ever starts from zero.

Files in this package and what each proves:
| File | Proves |
|------|--------|
| `APPLICATION.md` | The submission itself |
| `PROOF.md` | Build/test/clippy green, git history, live broadcast |
| `ADR-LOG.md` | Deliberate, researched decision-making |
| `BRAIN-LOG.md` | The agentic methodology + real session-recovery incident |
| `README.md` | The product |
| `SECURITY.md` | The keyless security architecture |
