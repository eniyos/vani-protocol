//! Sarvam AI voice helpers (speech-to-text + text-to-speech) — the Week-2
//! voice layer. The API key comes from the environment (`SARVAM_API_KEY`) and
//! is used only as a per-request header; it is never stored or logged.

use anyhow::{bail, Context, Result};
use base64::Engine;
use reqwest::Client;
use serde_json::{json, Value};

const SARVAM_BASE: &str = "https://api.sarvam.ai";
const STT_MODEL: &str = "saaras:v3";
const TTS_MODEL: &str = "bulbul:v3";

/// Transcribe base64-encoded audio (WAV/MP3/OGG, ≤30 s) to text.
/// `language_code` is BCP-47 (`hi-IN`, `te-IN`, …) or `unknown` for auto-detect.
pub async fn speech_to_text(
    client: &Client,
    api_key: &str,
    audio_base64: &str,
    language_code: &str,
) -> Result<String> {
    let audio = base64::engine::general_purpose::STANDARD
        .decode(audio_base64)
        .context("audio_base64 is not valid base64")?;
    if audio.is_empty() {
        bail!("audio_base64 is empty");
    }

    let form = reqwest::multipart::Form::new()
        .text("model", STT_MODEL)
        .text("mode", "transcribe")
        .text("language_code", language_code.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(audio)
                .file_name("audio.wav")
                .mime_str("audio/wav")?,
        );

    let resp = client
        .post(format!("{SARVAM_BASE}/speech-to-text"))
        .header("api-subscription-key", api_key)
        .multipart(form)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        bail!("Sarvam STT {status}: {body}");
    }
    let resp: Value = serde_json::from_str(&body)
        .with_context(|| format!("Sarvam STT returned non-JSON: {body}"))?;

    let transcript = resp.get("transcript").and_then(Value::as_str).unwrap_or("");
    if transcript.is_empty() {
        bail!("Sarvam STT returned no transcript");
    }
    let detected = resp.get("language_code").and_then(Value::as_str).unwrap_or("?");
    Ok(format!("[{detected}] {transcript}"))
}

/// Synthesize speech from text (≤2,500 chars) using `bulbul:v3`.
/// Returns base64-encoded WAV audio. Voice defaults to `shubh`.
pub async fn text_to_speech(
    client: &Client,
    api_key: &str,
    text: &str,
    language_code: &str,
    speaker: &str,
) -> Result<String> {
    let resp: Value = client
        .post(format!("{SARVAM_BASE}/text-to-speech"))
        .header("api-subscription-key", api_key)
        .json(&json!({
            "text": text,
            "language_code": language_code,
            "speaker": speaker,
            "model": TTS_MODEL,
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let audios = resp
        .get("audios")
        .and_then(Value::as_array)
        .context("Sarvam TTS response missing audios")?;
    let joined: String = audios.iter().filter_map(|a| a.as_str()).collect();
    if joined.is_empty() {
        bail!("Sarvam TTS returned no audio");
    }
    Ok(joined)
}
