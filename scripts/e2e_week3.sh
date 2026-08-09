#!/usr/bin/env bash
# End-to-end proof that Vani actually signs + broadcasts on Devnet via Turnkey.
#
# This is the step that turns the (already built) SOL/token execution layer from
# "unit-tested but never run" into "proven on a real chain". It flags the same
# capture as the tools themselves: NO private key ever touches this repo's box —
# only TURNKEY API creds in .env, and the wallet's *address* (used only to fund
# it via public devnet faucet).
#
# Usage:
#   scripts/e2e_week3.sh                 # prints the setup runbook if .env is bare
#
# Requires: a real devnet wallet + creds in .env, and the `solana` CLI
# (for the faucet airdrop only). The MCP server is driven over stdio by a tiny
# python driver built into this script.
set -euo pipefail

cd "$(dirname "$0")/.."

# Load .env (dotenvy also does this at boot, but we need the values here).
if [ -f .env ]; then
  set -a; . ./.env; set +a
else
  echo "No .env — copy .env.example → .env and fill in the Turnkey values first."
  exit 1
fi

missing=()
[ -n "${TURNKEY_ORGANIZATION_ID:-}" ]  || missing+=("TURNKEY_ORGANIZATION_ID")
[ -n "${TURNKEY_API_PUBLIC_KEY:-}" ]   || missing+=("TURNKEY_API_PUBLIC_KEY")
[ -n "${TURNKEY_API_PRIVATE_KEY:-}" ]  || missing+=("TURNKEY_API_PRIVATE_KEY")
[ -n "${TURNKEY_SOLANA_WALLET_ADDRESS:-}" ] || missing+=("TURNKEY_SOLANA_WALLET_ADDRESS")

if [ "${#missing[@]}" -gt 0 ]; then
  echo "Turnkey execution isn't wired up yet. Weeks 3–3.5 code is built & unit-tested"
  echo "(45 tests green) but has never touched a real chain. To prove it, do this:"
  echo
  echo "  1) https://app.turnkey.com → sign up (free 'Starter'). That's your Parent Org;"
  echo "     copy the Organization ID from the top-left of the dashboard."
  echo "  2) 'API Keys' → Create API Key → download the pair. Two files drop:"
  echo "     api_public_key.txt (66 hex chars, starts 02/03) and"
  echo "     api_private_key.txt (64 hex chars, raw P-256 scalar)."
  echo "  3) 'Wallets' → Create Wallet → Solana → copy the derived address."
  echo "  4) In .env set: TURNKEY_ORGANIZATION_ID, TURNKEY_API_PUBLIC_KEY (from the .txt),"
  echo "     TURNKEY_API_PRIVATE_KEY (from the .txt), TURNKEY_SOLANA_WALLET_ADDRESS,"
  echo "     VANI_EXECUTE_NETWORK=solana:devnet. Keep .env out of git (it is)."
  echo "  5) Re-run this script — it funds the wallet and broadcasts a real send."
  echo
  echo "Missing: ${missing[*]}"
  exit 0
fi

# Format preflight — catches copy-paste mistakes before any network call.
if ! [[ "$TURNKEY_API_PRIVATE_KEY" =~ ^[0-9a-fA-F]{64}$ ]]; then
  echo "TURNKEY_API_PRIVATE_KEY must be exactly 64 hex chars (32-byte raw P-256 scalar)."
  echo "It's the contents of api_private_key.txt — got ${#TURNKEY_API_PRIVATE_KEY} chars."
  exit 1
fi
if ! [[ "$TURNKEY_API_PUBLIC_KEY" =~ ^(02|03)[0-9a-fA-F]{64}$ ]]; then
  echo "TURNKEY_API_PUBLIC_KEY must be 66 hex chars starting 02 or 03 (compressed P-256)."
  echo "It's the contents of api_public_key.txt."
  exit 1
fi

WALLET="$TURNKEY_SOLANA_WALLET_ADDRESS"
TO="${VANI_E2E_TO:-11111111111111111111111111111111}"  # devnet sink; set VANI_E2E_TO for a real recipient
CLUSTER_URL="${VANI_E2E_URL:-https://api.devnet.solana.com}"
AMOUNT="${VANI_E2E_AMOUNT:-0.001}"

echo "== Ensuring Turnkey devnet wallet is funded =="
solana config set --url "$CLUSTER_URL" >/dev/null
HAVE=$(solana balance "$WALLET" 2>/dev/null | awk '{print $1}')
if awk -v b="$HAVE" 'BEGIN{exit !(b > 0.005)}'; then
  echo "wallet already funded: $HAVE SOL — skipping faucet"
else
  echo "funding $WALLET (public faucet; often dry/rate-limited)..."
  if solana airdrop 1 "$WALLET" 2>&1 | tail -1; then
    echo "faucet airdrop ok"
  else
    echo "faucet unavailable — falling back to the local devnet keypair"
    if solana transfer "$WALLET" 1 --url "$CLUSTER_URL" --allow-unfunded-recipient 2>&1 | tail -1; then
      echo "funded from local devnet keypair"
    else
      echo "could not fund $WALLET — fund it manually, then re-run."; exit 1
    fi
  fi
fi

echo
if [ ! -x target/release/vani-mcp ]; then
  echo "== Building release binary (first run) =="
  cargo build --release
fi

echo "== Booting Vani MCP server and issuing a real \`vani_execute\` =="
python3 - <<PYEOF
import json, subprocess
proc = subprocess.Popen(["target/release/vani-mcp"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
def send(m): proc.stdin.write((json.dumps(m)+"\n").encode()); proc.stdin.flush()
def read_until():
    while True:
        line = proc.stdout.readline()
        if not line: raise RuntimeError("server closed stdout")
        o = json.loads(line)
        if "id" in o and o["id"] is not None: return o
send({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"e2e","version":"0"}}})
read_until()
send({"jsonrpc":"2.0","method":"notifications/initialized","params":{}})
send({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"vani_execute","arguments":{
    "text": "$AMOUNT SOL bhej do", "to": "$TO"}}})
r = read_until()["result"]
out = (r.get("content") or [{}])[0].get("text","")
if r.get("isError"):
    print("vani_execute ERROR:", out)
    raise SystemExit(2)
print(out)
import re
m = re.search(r"signature ([A-Za-z0-9]+)", out)
if m:
    print("\\nExplorer: https://explorer.solana.com/tx/%s?cluster=devnet" % m.group(1))
proc.kill()
PYEOF
echo "== E2E broadcast completed =="