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
    // Clients may paste base64 with line breaks; the strict decoder rejects them.
    let cleaned: String = audio_base64.chars().filter(|c| !c.is_whitespace()).collect();
    let audio = base64::engine::general_purpose::STANDARD
        .decode(cleaned)
        .context("audio_base64 is not valid base64")?;
    if audio.is_empty() {
        bail!("audio_base64 is empty");
    }

    // Advertise the real container so Sarvam accepts the part (wrong mime -> 400).
    let (file_name, mime) = detect_audio_mime(&audio);
    let form = reqwest::multipart::Form::new()
        .text("model", STT_MODEL)
        .text("mode", "transcribe")
        .text("language_code", language_code.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(audio)
                .file_name(file_name)
                .mime_str(mime)?,
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

/// Sniff the container format from magic bytes so the multipart `file` part
/// advertises a mime Sarvam accepts. Falls back to WAV (the common case).
fn detect_audio_mime(audio: &[u8]) -> (&'static str, &'static str) {
    let wave = audio.len() >= 12 && audio.starts_with(b"RIFF") && &audio[8..12] == b"WAVE";
    let mp3 = audio.starts_with(b"ID3")
        || (audio.len() >= 2 && audio[0] == 0xFF && (audio[1] & 0xE0) == 0xE0);
    if wave {
        ("audio.wav", "audio/wav")
    } else if mp3 {
        ("audio.mp3", "audio/mpeg")
    } else if audio.starts_with(b"OggS") {
        ("audio.ogg", "audio/ogg")
    } else {
        ("audio.wav", "audio/wav")
    }
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
    if text.chars().count() > 2500 {
        bail!("text exceeds Sarvam's 2,500-char TTS limit");
    }
    let resp = client
        .post(format!("{SARVAM_BASE}/text-to-speech"))
        .header("api-subscription-key", api_key)
        .json(&json!({
            "text": text,
            "language_code": language_code,
            "speaker": speaker,
            "model": TTS_MODEL,
        }))
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        bail!("Sarvam TTS {status}: {body}");
    }
    let resp: Value = serde_json::from_str(&body)
        .with_context(|| format!("Sarvam TTS returned non-JSON: {body}"))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_wav_from_riff_header() {
        let mut b = b"RIFF1234WAVE".to_vec();
        b.extend_from_slice(&[0u8; 8]);
        assert_eq!(detect_audio_mime(&b), ("audio.wav", "audio/wav"));
    }

    #[test]
    fn detects_mp3_from_id3_tag() {
        let b = b"ID3\x04\x00\x00\x00\x00\x00\x00".to_vec();
        assert_eq!(detect_audio_mime(&b), ("audio.mp3", "audio/mpeg"));
    }

    #[test]
    fn detects_mp3_from_frame_sync() {
        let b = [0xFFu8, 0xFB, 0x90, 0x00].to_vec();
        assert_eq!(detect_audio_mime(&b), ("audio.mp3", "audio/mpeg"));
    }

    #[test]
    fn detects_ogg() {
        let b = b"OggS\x00\x02".to_vec();
        assert_eq!(detect_audio_mime(&b), ("audio.ogg", "audio/ogg"));
    }

    #[test]
    fn unknown_bytes_fall_back_to_wav() {
        assert_eq!(detect_audio_mime(b"<html>not audio"), ("audio.wav", "audio/wav"));
    }
}
