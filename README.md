# OpenSpeak

OpenSpeak is a local-first desktop dictation app built with Tauri, Rust, and whisper.cpp.

It is designed for fast global dictation workflows:
- Trigger recording from a global hotkey
- Speak naturally
- Stop and insert text where you are working

## Features

- Local transcription via `whisper.cpp` (`whisper-rs`)
- Global hotkey toggle for start/stop dictation
- Menu bar (tray-first) app flow on macOS
- Recording overlay HUD for background visual feedback
- Two output modes:
  - `clipboard`: copy text for manual paste
  - `auto-paste`: copy then trigger paste automatically
- Basic spoken formatting commands:
  - `comma`, `period`, `question mark`
  - `new line`, `new paragraph`
- Model download and local model management:
  - Tiny (75 MB, fastest)
  - Base (142 MB)
  - Small (466 MB, recommended default)
  - Medium (1.5 GB)
  - Large-v3 (3.1 GB, highest accuracy)
  - Turbo (fast large model)

## Architecture

- Frontend: React + Vite
- Desktop shell: Tauri v2
- Core runtime: Rust
- Audio capture: `cpal`
- Inference: `whisper-rs` / `whisper.cpp`

## Requirements

- macOS (current focus)
- Node.js 20+
- Rust toolchain (`rustup`, `cargo`)
- Xcode Command Line Tools
- `cmake` (required by whisper build)

Install `cmake`:

```bash
brew install cmake
```

## Getting Started

Install dependencies:

```bash
npm install
```

Run in development:

```bash
cargo tauri dev
```

Production build:

```bash
cargo tauri build
```

Build outputs:
- `.app`: `src-tauri/target/release/bundle/macos/`

## Install (GitHub Releases, macOS)

Current release artifacts are unsigned macOS app bundles (`.app.tar.gz`).

1. Download the latest `OpenSpeak_*.app.tar.gz` from GitHub Releases.
2. Extract it and move `OpenSpeak.app` to `/Applications`.
3. Remove the quarantine flag once:

```bash
xattr -rd com.apple.quarantine /Applications/OpenSpeak.app
```

4. Launch OpenSpeak from Applications.

If macOS still blocks launch, use Finder `Right-click -> Open` once and confirm.

## Permissions (macOS)

For full functionality, OpenSpeak needs:
- Microphone access (record speech)
- Accessibility access (auto-paste automation)

If `auto-paste` is disabled, Accessibility permission is optional.

## Model Storage

By default, model files are stored under:

`~/Library/Application Support/openspeak/models/`

OpenSpeak also supports legacy paths from earlier project naming and will continue to read existing local data if present.

## Configuration

OpenSpeak persists settings in local app data, including:
- Global hotkey
- Default model
- Paste mode
- Privacy flags

Default hotkey:

`CommandOrControl+Shift+Space`

## Development Notes

- The app runs tray-first by default; open settings from the tray menu.
- Closing the settings window hides it instead of quitting.
- The overlay is a separate transparent always-on-top window.

## GitHub Releases

This repository includes a GitHub Actions release workflow:

- File: `.github/workflows/release.yml`
- Trigger: push a semantic version tag (`v*.*.*`)
- Runner: `macos-latest`
- Artifacts uploaded to GitHub Releases via `tauri-action`

### Create a release

1. Bump versions if needed (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`).
2. Commit to `main`.
3. Create and push a tag:
   - `git tag v0.1.1`
   - `git push origin v0.1.1`
4. Wait for the `Release` workflow to complete.
5. Verify assets in the GitHub Releases tab.

### Unsigned build note

Because release builds are currently unsigned, users may need to run:

```bash
xattr -rd com.apple.quarantine /Applications/OpenSpeak.app
```

### Optional: signing and notarization (recommended)

The workflow supports macOS signing/notarization if these repository secrets are set:

- `APPLE_SIGNING_IDENTITY`
- `APPLE_CERTIFICATE` (base64-encoded `.p12`)
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_ID`
- `APPLE_PASSWORD` (app-specific password)
- `APPLE_TEAM_ID`

If these secrets are not set, builds can still complete, but distribution UX is typically better with signed/notarized artifacts.

## License

Add your preferred license in `LICENSE`.
