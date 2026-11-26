use crate::audio_engine::AudioEngine;
use crate::input_hook::{InputEvent, RepeatFilter};
use crate::models::{
    AppConfig, KeyEdge, KeyZone, MouseButtonKind, SleepStatus, SwitchPack,
};
use crate::sound_bank::{builtin_switch_packs, SoundBank};
use crate::stats::StatsTracker;
use crate::storage::Storage;
use parking_lot::{Mutex, RwLock};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub sleep_status: RwLock<SleepStatus>,
    pub storage: Storage,
    pub audio: AudioEngine,
    pub sound_bank: RwLock<Arc<SoundBank>>,
    pub stats: StatsTracker,
    pub repeat_filter: RepeatFilter,
    pub hook_active: Arc<AtomicBool>,
    last_scroll_tick: Mutex<Instant>,
}

impl AppState {
    pub fn new(data_dir: PathBuf, sounds_dir: PathBuf) -> Arc<Self> {
        let storage = Storage::new(data_dir);
        let config = storage.load_config();
        let initial_stats = storage.load_stats();

        let audio = AudioEngine::new(config.volume, config.output_device.as_deref());
        let sr = audio.sample_rate();
        let sound_bank = Arc::new(SoundBank::new(sounds_dir, sr));

        let state = Arc::new(Self {
            config: RwLock::new(config),
            sleep_status: RwLock::new(SleepStatus::default()),
            storage,
            audio,
            sound_bank: RwLock::new(sound_bank),
            stats: StatsTracker::new(initial_stats),
            repeat_filter: RepeatFilter::default(),
            hook_active: Arc::new(AtomicBool::new(false)),
            last_scroll_tick: Mutex::new(Instant::now() - Duration::from_secs(1)),
        });

        state.preload_active_assets();
        state
    }

    pub fn all_switch_packs(&self) -> Vec<SwitchPack> {
        let mut packs = builtin_switch_packs();
        let cfg = self.config.read();
        packs.extend(cfg.custom_packs.clone());
        packs
    }

    pub fn active_switch_pack(&self) -> SwitchPack {
        let active_id = self.config.read().active_switch_id.clone();
        self.all_switch_packs()
            .into_iter()
            .find(|p| p.id == active_id)
            .unwrap_or_else(|| builtin_switch_packs().remove(0))
    }

    pub fn preload_active_assets(&self) {
        let pack = self.active_switch_pack();
        let cfg = self.config.read();
        let sprites = vec![
            pack.zones.alpha.sprite.clone(),
            pack.zones.space.sprite.clone(),
            pack.zones.enter.sprite.clone(),
            pack.zones.backspace.sprite.clone(),
            pack.zones.modifier.sprite.clone(),
            pack.zones.function.sprite.clone(),
            pack.zones.numpad.sprite.clone(),
            cfg.mouse.sound_id.clone(),
            cfg.mouse.middle_sound_id.clone(),
            cfg.scroll.sound_id.clone(),
        ];
        let bank = Arc::clone(&self.sound_bank.read());
        bank.preload_sprites(&sprites);
    }

    pub fn rebuild_sound_bank_for_rate(&self, new_rate: u32) {
        let sounds_dir = self.sound_bank.read().sounds_dir().to_path_buf();
        let current_rate = self.sound_bank.read().target_sample_rate();
        if current_rate != new_rate {
            *self.sound_bank.write() = Arc::new(SoundBank::new(sounds_dir, new_rate));
            self.preload_active_assets();
        }
    }

    /// Hot-path input event handler called directly from the native OS hook thread.
    pub fn handle_input_event(&self, event: InputEvent) {
        let cfg = self.config.read().clone();
        if !cfg.enabled || self.sleep_status.read().is_sleeping {
            return;
        }

        let bank = Arc::clone(&self.sound_bank.read());
        let sr = self.audio.sample_rate();

        match event {
            InputEvent::Key {
                code,
                zone,
                pan,
                edge,
            } => {
                if !self.repeat_filter.should_emit(code, edge) {
                    return;
                }
                if zone == KeyZone::Modifier && !cfg.key_categories.play_modifiers {
                    return;
                }
                if zone == KeyZone::Function && !cfg.key_categories.play_function_keys {
                    return;
                }
                if zone == KeyZone::Numpad && !cfg.key_categories.play_numpad {
                    return;
                }
                if edge == KeyEdge::Up && !cfg.key_categories.play_key_up {
                    return;
                }

                let pack = self.active_switch_pack();
                let spec = pack.zones.for_zone(zone);
                let is_down = edge == KeyEdge::Down;
                if let Some(sample) = bank.pick_sample(&spec.sprite, is_down) {
                    let edge_gain = if is_down {
                        1.0
                    } else {
                        cfg.key_categories.key_up_volume
                    };
                    let effective_pan = if cfg.spatial.enabled {
                        (pan * cfg.spatial.width).clamp(-0.9, 0.9)
                    } else {
                        0.0
                    };
                    let cmd = AudioEngine::build_voice_cmd(
                        sample,
                        spec.gain_scale * edge_gain,
                        spec.pitch_scale,
                        cfg.pitch_randomization,
                        effective_pan,
                        cfg.spatial.enabled && cfg.spatial.itd_enabled,
                        sr,
                        &cfg.tone_pad,
                    );
                    self.audio.play(cmd);
                }

                if is_down {
                    self.stats.record_keystroke(zone, &pack.id);
                }
            }
            InputEvent::MouseButton { button, edge } => {
                if !cfg.mouse.enabled {
                    return;
                }
                if edge == KeyEdge::Up && !cfg.mouse.play_release {
                    return;
                }
                let sprite_id = if button == MouseButtonKind::Middle {
                    &cfg.mouse.middle_sound_id
                } else {
                    &cfg.mouse.sound_id
                };
                let is_down = edge == KeyEdge::Down;
                if let Some(sample) = bank.pick_sample(sprite_id, is_down) {
                    let edge_gain = if is_down { 1.0 } else { 0.65 };
                    let pan = if cfg.spatial.enabled {
                        0.28 * cfg.spatial.width
                    } else {
                        0.0
                    };
                    let cmd = AudioEngine::build_voice_cmd(
                        sample,
                        cfg.mouse.volume * edge_gain,
                        1.0,
                        cfg.pitch_randomization * 0.7,
                        pan,
                        cfg.spatial.enabled && cfg.spatial.itd_enabled,
                        sr,
                        &cfg.tone_pad,
                    );
                    self.audio.play(cmd);
                }
                if is_down {
                    self.stats.record_mouse_click();
                }
            }
            InputEvent::Scroll { delta } => {
                if !cfg.scroll.enabled {
                    return;
                }
                let now = Instant::now();
                {
                    let mut last = self.last_scroll_tick.lock();
                    if now.duration_since(*last) < Duration::from_millis(cfg.scroll.min_interval_ms)
                    {
                        return;
                    }
                    *last = now;
                }

                if let Some(sample) = bank.pick_sample(&cfg.scroll.sound_id, true) {
                    let velocity_pitch = (0.96 + (delta.abs().min(10.0) * 0.015)).clamp(0.92, 1.18);
                    let pan = if cfg.spatial.enabled {
                        0.28 * cfg.spatial.width
                    } else {
                        0.0
                    };
                    let cmd = AudioEngine::build_voice_cmd(
                        sample,
                        cfg.scroll.volume,
                        velocity_pitch,
                        cfg.pitch_randomization * 0.5,
                        pan,
                        false,
                        sr,
                        &cfg.tone_pad,
                    );
                    self.audio.play(cmd);
                }
                self.stats.record_scroll_tick();
            }
        }
    }

    /// Plays a short preview burst for a switch pack (or a single sprite) from the UI.
    pub fn preview_switch_pack(&self, pack_id: &str) {
        let pack = self
            .all_switch_packs()
            .into_iter()
            .find(|p| p.id == pack_id)
            .unwrap_or_else(|| self.active_switch_pack());
        let cfg = self.config.read().clone();
        let bank = Arc::clone(&self.sound_bank.read());
        let sr = self.audio.sample_rate();

        let sequence = [
            (KeyZone::Alpha, -0.45f32, true, 0u64),
            (KeyZone::Alpha, -0.45f32, false, 65u64),
            (KeyZone::Alpha, -0.10f32, true, 125u64),
            (KeyZone::Alpha, -0.10f32, false, 185u64),
            (KeyZone::Alpha, 0.30f32, true, 240u64),
            (KeyZone::Alpha, 0.30f32, false, 300u64),
            (KeyZone::Space, 0.0f32, true, 365u64),
            (KeyZone::Space, 0.0f32, false, 435u64),
            (KeyZone::Enter, 0.52f32, true, 510u64),
            (KeyZone::Enter, 0.52f32, false, 580u64),
        ];

        // Trigger first stroke immediately, rest on a tiny background thread so UI never blocks
        for (zone, pan, is_down, delay_ms) in sequence {
            let spec = pack.zones.for_zone(zone).clone();
            let sample = bank.pick_sample(&spec.sprite, is_down);
            if let Some(sample) = sample {
                let effective_pan = if cfg.spatial.enabled {
                    pan * cfg.spatial.width
                } else {
                    0.0
                };
                let edge_gain = if is_down {
                    1.0
                } else {
                    cfg.key_categories.key_up_volume
                };
                let cmd = AudioEngine::build_voice_cmd(
                    sample,
                    spec.gain_scale * edge_gain,
                    spec.pitch_scale,
                    cfg.pitch_randomization,
                    effective_pan,
                    cfg.spatial.enabled && cfg.spatial.itd_enabled,
                    sr,
                    &cfg.tone_pad,
                );
                if delay_ms == 0 {
                    self.audio.play(cmd);
                } else {
                    // Schedule via lightweight detached sleep
                    let tx_cmd = cmd;
                    let vol_engine = self.audio_sender_clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(delay_ms));
                        let _ = vol_engine.try_send(tx_cmd);
                    });
                }
            }
        }
    }

    pub fn preview_sprite(&self, sprite_id: &str, volume_override: Option<f32>) {
        let cfg = self.config.read().clone();
        let bank = Arc::clone(&self.sound_bank.read());
        let sr = self.audio.sample_rate();
        let gain = volume_override.unwrap_or(1.0);

        if let Some(down) = bank.pick_sample(sprite_id, true) {
            let cmd = AudioEngine::build_voice_cmd(
                down,
                gain,
                1.0,
                cfg.pitch_randomization,
                0.0,
                false,
                sr,
                &cfg.tone_pad,
            );
            self.audio.play(cmd);
        }
        if let Some(up) = bank.pick_sample(sprite_id, false) {
            let cmd = AudioEngine::build_voice_cmd(
                up,
                gain * cfg.key_categories.key_up_volume,
                1.0,
                cfg.pitch_randomization,
                0.0,
                false,
                sr,
                &cfg.tone_pad,
            );
            let sender = self.audio_sender_clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(85));
                let _ = sender.try_send(cmd);
            });
        }
    }

    fn audio_sender_clone(&self) -> crossbeam_channel::Sender<crate::audio_engine::VoiceCmd> {
        self.audio.clone_sender()
    }
}
