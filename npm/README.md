See the [main README](https://github.com/eniyos/vani-protocol#readme) for full documentation.

## Quick install

```sh
npx vani-mcp           # no install — runs the pre-built binary
npm install -g vani-mcp # global install
cargo install vani-mcp  # if you have Rust
```

## Wire into Claude Desktop / Cursor / Kiro

```jsonc
{
  "mcpServers": {
    "vani": {
      "command": "npx",
      "args": ["vani-mcp"],
      "env": {
        "SARVAM_API_KEY": "sk_...",
        "TURNKEY_ORGANIZATION_ID": "...",
        "TURNKEY_API_PUBLIC_KEY":  "...",
        "TURNKEY_API_PRIVATE_KEY": "...",
        "TURNKEY_SOLANA_WALLET_ADDRESS": "..."
      }
    }
  }
}
```

Full docs → https://github.com/eniyos/vani-protocol
