# Klack-rs

**Cross-platform mechanical keyboard, mouse, and scroll sound engine compatible 1:1 with [Klack v2.2](https://tryklack.com/).**

Built with **Rust**, **Tauri v2**, **cpal** (lock-free real-time native audio mixer), and **React 19 + Tailwind CSS 4** for **macOS**, **Windows**, and **Linux**. Powered by the MIT-licensed multi-sampled audio sprites from [joshwcomeau/button-sounds](https://github.com/joshwcomeau/button-sounds).

## Features (1:1 Parity with Klack v2.2)

- **Zero-Lag Native Audio Engine**: Pre-decoded 48 kHz `f32` PCM slices mixed directly inside a lock-free `cpal` real-time callback (CoreAudio / WASAPI / PipeWire) — zero WebView IPC on the keystroke hot path.
- **Up / Down Keystroke Dynamics**: Every switch sprite has 4–8 separate `down` and `up` recordings with OS key-repeat deduplication so held keys never stutter.
- **8 Curated Switch Sets + Mix & Match Builder**: Leobog Hi75, NuPhy Halo, UHK Tactile, ASUS Mechanical, ErgoDox MX, Studio Pads, Cardboard Thump, and Retro Arcade — plus a 55-sprite custom Mix & Match Switch Builder with instant Preview.
- **Interactive 2D Tone Pad**: Real-time DSP pad sculpting **Warm/Thocky ↔ Crisp/Clacky** (pitch + tilt filter) and **Damped ↔ Punchy** (decay & transient punch).
- **Immersive Spatial Audio**: Physical key positions mapped to constant-power stereo pan (`[-0.75, +0.75]`) with optional Interaural Time Delay (ITD) for headphones.
- **Randomized Pitching & Anti-Repetition**: Never repeats the same sample twice in a row, with configurable pitch jitter and micro-gain variation.
- **Mouse Click & Scroll Wheel Sounds**: Dedicated left/right/middle mouse clicks (`mx-master-3`, `rival-3`, `asus-mouse`) and velocity-reactive rotary scroll ticks (`hi75-knob`) with configurable debounce.
- **Sleep Triggers**: Auto-pause on Gaming/Fullscreen, Screen Sharing/Calls (Zoom, Discord, OBS, Teams), Focus/Do Not Disturb, Media Playback, or custom Excluded Apps.
- **Local Typing Stats**: Keystrokes today/all-time, live & peak WPM, mouse clicks, scroll ticks, and key-zone distribution — 100% local, zero keystroke text ever logged.
- **Menubar / System Tray Integration**: Dynamic tray icon (active / muted / sleeping), quick popover mode, customizable global toggle shortcut (`Option+Cmd+K` / `Ctrl+Alt+K`), Launch at Login, and macOS Accessibility onboarding.

## Development

### Prerequisites
- Rust (stable, 1.80+)
- Node.js 20+ & npm
- Platform Tauri v2 build prerequisites (Xcode CLT on macOS, WebView2/MSVC on Windows, `libwebkit2gtk-4.1-dev` + `libasound2-dev` + `libxtst-dev` on Linux)

### Setup & Run
```bash
# 1. Install frontend dependencies
npm install

# 2. (Optional) Re-download all 55 MIT-licensed MP3 sprites
./scripts/fetch-sounds.sh        # macOS / Linux
# pwsh ./scripts/fetch-sounds.ps1 # Windows (PowerShell 7+)

# 3. Run in development mode
npm run tauri:dev

# 4. Run automated tests & type-checking
npm run build
cd src-tauri && cargo test

# 5. Build native installers
npm run tauri:build
```

## Privacy
Klack-rs **never** logs, stores, or transmits the text you type. Global input hooks only read physical key-zone coordinates (`alpha`, `space`, `enter`, etc.) to trigger audio and increment anonymous zone counters.

## License & Credits
- **Klack-rs**: [MIT License](./LICENSE)
- **Bundled Audio Sprites**: Sourced from [joshwcomeau/button-sounds](https://github.com/joshwcomeau/button-sounds) under the MIT License (see [THIRD_PARTY_LICENSES.md](./THIRD_PARTY_LICENSES.md)).
