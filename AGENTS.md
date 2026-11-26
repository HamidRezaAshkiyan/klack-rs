# AGENTS.md — klack-rs

## Overview

**Klack-rs** is a cross-platform (macOS, Windows, Linux) mechanical keyboard, mouse, and scroll sound engine achieving 1:1 feature parity with Klack v2.2 (`tryklack.com`), built with **Rust + Tauri v2 + React 19 + TypeScript + Tailwind CSS 4** and powered by the 55 MIT-licensed multi-sampled audio sprites from `joshwcomeau/button-sounds`.

## Tech Stack

- **Backend (`src-tauri/`, Rust edition 2021)**: Tauri `2` (`tray-icon`, `image-png`, `autostart`, `global-shortcut`, `notification`, `dialog`, `opener`), `cpal` `0.15`, `symphonia` `0.5` (`mp3`, `wav`, `pcm`), `crossbeam-channel`, `fastrand`, `parking_lot`, `chrono`, `core-graphics` (macOS), `windows-sys` (Windows), `rdev` (Linux).
- **Frontend (`src/`)**: React `19.1`, TypeScript `5.8`, Vite `6.3` (`http://localhost:1420`), Tailwind CSS `4.1`, Zustand `5`, `lucide-react`.

## Architecture

- **Audio Hot Path (`audio_engine.rs`, `sound_bank.rs`)**: All sprites follow `button-sounds`'s `SHARED_SPRITE_MAP` (`down[i]` at `(i+1)*1000ms` for 190ms; `up[i]` at `(i+1)*1000ms+200ms` for 790ms). Sprites are decoded once via `symphonia` into stereo-ready mono `f32` PCM slices at the active `cpal` sample rate and played via a lock-free 32-voice real-time mixer supporting pitch randomization, Tone Pad XY DSP, constant-power stereo pan, and headphone ITD micro-delay.
- **Global Input (`input_hook.rs`, `platform/{macos,windows,linux}.rs`)**: Passive OS hooks (`CGEventTap` on macOS, `WH_KEYBOARD_LL`/`WH_MOUSE_LL` on Windows, `rdev` on Linux) normalize events into `(KeyZone, pan, KeyEdge)` with `RepeatFilter` deduplication.
- **Sleep Triggers & Stats (`sleep_monitor.rs`, `stats.rs`, `storage.rs`)**: Background thread samples OS context every 2s for sleep triggers and persists config/stats JSON to Tauri `app_data_dir()`.

## Commands

```bash
npm install
npm run build                    # TypeScript check + Vite production build
cd src-tauri && cargo test       # Rust unit tests (audio mixer, sprite slicer, hooks, stats, storage)
npm run tauri:dev                # Run desktop app in dev mode
npm run tauri:build              # Build native installer
./scripts/fetch-sounds.sh        # Refresh 55 MP3 sprites (macOS/Linux)
pwsh ./scripts/fetch-sounds.ps1  # Refresh 55 MP3 sprites (Windows PowerShell 7)
```
