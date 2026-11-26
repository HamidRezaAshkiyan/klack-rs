use crate::models::{AudioDeviceInfo, TonePadSettings};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use thiserror::Error;

const MAX_VOICES: usize = 32;
const CMD_QUEUE_CAPACITY: usize = 128;
const MAX_ITD_SAMPLES: usize = 24; // ~0.5ms at 48kHz

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("no audio output device available")]
    NoOutputDevice,
    #[error("audio stream error: {0}")]
    Stream(String),
}

#[derive(Debug, Clone)]
pub struct VoiceCmd {
    pub sample: Arc<[f32]>,
    pub gain: f32,
    pub pitch_ratio: f32,
    /// -1.0 (hard left) .. +1.0 (hard right)
    pub pan: f32,
    /// Interaural time delay in samples (positive = right delayed, negative = left delayed)
    pub itd_samples: i32,
    /// Tone Pad snapshot at trigger time
    pub tone_x: f32,
    pub tone_y: f32,
}

struct ActiveVoice {
    sample: Arc<[f32]>,
    pos: f64,
    pitch_ratio: f64,
    gain_l: f32,
    gain_r: f32,
    delay_l: usize,
    delay_r: usize,
    elapsed: usize,
    /// 1-pole tilt filter state for Tone Pad X (warm ↔ crisp)
    lp_state: f32,
    lp_alpha: f32,
    tone_x: f32,
    /// Envelope decay rate from Tone Pad Y (damped ↔ punchy)
    decay_per_sample: f32,
    env: f32,
}

/// Pure, lock-free real-time mixer state (also directly unit-testable offline!).
pub struct RealtimeMixer {
    voices: Vec<ActiveVoice>,
    sample_rate: f32,
}

impl RealtimeMixer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            voices: Vec::with_capacity(MAX_VOICES),
            sample_rate: sample_rate.max(8000) as f32,
        }
    }

    pub fn trigger(&mut self, cmd: VoiceCmd) {
        if cmd.sample.is_empty() || cmd.gain <= 0.0001 {
            return;
        }
        // Steal oldest voice if full
        if self.voices.len() >= MAX_VOICES {
            self.voices.remove(0);
        }

        // Constant-power stereo pan: pan in [-1.0, 1.0] -> angle in [0, PI/2]
        let pan = cmd.pan.clamp(-1.0, 1.0);
        let angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
        let gain_l = cmd.gain * angle.cos();
        let gain_r = cmd.gain * angle.sin();

        let clamped_itd = cmd
            .itd_samples
            .clamp(-(MAX_ITD_SAMPLES as i32), MAX_ITD_SAMPLES as i32);
        let (delay_l, delay_r) = if clamped_itd > 0 {
            (0usize, clamped_itd as usize)
        } else {
            ((-clamped_itd) as usize, 0usize)
        };

        // Tone Pad X: pitch nudge (±14%) + 1-pole lowpass/high-shelf tilt
        let tone_x = cmd.tone_x.clamp(-1.0, 1.0);
        let tone_y = cmd.tone_y.clamp(-1.0, 1.0);
        let tone_pitch = 1.0 + tone_x * 0.14;
        let effective_pitch = (cmd.pitch_ratio * tone_pitch).clamp(0.45, 2.2) as f64;

        // Cutoff frequency for warmth/brightness tilt (~2.6kHz pivot)
        let cutoff = 2600.0f32;
        let dt = 1.0 / self.sample_rate;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let lp_alpha = dt / (rc + dt);

        // Tone Pad Y: negative = faster damping (soft/muted), positive = punchier transient
        let decay_per_sample = if tone_y < 0.0 {
            let damp_ms = 220.0 + (1.0 + tone_y) * 500.0; // down to ~220ms
            (-1.0 / (damp_ms * 0.001 * self.sample_rate)).exp()
        } else {
            1.0
        };
        let punch_boost = 1.0 + tone_y.max(0.0) * 0.28;

        self.voices.push(ActiveVoice {
            sample: cmd.sample,
            pos: 0.0,
            pitch_ratio: effective_pitch,
            gain_l: gain_l * punch_boost,
            gain_r: gain_r * punch_boost,
            delay_l,
            delay_r,
            elapsed: 0,
            lp_state: 0.0,
            lp_alpha,
            tone_x,
            decay_per_sample,
            env: 1.0,
        });
    }

    /// Renders `frames` interleaved stereo (or N-channel) f32 samples into `out`.
    pub fn render(&mut self, out: &mut [f32], channels: usize, master_volume: f32) {
        let ch = channels.max(1);
        let frames = out.len() / ch;

        for frame_idx in 0..frames {
            let mut mix_l = 0.0f32;
            let mut mix_r = 0.0f32;

            let mut i = 0;
            while i < self.voices.len() {
                let v = &mut self.voices[i];
                let sample_len = v.sample.len();

                let raw = {
                    let idx = v.pos as usize;
                    if idx >= sample_len {
                        0.0
                    } else {
                        let frac = (v.pos - idx as f64) as f32;
                        let s0 = v.sample[idx];
                        let s1 = v.sample.get(idx + 1).copied().unwrap_or(0.0);
                        s0 + (s1 - s0) * frac
                    }
                };

                // Tone Pad tilt filter (warm low-pass vs crisp high-emphasis)
                v.lp_state += v.lp_alpha * (raw - v.lp_state);
                let high = raw - v.lp_state;
                let shaped = if v.tone_x < 0.0 {
                    // Blend toward warm low-passed signal
                    let amt = -v.tone_x * 0.72;
                    raw * (1.0 - amt) + v.lp_state * amt
                } else {
                    // Boost high-frequency click/clack presence
                    raw + high * (v.tone_x * 0.55)
                };

                let val = shaped * v.env;
                v.env *= v.decay_per_sample;

                if v.elapsed >= v.delay_l {
                    mix_l += val * v.gain_l;
                }
                if v.elapsed >= v.delay_r {
                    mix_r += val * v.gain_r;
                }

                v.elapsed += 1;
                if v.elapsed > v.delay_l.min(v.delay_r) {
                    v.pos += v.pitch_ratio;
                }

                let finished = (v.pos as usize) >= sample_len
                    && v.elapsed >= v.delay_l.max(v.delay_r) + 2;
                if finished {
                    self.voices.swap_remove(i);
                } else {
                    i += 1;
                }
            }

            // Master volume + gentle soft-clipping limiter so rapid chords never distort harshly
            let l = (mix_l * master_volume).tanh();
            let r = (mix_r * master_volume).tanh();

            let base = frame_idx * ch;
            if ch == 1 {
                out[base] = (l + r) * 0.5;
            } else {
                out[base] = l;
                out[base + 1] = r;
                for extra in 2..ch {
                    out[base + extra] = 0.0;
                }
            }
        }
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.len()
    }
}

struct StreamHandle {
    _stream: Stream,
}

// cpal::Stream is not Send on some platforms; we guard it strictly behind a Mutex and only swap on control calls.
unsafe impl Send for StreamHandle {}

pub struct AudioEngine {
    tx: Mutex<Sender<VoiceCmd>>,
    stream: Mutex<Option<StreamHandle>>,
    master_volume_bits: Arc<AtomicU32>,
    sample_rate: Arc<AtomicU32>,
}

impl AudioEngine {
    pub fn new(initial_volume: f32, preferred_device: Option<&str>) -> Self {
        let (tx, rx) = bounded::<VoiceCmd>(CMD_QUEUE_CAPACITY);
        let master_volume_bits = Arc::new(AtomicU32::new(initial_volume.clamp(0.0, 1.5).to_bits()));
        let sample_rate = Arc::new(AtomicU32::new(48_000));

        let engine = Self {
            tx: Mutex::new(tx),
            stream: Mutex::new(None),
            master_volume_bits,
            sample_rate,
        };

        if let Err(err) = engine.open_stream_with_rx(preferred_device, rx) {
            tracing::warn!("audio output stream could not start immediately: {}", err);
        }

        engine
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, volume: f32) {
        self.master_volume_bits
            .store(volume.clamp(0.0, 1.5).to_bits(), Ordering::Relaxed);
    }

    pub fn play(&self, cmd: VoiceCmd) {
        let tx = self.tx.lock();
        let _ = tx.try_send(cmd);
    }

    pub fn clone_sender(&self) -> Sender<VoiceCmd> {
        self.tx.lock().clone()
    }

    pub fn switch_output_device(&self, preferred_device: Option<&str>) -> Result<u32, AudioError> {
        let (new_tx, new_rx) = bounded::<VoiceCmd>(CMD_QUEUE_CAPACITY);
        let sr = self.open_stream_with_rx(preferred_device, new_rx)?;
        *self.tx.lock() = new_tx;
        Ok(sr)
    }

    fn open_stream_with_rx(
        &self,
        preferred_device: Option<&str>,
        rx: Receiver<VoiceCmd>,
    ) -> Result<u32, AudioError> {
        let host = cpal::default_host();
        let device = if let Some(name) = preferred_device {
            host.output_devices()
                .ok()
                .and_then(|mut devs| devs.find(|d| d.name().ok().as_deref() == Some(name)))
                .or_else(|| host.default_output_device())
        } else {
            host.default_output_device()
        }
        .ok_or(AudioError::NoOutputDevice)?;

        let supported = device
            .default_output_config()
            .map_err(|e| AudioError::Stream(e.to_string()))?;
        let sr = supported.sample_rate().0;
        let channels = supported.channels() as usize;
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.into();

        self.sample_rate.store(sr, Ordering::Relaxed);
        let vol_bits = Arc::clone(&self.master_volume_bits);

        let err_fn = |err| tracing::error!("cpal output stream error: {}", err);

        let stream = match sample_format {
            SampleFormat::F32 => {
                let mut mixer = RealtimeMixer::new(sr);
                device.build_output_stream(
                    &config,
                    move |data: &mut [f32], _| {
                        while let Ok(cmd) = rx.try_recv() {
                            mixer.trigger(cmd);
                        }
                        let vol = f32::from_bits(vol_bits.load(Ordering::Relaxed));
                        mixer.render(data, channels, vol);
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::I16 => {
                let mut mixer = RealtimeMixer::new(sr);
                let mut scratch = Vec::new();
                device.build_output_stream(
                    &config,
                    move |data: &mut [i16], _| {
                        while let Ok(cmd) = rx.try_recv() {
                            mixer.trigger(cmd);
                        }
                        scratch.resize(data.len(), 0.0);
                        let vol = f32::from_bits(vol_bits.load(Ordering::Relaxed));
                        mixer.render(&mut scratch, channels, vol);
                        for (dst, &src) in data.iter_mut().zip(scratch.iter()) {
                            *dst = (src.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                let mut mixer = RealtimeMixer::new(sr);
                let mut scratch = Vec::new();
                device.build_output_stream(
                    &config,
                    move |data: &mut [u16], _| {
                        while let Ok(cmd) = rx.try_recv() {
                            mixer.trigger(cmd);
                        }
                        scratch.resize(data.len(), 0.0);
                        let vol = f32::from_bits(vol_bits.load(Ordering::Relaxed));
                        mixer.render(&mut scratch, channels, vol);
                        for (dst, &src) in data.iter_mut().zip(scratch.iter()) {
                            *dst = ((src.clamp(-1.0, 1.0) * 0.5 + 0.5) * u16::MAX as f32) as u16;
                        }
                    },
                    err_fn,
                    None,
                )
            }
            other => {
                return Err(AudioError::Stream(format!(
                    "unsupported sample format: {:?}",
                    other
                )));
            }
        }
        .map_err(|e| AudioError::Stream(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioError::Stream(e.to_string()))?;

        *self.stream.lock() = Some(StreamHandle { _stream: stream });

        Ok(sr)
    }

    pub fn list_output_devices() -> Vec<AudioDeviceInfo> {
        let host = cpal::default_host();
        let default_name = host.default_output_device().and_then(|d| d.name().ok());
        let mut out = Vec::new();
        if let Ok(devices) = host.output_devices() {
            for dev in devices {
                if let Ok(name) = dev.name() {
                    let is_default = default_name.as_deref() == Some(name.as_str());
                    out.push(AudioDeviceInfo { name, is_default });
                }
            }
        }
        out
    }

    pub fn compute_itd_samples(pan: f32, sample_rate: u32, itd_enabled: bool) -> i32 {
        if !itd_enabled {
            return 0;
        }
        // Up to 0.25ms ITD at hard pan
        let max_delay = (sample_rate as f32 * 0.00025).round();
        (pan.clamp(-1.0, 1.0) * max_delay) as i32
    }

    pub fn build_voice_cmd(
        sample: Arc<[f32]>,
        gain: f32,
        base_pitch: f32,
        pitch_randomization: f32,
        pan: f32,
        itd_enabled: bool,
        sample_rate: u32,
        tone_pad: &TonePadSettings,
    ) -> VoiceCmd {
        let jitter = if pitch_randomization > 0.0 {
            (fastrand::f32() * 2.0 - 1.0) * pitch_randomization
        } else {
            0.0
        };
        let gain_jitter = 1.0 + (fastrand::f32() * 2.0 - 1.0) * 0.04;
        VoiceCmd {
            sample,
            gain: (gain * gain_jitter).max(0.0),
            pitch_ratio: (base_pitch * (1.0 + jitter)).clamp(0.5, 2.0),
            pan,
            itd_samples: Self::compute_itd_samples(pan, sample_rate, itd_enabled),
            tone_x: tone_pad.x,
            tone_y: tone_pad.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_pan_places_left_key_louder_in_left_channel() {
        let mut mixer = RealtimeMixer::new(48_000);
        let sample: Arc<[f32]> = Arc::from(vec![0.5f32; 256]);

        mixer.trigger(VoiceCmd {
            sample,
            gain: 0.8,
            pitch_ratio: 1.0,
            pan: -0.75,
            itd_samples: 0,
            tone_x: 0.0,
            tone_y: 0.0,
        });

        let mut buf = vec![0.0f32; 512]; // 256 stereo frames
        mixer.render(&mut buf, 2, 1.0);

        let energy_l: f32 = buf.iter().step_by(2).map(|x| x * x).sum();
        let energy_r: f32 = buf.iter().skip(1).step_by(2).map(|x| x * x).sum();
        assert!(energy_l > energy_r * 3.0, "L={energy_l} should dominate R={energy_r}");
    }

    #[test]
    fn tone_pad_and_polyphony_never_produce_nans_or_clip() {
        let mut mixer = RealtimeMixer::new(48_000);
        let impulse: Arc<[f32]> = Arc::from(
            (0..512)
                .map(|i| (i as f32 * 0.12).sin() * 0.9)
                .collect::<Vec<_>>(),
        );
        for k in 0..40 {
            mixer.trigger(VoiceCmd {
                sample: Arc::clone(&impulse),
                gain: 1.0,
                pitch_ratio: 0.95 + (k % 5) as f32 * 0.02,
                pan: -0.6 + (k % 10) as f32 * 0.12,
                itd_samples: 6,
                tone_x: 0.8,
                tone_y: 0.7,
            });
        }
        assert!(mixer.active_voice_count() <= MAX_VOICES);
        let mut out = vec![0.0f32; 1024];
        mixer.render(&mut out, 2, 1.2);
        assert!(out.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
    }
}
