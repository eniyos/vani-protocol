#!/usr/bin/env node
/**
 * postinstall: download the correct vani-mcp binary from GitHub Releases.
 *
 * The binary is placed at bin/vani-mcp (or bin/vani-mcp.exe on Windows)
 * and made executable so `npx vani-mcp` works immediately.
 */

"use strict";

const fs = require("fs");
const https = require("https");
const path = require("path");
const { execSync } = require("child_process");
const zlib = require("zlib");

const PKG = require("../package.json");
const VERSION = PKG.version; // e.g. "0.1.0"
const REPO = "eniyos/vani-protocol";
// The real native binary lives in bin/native/ so it doesn't collide with the
// JS launcher wrapper (bin/vani-mcp) that npm puts on $PATH.
const IS_WIN = process.platform === "win32";
const BIN_NAME = IS_WIN ? "vani-mcp.exe" : "vani-mcp";
const BIN_DIR = path.join(__dirname, "..", "bin", "native");
const BIN_PATH = path.join(BIN_DIR, BIN_NAME);

// ── Platform → GitHub Release asset name ───────────────────────────────────
function getTarget() {
  const { platform, arch } = process;
  if (platform === "darwin") {
    return arch === "arm64"
      ? "aarch64-apple-darwin"
      : "x86_64-apple-darwin";
  }
  if (platform === "linux") {
    return arch === "arm64"
      ? "aarch64-unknown-linux-musl"
      : "x86_64-unknown-linux-musl";
  }
  if (platform === "win32") {
    return "x86_64-pc-windows-msvc";
  }
  throw new Error(`Unsupported platform: ${platform}/${arch}`);
}

function getAssetName(target) {
  const ext = IS_WIN ? "zip" : "tar.gz";
  return `vani-mcp-v${VERSION}-${target}.${ext}`;
}

// ── Tiny HTTP(S) fetch that follows redirects ────────────────────────────────
function fetch(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "vani-mcp-npm-installer" } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return fetch(res.headers.location).then(resolve).catch(reject);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
        }
        const chunks = [];
        res.on("data", (c) => chunks.push(c));
        res.on("end", () => resolve(Buffer.concat(chunks)));
        res.on("error", reject);
      })
      .on("error", reject);
  });
}

// ── tar.gz extractor (pure Node, no child_process) ───────────────────────────
function extractTarGz(buf, binName) {
  return new Promise((resolve, reject) => {
    const gunzip = zlib.createGunzip();
    const { Readable } = require("stream");
    const readable = Readable.from(buf);

    let pos = 0;
    const inflated = [];
    gunzip.on("data", (c) => inflated.push(c));
    gunzip.on("end", () => {
      const tar = Buffer.concat(inflated);
      // Parse POSIX ustar blocks (512-byte headers + content)
      let offset = 0;
      while (offset + 512 <= tar.length) {
        const header = tar.slice(offset, offset + 512);
        const name = header.slice(0, 100).toString("utf8").replace(/\0/g, "").trim();
        const sizeOctal = header.slice(124, 136).toString("utf8").replace(/\0/g, "").trim();
        const size = sizeOctal ? parseInt(sizeOctal, 8) : 0;
        offset += 512;
        if (!name) break;
        const content = tar.slice(offset, offset + size);
        offset += Math.ceil(size / 512) * 512;
        if (path.basename(name) === binName) {
          resolve(content);
          return;
        }
      }
      reject(new Error(`${binName} not found in archive`));
    });
    gunzip.on("error", reject);
    readable.pipe(gunzip);
  });
}

// ── zip extractor (uses system unzip on Unix, PowerShell on Windows) ─────────
async function extractZip(buf, binName, destDir) {
  fs.mkdirSync(destDir, { recursive: true });
  const tmpZip = path.join(destDir, "_vani_tmp.zip");
  fs.writeFileSync(tmpZip, buf);
  if (IS_WIN) {
    execSync(
      `powershell -Command "Expand-Archive -Force '${tmpZip}' '${destDir}'"`,
      { stdio: "inherit" }
    );
  } else {
    execSync(`unzip -o "${tmpZip}" "${binName}" -d "${destDir}"`, { stdio: "inherit" });
  }
  fs.unlinkSync(tmpZip);
  const extracted = path.join(destDir, binName);
  return fs.readFileSync(extracted);
}

// ── Main ─────────────────────────────────────────────────────────────────────
async function main() {
  // Skip in CI environments that don't need the binary (e.g. publishing itself)
  if (process.env.VANI_SKIP_INSTALL) {
    console.log("vani-mcp: skipping binary download (VANI_SKIP_INSTALL set)");
    return;
  }

  // Already installed (e.g. re-running postinstall)
  if (fs.existsSync(BIN_PATH)) {
    console.log(`vani-mcp: binary already present at ${BIN_PATH}`);
    return;
  }

  let target;
  try {
    target = getTarget();
  } catch (e) {
    console.warn(`vani-mcp: ${e.message} — skipping binary download.`);
    console.warn("Install manually: https://github.com/eniyos/vani-protocol/releases");
    return;
  }

  const assetName = getAssetName(target);
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${assetName}`;

  console.log(`vani-mcp: downloading ${assetName} …`);
  let buf;
  try {
    buf = await fetch(url);
  } catch (e) {
    console.warn(`vani-mcp: download failed (${e.message}).`);
    console.warn(`Install manually from: https://github.com/${REPO}/releases`);
    return; // non-fatal: npm install still succeeds
  }

  fs.mkdirSync(BIN_DIR, { recursive: true });

  let binBytes;
  if (assetName.endsWith(".zip")) {
    binBytes = await extractZip(buf, BIN_NAME, BIN_DIR);
  } else {
    binBytes = await extractTarGz(buf, BIN_NAME);
  }

  fs.writeFileSync(BIN_PATH, binBytes, { mode: 0o755 });
  console.log(`vani-mcp: installed to ${BIN_PATH}`);
}

main().catch((e) => {
  // Never fail npm install — just warn
  console.warn("vani-mcp postinstall warning:", e.message);
});
