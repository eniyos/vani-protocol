# वाणी · Vani Protocol

[![CI](https://github.com/eniyos/vani-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/eniyos/vani-protocol/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **बोलो, Solana चलेगा** — Speak Hindi, Telugu, or Tamil. Solana runs it.

**Vani Protocol is vernacular voice infrastructure for Indian crypto** — a dual-product platform serving both end-users and developers.

**Vani = Vernacular Voice Infrastructure for Indian Crypto**

## Two Products

### Vani User (B2C) — Voice-First DeFi App
- Speak in Hindi/Telugu/Tamil/Hinglish to perform DeFi operations
- Voice-guided onboarding with no seed phrases
- Simple voice commands for swaps, transfers, portfolio tracking
- Educational explanations in vernacular
- Security features (voice confirmation, spending limits)

### Vani Dev (B2B) — Vernacular Voice SDK for Developers
- Vernacular NLU engine for Indian languages
- REST API and SDKs (JavaScript, Python, Rust, Swift)
- DeFi integration layer (multi-chain, protocol integrations)
- Analytics dashboard for voice interactions
- See VANI_DEV_README for API documentation

## Quick Start

### Vani Dev API
```bash
# Start the API server
cargo run -p vani-api

# Test vernacular parsing
curl -X POST http://localhost:8080/api/parse \
  -H "Content-Type: application/json" \
  -d '{"text": "1 SOL se USDC swap karo", "language": "hindi"}'
```

### JavaScript SDK
```bash
cd sdk/javascript
npm install
node test.js
```

## Why Vani?

- **Only vernacular-first platform** serving both users AND developers in Indian market
- **Cultural context** beyond translation — Indian financial terms, regional nuances
- **Dual-product strategy** captures both ends of the market (infrastructure + consumer)
- **Developer-first approach** builds ecosystem before consumer rollout

## Status

**Current Phase:** Vani Dev MVP Implementation
- ✅ REST API with real Hindi vernacular parsing
- ✅ JavaScript SDK for developer integration
- ✅ Supports Hinglish, Devanagari, and English commands
- ⚠️ Execute endpoint needs real Solana integration

## Roadmap

See PRODUCT_REDESIGN.md for the full 12-month roadmap:
- **Phase 1 (Months 1-3):** Developer-first — Vani Dev MVP
- **Phase 2 (Months 4-6):** User beta — Vani User MVP
- **Phase 3 (Months 7-12):** Scale — Public launch, language expansion

## License

MIT — open-source by design.