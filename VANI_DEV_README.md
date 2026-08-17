# Vani Dev — Vernacular Voice SDK for Developers

## Quick Start

### API Server

**Start the API:**
```bash
cargo run -p vani-api
```

**Test the API:**
```bash
# Health check
curl http://localhost:8080/health

# Parse a vernacular command
curl -X POST http://localhost:8080/api/parse \
  -H "Content-Type: application/json" \
  -d '{"text": "1 SOL se USDC swap karo", "language": "hindi"}'

# Execute a transaction
curl -X POST http://localhost:8080/api/execute \
  -H "Content-Type: application/json" \
  -d '{"intent": {"action":"swap","source":"SOL","target":"USDC","amount":1.0,"raw":"1 SOL se USDC swap karo","confidence":0.95}, "wallet_address": "your_wallet_address"}'
```

### JavaScript SDK

**Install:**
```bash
cd sdk/javascript
npm install
```

**Usage:**
```javascript
import VaniSDK from '@vani/sdk';

const vani = new VaniSDK({
  apiURL: 'http://localhost:8080'
});

// Parse a vernacular command
const intent = await vani.parseCommand('1 SOL se USDC swap karo', 'hindi');
console.log(intent);
// { action: 'swap', source: 'SOL', target: 'USDC', amount: 1.0, confidence: 0.95 }

// Execute the transaction
const result = await vani.executeTransaction(intent);
console.log(result);
// { success: true, transaction_id: 'tx_123', error: null }

// Or combine both steps
const result = await vani.parseAndExecute('2 USDC se SOL swap karo', 'hindi');
```

**Run tests:**
```bash
cd sdk/javascript
node test.js
```

## API Endpoints

### POST /api/parse
Parse vernacular text into structured intent.

**Request:**
```json
{
  "text": "1 SOL se USDC swap karo",
  "language": "hindi"
}
```

**Response:**
```json
{
  "action": "swap",
  "source": "SOL",
  "target": "USDC", 
  "amount": 1.0,
  "raw": "1 SOL se USDC swap karo",
  "confidence": 0.95
}
```

### POST /api/execute
Execute a parsed intent as a blockchain transaction.

**Request:**
```json
{
  "intent": {
    "action": "swap",
    "source": "SOL",
    "target": "USDC",
    "amount": 1.0,
    "raw": "1 SOL se USDC swap karo",
    "confidence": 0.95
  },
  "wallet_address": "optional_wallet_address"
}
```

**Response:**
```json
{
  "success": true,
  "transaction_id": "tx_123",
  "error": null
}
```

### GET /health
Health check endpoint.

**Response:**
```
Vani API is running
```

## Current Status

**✅ Working:**
- REST API server (Axum framework) running on http://localhost:8080
- Real vernacular parser integrated from vani-mcp (Hindi/Hinglish/Telugu/Tamil)
- Parse endpoint with real NLU processing (not mock)
- Execute endpoint with smart action validation
- JavaScript SDK with full API integration
- CORS enabled for cross-origin requests
- Confidence scoring based on parsing quality

**🔧 TODO:**
- Connect to real Solana execution (requires Turnkey credentials)
- Add authentication (API keys)
- Add error handling and validation
- Add rate limiting
- Add logging and analytics
- Create Python SDK
- Create Rust SDK
- Add TypeScript types
- Create comprehensive documentation

## Development

**Build API:**
```bash
cargo build -p vani-api
```

**Run API in development:**
```bash
cargo run -p vani-api
```

**Build for release:**
```bash
cargo build -p vani-api --release
```

## Architecture

```
Developer App
    ↓ HTTP/JavaScript SDK
Vani API (Axum)
    ↓ Internal calls
Vani MCP (existing Rust code)
    ↓ Blockchain
Solana / Turnkey
```

## Next Steps

1. **Integrate vernacular parser** from vani-mcp into API
2. **Connect real execution** to Solana via existing execute module
3. **Add authentication** for developer API keys
4. **Create documentation** with examples
5. **Build additional SDKs** (Python, Rust, Swift)
6. **Add analytics** for developer insights
7. **Create developer dashboard**