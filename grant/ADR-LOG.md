# Decision Log (ADRs) — evidence of deliberate, logged agentic engineering

Every architectural call is logged and dated, newest last. This is how the agent avoided shipping unverifiable code — several ADRs exist precisely *because* research disproved an assumption. Full version lives in the project brain vault (`Vani-Protocol-brain/wiki/decisions.md`).

## ADR-009 — Execute via `sign_transaction` (TEE sign) + self-broadcast (accepted, 2026-08-09)
Turnkey's hosted broadcast (`SolSendTransaction`) is gated behind an org feature flag we can't enable on a fresh Starter org (verified in console + `set_organization_feature` enum). **Decision:** use core `sign_transaction` (TEE signs, returns signed tx) and broadcast via our own RPC. Keys still never touch Vani. **Result:** unblocked the first live broadcast immediately.

## ADR-008 — Defer swap until a real broadcast proves the layer (accepted, 2026-08-08)
Jupiter `swap-instructions` → v0-message + lookup-table assembly cannot be validated offline, and two bugs were already caught from unverifiable code. **Decision:** gate the swap behind a real first broadcast. **Result:** the live broadcast (sig `2pJ5bNh6…Af4jK`) proved the signing layer, so the swap build then proceeded against live responses — and is now complete.

## ADR-007 — Integrate Turnkey directly; Knot Wallet dropped (accepted, 2026-08-07)
The planned "Knot Wallet MPC" was verified un-integrable: dormant, unlicensed, hosted API down, and itself a Turnkey wrapper. **Decision:** thin Rust client against Turnkey's REST API (P-256 API-key auth). **Result:** no dependency on Knot's downtime or license.

## ADR-006 — Build our own Rust MCP server; "solana-claude" dropped (accepted, 2026-08-06)
The spec assumed an open-source 23-tool MCP server that **does not exist** (verified on GitHub). Real kits are TS-first and hold keys. **Decision:** hand-roll on `rmcp`. **Result:** Rust-native claim is real and keyless from day one.

## ADR-005 — MIT open-source + Superteam India grant route (accepted, 2026-08-06)
**Decision:** public code under MIT, grant after MVP. Public code is a reputation asset and a grant requirement.

## ADR-004 — Free-tier-only until grant approval (accepted, 2026-08-06)
**Decision:** use only free tiers (Sarvam ₹100 credits, Turnkey Starter, public devnet). This grant's funds go straight to the paid upgrades that unlock the beta.

## ADR-003 — Vernacular-first (Hindi/Telugu over English) (accepted, 2026-08-06)
**Decision:** prioritize Hindi/Telugu; English secondary. Recognized as the riskiest correctness surface; validated with 27 language tests.

## ADR-002 — MPC signing; never manage private keys (accepted, 2026-08-06; supplier superseded by ADR-007)
**Decision:** all signing via MPC/TEE; never store/log/transmit private keys; policy controls mandatory. **Result:** the keyless security architecture documented in `SECURITY.md`.

## ADR-001 — Rust-native MCP on solana-claude (superseded by ADR-006)
Superseded by ADR-006 when the presumed underlying repo turned out not to exist.
