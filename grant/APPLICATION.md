# Agentic Engineering Grant Application — Superteam

**Project title:** Vani Protocol
**One-line description:** A keyless, Rust-native voice agent for Solana — Hindi, Telugu & Tamil speakers send, swap, and check balances by voice, with wallet keys locked in a TEE.
**TG username:** @enjoys
**Wallet address:** *(to be filled — the Earn auto-fill button, or your own Solana address; not the Turnkey test wallet)*

---

## 1. What Vani is

Vani is a **vernacular, voice-first AI agent SDK for Solana**. The problem: most of India's ~700M smartphone users aren't comfortable with English-first DeFi, and every existing Solana agent kit (solana-agent-kit, AgentKit) is TypeScript/Python-first, **holds private keys in the app**, and is unusable over voice. Vani flips all three:

- **Rust-native MCP server** on the official `rmcp` SDK — a single binary, no Python/TS runtime, 9 tools.
- **Keyless by design** — wallet private keys live inside Turnkey's Trusted Execution Environment; Vani builds transactions locally, hands the bytes to the TEE to sign, and broadcasts over its own RPC. A private key **never** exists in Vani (`SECURITY.md`).
- **Voice-first** — Sarvam AI speech-to-text / text-to-speech with a rule-based intent parser covering **Hindi, Hinglish, Telugu, and Tamil** (27 language tests). "मुझे अपना बैलेंस दिखाओ" → your balance. "1 SOL se USDC swap karo" → an on-chain swap.

What's live today: SOL & SPL-token sends (USDC/USDT/BONK/JUP), **on-chain Jupiter swaps** (v0 transactions with address lookup tables), prices, quotes, balances — all signed in the TEE, broadcast via our own RPC.

## 2. Why this is an *agentic engineering* project

Vani was **built end-to-end by an AI pair (Claude Code)** — not "a product with an AI feature." That's the point of the project, and it's the point of this grant:

- **A brain-log methodology.** The build runs across many AI sessions; each session dies with its context. To survive that, every decision, bug, and milestone is written to an append-only markdown brain-log. When a full session was lost, the project was recovered **in minutes** from one file — not restarted from zero.
- **An ADR decision log.** Architectural calls are logged and dated (e.g. dropping an unmaintainable MPC wrapper for direct Turnkey TEE integration after one day of research).
- **Test-driven, lint-clean discipline.** 48 tests green, clippy `-D warnings`, release build clean, **no dead code** — enforced by the agent, auditable in the repo.
- **Proof over prose.** Every claim is backed by a commit or an on-chain signature, not a slide.

## 3. Project plan

Four weeks, zero budget, free-tier only:

- **Week 1 — Agent layer.** Rust MCP server (rmcp), 5 read-only tools (balances, Jupiter prices/quotes, vernacular parser). ✅
- **Week 2 — Voice layer.** Sarvam STT/TTS round-trip; 27 Hindi/Telugu/Tamil tests. ✅
- **Week 3 — Security & execution.** Turnkey TEE signing (ECDSA P-256 stamped requests); live devnet broadcast landed; SPL sends; Jupiter swap execution built. ✅
- **Week 4 — Launch.** 5 beta users, 15+ real voice executions, feedback ≥ 4.2/5, Loom demo, submit this grant. 🔄

## 4. Milestones

| # | Milestone | Status |
|---|-----------|--------|
| M0 | Rust MCP server, 5 read-only tools, parser — 32 tests, clippy clean | ✅ Week 1 |
| M1 | Voice layer — STT/TTS round-trip, 27 Hindi/Telugu/Tamil tests | ✅ Week 2 |
| M2 | Keyless execution — Turnkey TEE signing, **live devnet broadcast**, SPL sends, Jupiter swaps | ✅ Week 3 |
| M3 | 5 beta users, 15+ real voice executions, feedback ≥ 4.2/5, demo | 🔄 Week 4 |

## 5. Strongest proof I can ship fast

1. **Working, committed code:** [github.com/eniyos/vani-protocol](https://github.com/eniyos/vani-protocol) — 12 commits, **48 tests green**, clippy clean, release build clean.
2. **Real on-chain proof:** a genuine devnet transaction was signed inside Turnkey's TEE and broadcast over Vani's own RPC (signature `2pJ5bNh6…Af4jK`). The keyless claim is demonstrated, not diagrammed.
3. **Shipped on schedule every week:** the 4-week plan from blank repo to live broadcast is already executed ahead of schedule.
4. **Zero-budget discipline (ADR-004):** Sarvam ₹100 credits, Turnkey Starter, public devnet — the grant goes to shipping users, not infrastructure.

## 6. How this grant upscales agentic engineering

At $0/month the agent build has hit its ceiling: Sarvam's free-tier rate limits (60 req/min STT), no mainnet RPC, and single-agent workflows. The grant pays for the exact upgrades that make the *agentic pipeline* production-grade:

- **Paid Sarvam voice credits** → real 15+ user beta on Telegram/Twitter (M3), not synthetic tests.
- **Helius mainnet RPC** → the keyless execution layer (sends + swaps) proven on real mainnet instead of devnet.
- **Multi-agent workflows** → scale the brain-log methodology from one Claude session into parallel build/review agents, which is how a solo founder ships an SDK in 4 weeks.

Vani is already the strongest possible demo of this grant's thesis: an agent built a working, keyless Solana product in weeks — and this grant lets that agent go from demo to shipped users.
