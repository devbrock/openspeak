# Brock's Dictation Tool

Cross-platform dictation app scaffold, implemented as a macOS-first Tauri + Rust desktop product with local-only transcription architecture.

## Current status

This repository now includes:

- Tauri/React app shell and settings UI.
- Rust command surface (`start_recording`, `stop_recording`, `set_hotkey`, `set_model`, `download_model`, `get_status`, `get_config`).
- Core modules for config, parser, model manager, live microphone capture, whisper.cpp transcription, and output injection.
- macOS-focused output strategy: clipboard paste with clipboard restore.
- Basic deterministic voice-command parser, including punctuation and multi-word commands (`new line`, `new paragraph`, `question mark`).

## Local setup

### 1) Install prerequisites

- Node.js 20+
- Rust toolchain (`rustup`, `rustc`, `cargo`)
- Tauri system dependencies for macOS
- `cmake` (required by whisper.cpp Rust bindings)

```bash
brew install cmake
```

### 2) Install frontend deps

```bash
npm install
```

### 3) Run desktop app

```bash
cargo tauri dev
```

On first transcription, the app downloads the selected whisper.cpp model to:
`~/Library/Application Support/brocks-dictation-tool/models/`

When transcription completes, text is copied to clipboard. Paste into the target app with `Cmd+V`.

## Next implementation steps

1. Add global hotkey registration in `platform/macos.rs`.
2. Add model checksum verification and resumable download.
3. Add confidence/latency telemetry panel in the desktop UI.
4. Add integration tests across TextEdit, Terminal/iTerm, and browser text fields.
