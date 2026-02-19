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
- Model download and local model management (`tiny`, `base`, `large`)

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

## License

Add your preferred license in `LICENSE`.
