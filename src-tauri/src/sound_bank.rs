use crate::models::{SpriteInfo, SwitchPack, SwitchZoneMapping, ZoneSoundSpec};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use thiserror::Error;

/// Exact sprite timing from joshwcomeau/button-sounds:
/// - down[i] starts at (i + 1) * 1000ms, duration 190ms
/// - up[i]   starts at (i + 1) * 1000ms + 200ms, duration 790ms
const DOWN_OFFSET_MS: usize = 0;
const DOWN_DURATION_MS: usize = 190;
const UP_OFFSET_MS: usize = 200;
const UP_DURATION_MS: usize = 790;
const SLOT_STRIDE_MS: usize = 1000;

#[derive(Debug, Error)]
pub enum SoundBankError {
    #[error("sound file not found: {0}")]
    NotFound(PathBuf),
    #[error("failed to open audio file: {0}")]
    Io(#[from] std::io::Error),
    #[error("audio decode error: {0}")]
    Decode(String),
}

/// All 55 MIT-licensed sprites from joshwcomeau/button-sounds with their exact pair count.
pub fn sprite_catalog() -> Vec<SpriteInfo> {
    let entries: &[(&str, &str, &str, usize)] = &[
        ("hi75-soft", "Leobog Hi75 Soft", "Keyboard", 7),
        ("hi75-medium", "Leobog Hi75 Medium", "Keyboard", 8),
        ("hi75-hard", "Leobog Hi75 Hard", "Keyboard", 6),
        ("hi75-space", "Leobog Hi75 Spacebar", "Keyboard", 6),
        ("hi75-knob", "Leobog Hi75 Rotary Knob", "Scroll", 8),
        ("halo-soft", "NuPhy Halo Soft", "Keyboard", 6),
        ("halo-hard", "NuPhy Halo Hard", "Keyboard", 6),
        ("halo-shift", "NuPhy Halo Shift", "Keyboard", 7),
        ("halo-space", "NuPhy Halo Spacebar", "Keyboard", 7),
        ("uhk-soft", "UHK Soft", "Keyboard", 6),
        ("uhk-medium", "UHK Medium", "Keyboard", 7),
        ("uhk-space", "UHK Spacebar", "Keyboard", 4),
        ("asus-kbd-soft", "ASUS Mech Soft", "Keyboard", 8),
        ("asus-kbd-medium", "ASUS Mech Medium", "Keyboard", 6),
        ("asus-kbd-hard", "ASUS Mech Hard", "Keyboard", 5),
        ("asus-kbd-space", "ASUS Mech Spacebar", "Keyboard", 6),
        ("ergo-mx", "ErgoDox Cherry MX", "Keyboard", 6),
        ("mx-master-3", "Logitech MX Master 3", "Mouse", 4),
        ("rival-3", "SteelSeries Rival 3", "Mouse", 7),
        ("rival-3-middle", "SteelSeries Rival 3 Middle", "Mouse", 6),
        ("asus-mouse", "ASUS Gaming Mouse", "Mouse", 7),
        ("sd-standard", "Stream Deck Standard", "Tactile", 7),
        ("sd-clicky", "Stream Deck Clicky", "Tactile", 4),
        ("sd-metallic", "Stream Deck Metallic", "Tactile", 7),
        ("sd-resonant", "Stream Deck Resonant", "Tactile", 6),
        ("sd-squelchy", "Stream Deck Squelchy", "Tactile", 6),
        ("sd-distorted", "Stream Deck Distorted", "Tactile", 5),
        ("ps5-snappy", "DualSense Snappy", "Gamepad", 8),
        ("ps5-clicky", "DualSense Clicky", "Gamepad", 7),
        ("ps5-dull", "DualSense Dull", "Gamepad", 7),
        ("ps5-double", "DualSense Double", "Gamepad", 8),
        ("ps5-paddle", "DualSense Paddle", "Gamepad", 6),
        ("low-thumps", "Low Thumps", "Tactile", 6),
        ("raddy-low", "Raddy Low", "Tactile", 5),
        ("raddy-medium", "Raddy Medium", "Tactile", 6),
        ("raddy-high", "Raddy High", "Tactile", 6),
        ("iidx", "Arcade IIDX", "Arcade", 4),
        ("treadmill-sharp", "Treadmill Sharp", "Tactile", 4),
        ("treadmill-flinty", "Treadmill Flinty", "Tactile", 4),
        ("treadmill-pitched", "Treadmill Pitched", "Tactile", 4),
        ("camera-latch", "Camera Latch", "Hardware", 5),
        ("carson-microflip", "Carson Microflip", "Scroll", 5),
        ("kvm-switch", "KVM Switch", "Hardware", 5),
        ("lock-open", "Lock Open", "Scroll", 6),
        ("lock-closed", "Lock Closed", "Scroll", 5),
        ("popper-dice", "Popper Dice", "Hardware", 5),
        ("popper-empty", "Popper Empty", "Hardware", 5),
        ("guitar-toggle-on", "Guitar Toggle On", "Hardware", 6),
        ("guitar-toggle-off", "Guitar Toggle Off", "Hardware", 6),
        ("heated-blanket", "Heated Blanket", "Hardware", 7),
        ("led-blacklight", "LED Blacklight", "Hardware", 4),
        ("synth-on", "Synth On", "Synth", 8),
        ("synth-off", "Synth Off", "Synth", 8),
        ("tonal-on", "Tonal On", "Synth", 4),
        ("tonal-off", "Tonal Off", "Synth", 4),
    ];

    entries
        .iter()
        .map(|(id, label, category, pairs)| SpriteInfo {
            id: (*id).to_string(),
            label: (*label).to_string(),
            category: (*category).to_string(),
            sample_pairs: *pairs,
        })
        .collect()
}

pub fn sample_pair_count(sprite_id: &str) -> usize {
    sprite_catalog()
        .into_iter()
        .find(|s| s.id == sprite_id)
        .map(|s| s.sample_pairs)
        .unwrap_or(4)
}

fn spec(sprite: &str, pitch_scale: f32, gain_scale: f32) -> ZoneSoundSpec {
    ZoneSoundSpec {
        sprite: sprite.to_string(),
        pitch_scale,
        gain_scale,
    }
}

fn category_accent(category: &str) -> &'static str {
    match category {
        "Keyboard" => "#f59e0b",
        "Mouse" => "#38bdf8",
        "Scroll" => "#22d3ee",
        "Tactile" => "#34d399",
        "Gamepad" => "#ec4899",
        "Arcade" => "#f43f5e",
        "Hardware" => "#a78bfa",
        "Synth" => "#818cf8",
        _ => "#f59e0b",
    }
}

/// 8 curated multi-zone Klack-style Switch Sets + all 55 individual button-sounds presets (63 total).
pub fn builtin_switch_packs() -> Vec<SwitchPack> {
    let mut packs = vec![
        SwitchPack {
            id: "leobog-hi75".into(),
            name: "Leobog Hi75".into(),
            brand: "Leobog".into(),
            feel: "Thocky Tactile".into(),
            category: "Curated".into(),
            description: "Deep, resonant gasket-mounted thock with dedicated spacebar and heavy modifier acoustics.".into(),
            accent_color: "#f59e0b".into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec("hi75-soft", 1.0, 1.0),
                space: spec("hi75-space", 1.0, 1.08),
                enter: spec("hi75-medium", 0.96, 1.05),
                backspace: spec("hi75-medium", 1.02, 1.02),
                modifier: spec("hi75-hard", 0.95, 0.95),
                function: spec("hi75-soft", 1.08, 0.92),
                numpad: spec("hi75-medium", 1.0, 0.96),
            },
        },
        SwitchPack {
            id: "nuphy-halo".into(),
            name: "NuPhy Halo".into(),
            brand: "NuPhy".into(),
            feel: "Creamy Linear".into(),
            category: "Curated".into(),
            description: "Smooth, creamy low-profile mechanical strokes with dedicated shift and ghostbar recordings.".into(),
            accent_color: "#38bdf8".into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec("halo-soft", 1.0, 1.0),
                space: spec("halo-space", 1.0, 1.08),
                enter: spec("halo-hard", 0.97, 1.05),
                backspace: spec("halo-hard", 1.03, 1.02),
                modifier: spec("halo-shift", 1.0, 0.96),
                function: spec("halo-soft", 1.07, 0.9),
                numpad: spec("halo-soft", 0.98, 0.96),
            },
        },
        SwitchPack {
            id: "uhk-tactile".into(),
            name: "UHK Tactile".into(),
            brand: "Ultimate Hacking".into(),
            feel: "Crisp Office".into(),
            category: "Curated".into(),
            description: "Clean, articulate tactile feedback ideal for long coding sessions without harsh treble.".into(),
            accent_color: "#34d399".into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec("uhk-soft", 1.0, 1.0),
                space: spec("uhk-space", 1.0, 1.08),
                enter: spec("uhk-medium", 0.95, 1.04),
                backspace: spec("uhk-medium", 1.02, 1.0),
                modifier: spec("uhk-medium", 0.97, 0.92),
                function: spec("uhk-soft", 1.06, 0.9),
                numpad: spec("uhk-soft", 0.99, 0.95),
            },
        },
        SwitchPack {
            id: "asus-mechanical".into(),
            name: "ASUS Mechanical".into(),
            brand: "ASUS ROG".into(),
            feel: "Clicky Gaming".into(),
            category: "Curated".into(),
            description: "High-definition, punchy mechanical switches with crisp bottom-out and snappy return.".into(),
            accent_color: "#fb7185".into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec("asus-kbd-soft", 1.0, 1.0),
                space: spec("asus-kbd-space", 1.0, 1.08),
                enter: spec("asus-kbd-hard", 0.98, 1.06),
                backspace: spec("asus-kbd-hard", 1.03, 1.02),
                modifier: spec("asus-kbd-medium", 0.98, 0.96),
                function: spec("asus-kbd-soft", 1.07, 0.92),
                numpad: spec("asus-kbd-medium", 1.0, 0.98),
            },
        },
        SwitchPack {
            id: "ergodox-mx".into(),
            name: "ErgoDox MX".into(),
            brand: "Cherry MX".into(),
            feel: "Classic Mechanical".into(),
            category: "Curated".into(),
            description: "Iconic Cherry MX keyswitch character with subtle acoustic tuning per stabilizer zone.".into(),
            accent_color: "#a78bfa".into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec("ergo-mx", 1.0, 1.0),
                space: spec("ergo-mx", 0.84, 1.12),
                enter: spec("ergo-mx", 0.91, 1.06),
                backspace: spec("ergo-mx", 0.94, 1.04),
                modifier: spec("ergo-mx", 0.96, 0.94),
                function: spec("ergo-mx", 1.08, 0.92),
                numpad: spec("ergo-mx", 1.02, 0.96),
            },
        },
        SwitchPack {
            id: "studio-pads".into(),
            name: "Studio Pads".into(),
            brand: "Stream Deck".into(),
            feel: "Broadcast Tactile".into(),
            category: "Curated".into(),
            description: "Muted studio-console button clicks with metallic and resonant accents on large keys.".into(),
            accent_color: "#22d3ee".into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec("sd-standard", 1.0, 1.0),
                space: spec("sd-resonant", 0.94, 1.08),
                enter: spec("sd-clicky", 0.98, 1.05),
                backspace: spec("sd-metallic", 1.0, 1.0),
                modifier: spec("sd-squelchy", 0.96, 0.94),
                function: spec("sd-clicky", 1.08, 0.9),
                numpad: spec("sd-standard", 1.02, 0.95),
            },
        },
        SwitchPack {
            id: "cardboard-thump".into(),
            name: "Cardboard Thump".into(),
            brand: "Lo-Fi Acoustic".into(),
            feel: "Deep Muted".into(),
            category: "Curated".into(),
            description: "Warm, ultra-low thumpy acoustic pops inspired by Klack's beloved Cardboard switch set.".into(),
            accent_color: "#d97706".into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec("low-thumps", 1.0, 1.05),
                space: spec("low-thumps", 0.82, 1.15),
                enter: spec("ps5-dull", 0.92, 1.05),
                backspace: spec("ps5-dull", 1.0, 1.0),
                modifier: spec("raddy-low", 0.96, 0.95),
                function: spec("raddy-medium", 1.05, 0.9),
                numpad: spec("low-thumps", 1.04, 0.98),
            },
        },
        SwitchPack {
            id: "retro-arcade".into(),
            name: "Retro Arcade".into(),
            brand: "IIDX / Microswitch".into(),
            feel: "Snappy Micro".into(),
            category: "Curated".into(),
            description: "Razor-sharp microswitch snaps combining arcade buttons and DualSense tactile clicks.".into(),
            accent_color: "#ec4899".into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec("ps5-snappy", 1.0, 1.0),
                space: spec("iidx", 0.95, 1.1),
                enter: spec("ps5-clicky", 0.96, 1.06),
                backspace: spec("treadmill-sharp", 1.0, 1.0),
                modifier: spec("ps5-paddle", 0.98, 0.94),
                function: spec("treadmill-flinty", 1.04, 0.9),
                numpad: spec("ps5-double", 1.0, 0.95),
            },
        },
    ];

    // Add every individual multi-sampled sound from joshwcomeau/button-sounds as a ready-to-use preset
    for s in sprite_catalog() {
        packs.push(SwitchPack {
            id: format!("sprite-{}", s.id),
            name: s.label.clone(),
            brand: "button-sounds".into(),
            feel: format!("{} · {}× pairs", s.category, s.sample_pairs),
            category: s.category.clone(),
            description: format!(
                "Pure {} recording ({} down/up variations) with subtle stabilizer pitch tuning.",
                s.label, s.sample_pairs
            ),
            accent_color: category_accent(&s.category).into(),
            is_custom: false,
            zones: SwitchZoneMapping {
                alpha: spec(&s.id, 1.0, 1.0),
                space: spec(&s.id, 0.86, 1.1),
                enter: spec(&s.id, 0.93, 1.05),
                backspace: spec(&s.id, 0.96, 1.02),
                modifier: spec(&s.id, 0.95, 0.94),
                function: spec(&s.id, 1.06, 0.92),
                numpad: spec(&s.id, 1.01, 0.96),
            },
        });
    }

    packs
}

#[derive(Debug, Clone)]
pub struct DecodedSprite {
    pub down_samples: Vec<Arc<[f32]>>,
    pub up_samples: Vec<Arc<[f32]>>,
}

/// Thread-safe cache of pre-decoded & pre-sliced mono f32 PCM sprites at the active output sample rate.
pub struct SoundBank {
    sounds_dir: PathBuf,
    target_sample_rate: u32,
    cache: Mutex<HashMap<String, Arc<DecodedSprite>>>,
    last_index: Mutex<HashMap<String, usize>>,
}

impl SoundBank {
    pub fn new(sounds_dir: PathBuf, target_sample_rate: u32) -> Self {
        Self {
            sounds_dir,
            target_sample_rate,
            cache: Mutex::new(HashMap::new()),
            last_index: Mutex::new(HashMap::new()),
        }
    }

    pub fn sounds_dir(&self) -> &Path {
        &self.sounds_dir
    }

    pub fn target_sample_rate(&self) -> u32 {
        self.target_sample_rate
    }

    /// Preload all sprites referenced by a switch pack + mouse + scroll so hot-path playback never touches disk.
    pub fn preload_sprites(&self, sprite_ids: &[String]) {
        for id in sprite_ids {
            if let Err(err) = self.get_or_load(id) {
                tracing::warn!("failed to preload sprite '{}': {}", id, err);
            }
        }
    }

    pub fn get_or_load(&self, sprite_id: &str) -> Result<Arc<DecodedSprite>, SoundBankError> {
        {
            let guard = self.cache.lock();
            if let Some(existing) = guard.get(sprite_id) {
                return Ok(Arc::clone(existing));
            }
        }

        let mp3_path = self.sounds_dir.join(format!("{sprite_id}.mp3"));
        let wav_path = self.sounds_dir.join(format!("{sprite_id}.wav"));
        let path = if mp3_path.exists() {
            mp3_path
        } else if wav_path.exists() {
            wav_path
        } else {
            return Err(SoundBankError::NotFound(mp3_path));
        };

        let (mono_pcm, src_rate) = decode_audio_to_mono(&path)?;
        let resampled = if src_rate != self.target_sample_rate {
            linear_resample(&mono_pcm, src_rate, self.target_sample_rate)
        } else {
            mono_pcm
        };

        let pair_count = sample_pair_count(sprite_id);
        let decoded = Arc::new(slice_sprite_pcm(
            &resampled,
            self.target_sample_rate,
            pair_count,
        ));

        let mut guard = self.cache.lock();
        guard.insert(sprite_id.to_string(), Arc::clone(&decoded));
        Ok(decoded)
    }

    /// Picks a random sample slice for `sprite_id` + edge (`is_down`), avoiding repeating the exact last index.
    pub fn pick_sample(&self, sprite_id: &str, is_down: bool) -> Option<Arc<[f32]>> {
        let sprite = self.get_or_load(sprite_id).ok()?;
        let pool = if is_down {
            &sprite.down_samples
        } else {
            &sprite.up_samples
        };
        if pool.is_empty() {
            return None;
        }
        let key = format!("{}:{}", sprite_id, if is_down { "d" } else { "u" });
        let idx = pick_non_repeating_index(pool.len(), &key, &self.last_index);
        Some(Arc::clone(&pool[idx]))
    }
}

fn pick_non_repeating_index(
    len: usize,
    key: &str,
    last_map: &Mutex<HashMap<String, usize>>,
) -> usize {
    if len <= 1 {
        return 0;
    }
    let mut guard = last_map.lock();
    let prev = guard.get(key).copied();
    let mut idx = fastrand::usize(0..len);
    if Some(idx) == prev {
        idx = (idx + 1 + fastrand::usize(0..len - 1)) % len;
    }
    guard.insert(key.to_string(), idx);
    idx
}

/// Slices a decoded mono PCM buffer according to joshwcomeau/button-sounds's shared sprite map.
pub fn slice_sprite_pcm(pcm: &[f32], sample_rate: u32, pair_count: usize) -> DecodedSprite {
    let sr = sample_rate as usize;
    let ms_to_samples = |ms: usize| -> usize { (ms * sr) / 1000 };

    let mut down_samples = Vec::with_capacity(pair_count);
    let mut up_samples = Vec::with_capacity(pair_count);

    for i in 0..pair_count {
        let base_ms = (i + 1) * SLOT_STRIDE_MS;
        let d_start = ms_to_samples(base_ms + DOWN_OFFSET_MS);
        let d_len = ms_to_samples(DOWN_DURATION_MS);
        if d_start < pcm.len() {
            let d_end = (d_start + d_len).min(pcm.len());
            let mut slice = pcm[d_start..d_end].to_vec();
            apply_declick_envelope(&mut slice, sample_rate);
            down_samples.push(Arc::from(slice));
        }

        let u_start = ms_to_samples(base_ms + UP_OFFSET_MS);
        let u_len = ms_to_samples(UP_DURATION_MS);
        if u_start < pcm.len() {
            let u_end = (u_start + u_len).min(pcm.len());
            let mut slice = pcm[u_start..u_end].to_vec();
            apply_declick_envelope(&mut slice, sample_rate);
            up_samples.push(Arc::from(slice));
        }
    }

    // Fallback if a custom short audio file is shorter than 1000ms: treat whole clip as down & up
    if down_samples.is_empty() && !pcm.is_empty() {
        let mut slice = pcm.to_vec();
        apply_declick_envelope(&mut slice, sample_rate);
        let arc: Arc<[f32]> = Arc::from(slice);
        down_samples.push(Arc::clone(&arc));
        up_samples.push(arc);
    }

    DecodedSprite {
        down_samples,
        up_samples,
    }
}

/// Applies a tiny 1.5ms fade-in and 4ms fade-out so sprite boundaries never click or pop.
fn apply_declick_envelope(samples: &mut [f32], sample_rate: u32) {
    let len = samples.len();
    if len < 8 {
        return;
    }
    let fade_in = ((sample_rate as usize * 15) / 10_000).min(len / 4);
    let fade_out = ((sample_rate as usize * 4) / 1000).min(len / 3);

    for (i, s) in samples.iter_mut().take(fade_in).enumerate() {
        *s *= i as f32 / fade_in as f32;
    }
    for i in 0..fade_out {
        let gain = (fade_out - 1 - i) as f32 / fade_out as f32;
        samples[len - fade_out + i] *= gain;
    }
}

/// Linear interpolation resampler for offline sprite preparation.
pub fn linear_resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if input.is_empty() || from_rate == to_rate || from_rate == 0 || to_rate == 0 {
        return input.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = ((input.len() as f64) / ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = (pos - idx as f64) as f32;
        let s0 = input.get(idx).copied().unwrap_or(0.0);
        let s1 = input.get(idx + 1).copied().unwrap_or(s0);
        out.push(s0 + (s1 - s0) * frac);
    }
    out
}

fn decode_audio_to_mono(path: &Path) -> Result<(Vec<f32>, u32), SoundBankError> {
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| SoundBankError::Decode(e.to_string()))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| SoundBankError::Decode("no supported audio track".into()))?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(48_000);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| SoundBankError::Decode(e.to_string()))?;

    let mut mono = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(_) => break,
        };

        append_mono_samples(&decoded, &mut mono);
    }

    Ok((mono, sample_rate))
}

fn append_mono_samples(buf: &AudioBufferRef<'_>, out: &mut Vec<f32>) {
    match buf {
        AudioBufferRef::F32(b) => {
            let channels = b.spec().channels.count();
            let frames = b.frames();
            for f in 0..frames {
                let mut sum = 0.0f32;
                for c in 0..channels {
                    sum += b.chan(c)[f];
                }
                out.push(sum / channels.max(1) as f32);
            }
        }
        AudioBufferRef::S16(b) => {
            let channels = b.spec().channels.count();
            let frames = b.frames();
            for f in 0..frames {
                let mut sum = 0.0f32;
                for c in 0..channels {
                    sum += b.chan(c)[f] as f32 / 32768.0;
                }
                out.push(sum / channels.max(1) as f32);
            }
        }
        AudioBufferRef::S32(b) => {
            let channels = b.spec().channels.count();
            let frames = b.frames();
            for f in 0..frames {
                let mut sum = 0.0f32;
                for c in 0..channels {
                    sum += b.chan(c)[f] as f32 / 2147483648.0;
                }
                out.push(sum / channels.max(1) as f32);
            }
        }
        AudioBufferRef::U8(b) => {
            let channels = b.spec().channels.count();
            let frames = b.frames();
            for f in 0..frames {
                let mut sum = 0.0f32;
                for c in 0..channels {
                    sum += (b.chan(c)[f] as f32 - 128.0) / 128.0;
                }
                out.push(sum / channels.max(1) as f32);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_sprite_at_exact_button_sounds_timestamps() {
        let sr = 1000u32; // 1 sample = 1 ms for crystal-clear indexing
        let mut pcm = vec![0.0f32; 6000];
        // Put distinct markers at down1 (1050ms) and up1 (1300ms)
        pcm[1050] = 0.8;
        pcm[1300] = 0.6;
        pcm[2050] = 0.9;

        let sprite = slice_sprite_pcm(&pcm, sr, 4);
        assert_eq!(sprite.down_samples.len(), 4);
        assert_eq!(sprite.up_samples.len(), 4);
        assert_eq!(sprite.down_samples[0].len(), 190);
        assert_eq!(sprite.up_samples[0].len(), 790);
        assert!(sprite.down_samples[0][50] > 0.7);
        assert!(sprite.up_samples[0][100] > 0.5);
        assert!(sprite.down_samples[1][50] > 0.8);
    }

    #[test]
    fn resamples_44100_to_48000_accurately() {
        let input = vec![0.5f32; 4410];
        let out = linear_resample(&input, 44100, 48000);
        assert!((out.len() as isize - 4800).abs() <= 2);
        assert!((out[100] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn non_repeating_picker_never_repeats_consecutive() {
        let map = Mutex::new(HashMap::new());
        let mut prev = pick_non_repeating_index(5, "hi75:d", &map);
        for _ in 0..50 {
            let next = pick_non_repeating_index(5, "hi75:d", &map);
            assert_ne!(next, prev);
            prev = next;
        }
    }

    #[test]
    fn builtin_packs_include_all_55_button_sounds() {
        let packs = builtin_switch_packs();
        let catalog = sprite_catalog();
        assert_eq!(catalog.len(), 55);
        assert_eq!(packs.len(), 8 + 55);
        for s in catalog {
            assert!(
                packs.iter().any(|p| p.id == format!("sprite-{}", s.id)),
                "missing preset for sprite {}",
                s.id
            );
        }
    }

    #[test]
    fn bundled_mp3_sprites_decode_cleanly() {
        let sounds_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/sounds");
        if !sounds_dir.exists() {
            return;
        }
        let bank = SoundBank::new(sounds_dir, 48_000);
        let sprite = bank.get_or_load("hi75-soft").expect("hi75-soft should decode");
        assert_eq!(sprite.down_samples.len(), 7);
        assert_eq!(sprite.up_samples.len(), 7);
        assert!(sprite.down_samples[0].len() > 8000);
    }
}
