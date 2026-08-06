//! The MCP server handler. `#[tool_router(server_handler)]` turns this struct
//! into a standalone rmcp server exposing the `#[tool]` methods over stdio.

use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;

use crate::config::Config;
use crate::jupiter;
use crate::rpc::SolanaRpc;
use crate::sarvam;
use crate::vanicommand;

/// System-program public key (32 bytes, base58) — a valid pubkey whose balance
/// is always 0; used only as a safe default when no address is supplied.
const SYSTEM_PROGRAM_ADDRESS: &str = "11111111111111111111111111111111";

#[derive(Clone)]
pub struct VaniServer {
    rpc: SolanaRpc,
    http: reqwest::Client,
    default_address: Option<String>,
    /// Sarvam API key for the voice tools (`tts_speak`, `stt_transcribe`).
    /// Held only in memory; used per-request, never logged.
    sarvam_key: Option<String>,
}

impl VaniServer {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Config::from_env()?;
        let rpc = SolanaRpc::new(config.rpc_url)?;
        Ok(Self {
            rpc,
            http: reqwest::Client::new(),
            default_address: config.default_address,
            sarvam_key: config.sarvam_api_key,
        })
    }

    fn resolve_address(&self, given: &str) -> String {
        let given = given.trim();
        if given.is_empty() {
            self.default_address
                .clone()
                .unwrap_or_else(|| SYSTEM_PROGRAM_ADDRESS.to_string())
        } else {
            given.to_string()
        }
    }
}

// ---- tool parameter schemas (derived into MCP JSON Schema by rmcp) ----

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BalanceParams {
    /// Solana address. Leave empty to use the configured default address.
    #[serde(default)]
    pub address: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TokenBalanceParams {
    /// Solana address that owns the token account.
    #[serde(default)]
    pub address: String,
    /// The SPL token mint address.
    pub mint: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PriceParams {
    /// Token symbol or comma-separated list (SOL, USDC, USDT, BONK, JUP).
    pub symbol: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct QuoteParams {
    /// Input token mint address.
    pub input_mint: String,
    /// Output token mint address.
    pub output_mint: String,
    /// Amount in the input token's smallest unit (lamports for SOL).
    pub amount: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VaniCommandParams {
    /// The command text in Hindi, Hinglish, Telugu, or English.
    pub text: String,
}

fn default_language() -> String {
    "hi-IN".to_string()
}

fn default_speaker() -> String {
    "shubh".to_string()
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TtsParams {
    /// Text to speak, in the target language (≤2,500 chars).
    pub text: String,
    /// BCP-47 language code (default hi-IN).
    #[serde(default = "default_language")]
    pub language: String,
    /// Sarvam voice name (default shubh). Female: roopa, priya, …
    #[serde(default = "default_speaker")]
    pub speaker: String,
}

fn default_unknown_language() -> String {
    "unknown".to_string()
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SttParams {
    /// Base64-encoded audio (WAV/MP3/OGG, ≤30 s, 16 kHz mono preferred).
    pub audio_base64: String,
    /// Expected BCP-47 language code, or "unknown" for auto-detect.
    #[serde(default = "default_unknown_language")]
    pub language: String,
}

// ---- the tools ----

#[tool_router(server_handler)]
impl VaniServer {
    #[tool(description = "Get the SOL balance of a Solana address (lamports -> SOL). Leave address empty to use the configured default.")]
    async fn get_balance(&self, Parameters(BalanceParams { address }): Parameters<BalanceParams>) -> String {
        let addr = self.resolve_address(&address);
        match self.rpc.sol_balance(&addr).await {
            Ok(lamports) => format!("{addr}: {:.6} SOL ({lamports} lamports)", lamports as f64 / 1e9),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get the SPL token balance (smallest units) held by an address for a token mint.")]
    async fn get_token_balance(&self, Parameters(TokenBalanceParams { address, mint }): Parameters<TokenBalanceParams>) -> String {
        let addr = self.resolve_address(&address);
        match self.rpc.token_balance(&addr, &mint).await {
            Ok(units) => format!("{addr}: {units} smallest units of {mint}"),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a live token price from the Jupiter price API (SOL, USDC, USDT, BONK, JUP; comma-separate multiple symbols).")]
    async fn get_price(&self, Parameters(PriceParams { symbol }): Parameters<PriceParams>) -> String {
        match jupiter::price(&self.http, &symbol).await {
            Ok(out) => out,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a Jupiter swap quote (read-only, nothing is executed). Amount is in the input token's smallest unit.")]
    async fn jupiter_quote(&self, Parameters(QuoteParams { input_mint, output_mint, amount }): Parameters<QuoteParams>) -> String {
        match jupiter::quote(&self.http, &input_mint, &output_mint, amount).await {
            Ok(out) => out,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Parse a Hindi/Hinglish/Telugu/English command into a structured intent (MVP rule-based parser).")]
    fn vani_command(&self, Parameters(VaniCommandParams { text }): Parameters<VaniCommandParams>) -> String {
        let intent = vanicommand::parse(&text);
        serde_json::to_string_pretty(&intent).unwrap_or_else(|_| format!("{intent:?}"))
    }

    #[tool(description = "Synthesize speech from text via Sarvam AI TTS (bulbul:v3). Returns base64-encoded WAV audio.")]
    async fn tts_speak(&self, Parameters(TtsParams { text, language, speaker }): Parameters<TtsParams>) -> String {
        let Some(key) = self.sarvam_key.as_deref() else {
            return "Error: SARVAM_API_KEY not set in environment".to_string();
        };
        match sarvam::text_to_speech(&self.http, key, &text, &language, &speaker).await {
            Ok(audio) => audio,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Transcribe base64-encoded audio to text via Sarvam AI STT (saaras:v3). Returns '[detected-language] transcript'.")]
    async fn stt_transcribe(&self, Parameters(SttParams { audio_base64, language }): Parameters<SttParams>) -> String {
        let Some(key) = self.sarvam_key.as_deref() else {
            return "Error: SARVAM_API_KEY not set in environment".to_string();
        };
        match sarvam::speech_to_text(&self.http, key, &audio_base64, &language).await {
            Ok(out) => out,
            Err(e) => format!("Error: {e}"),
        }
    }
}