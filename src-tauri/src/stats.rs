use crate::models::{KeyZone, TypingStats};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const WPM_WINDOW_SECS: u64 = 15;

pub struct StatsTracker {
    stats: Mutex<TypingStats>,
    recent_keys: Mutex<VecDeque<Instant>>,
}

impl StatsTracker {
    pub fn new(initial: TypingStats) -> Self {
        Self {
            stats: Mutex::new(initial),
            recent_keys: Mutex::new(VecDeque::with_capacity(256)),
        }
    }

    fn rollover_if_needed(stats: &mut TypingStats) {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if stats.today_date != today {
            stats.today_date = today;
            stats.keystrokes_today = 0;
            stats.mouse_clicks_today = 0;
            stats.scroll_ticks_today = 0;
            stats.current_wpm = 0;
        }
    }

    pub fn record_keystroke(&self, zone: KeyZone, switch_id: &str) {
        let now = Instant::now();
        let wpm = {
            let mut q = self.recent_keys.lock();
            q.push_back(now);
            let cutoff = now - Duration::from_secs(WPM_WINDOW_SECS);
            while q.front().is_some_and(|&t| t < cutoff) {
                q.pop_front();
            }
            // Standard WPM = (chars / 5) * (60 / window_secs)
            ((q.len() as f32 / 5.0) * (60.0 / WPM_WINDOW_SECS as f32)).round() as u32
        };

        let mut stats = self.stats.lock();
        Self::rollover_if_needed(&mut stats);
        stats.keystrokes_today += 1;
        stats.keystrokes_all_time += 1;
        stats.current_wpm = wpm;
        if wpm > stats.peak_wpm {
            stats.peak_wpm = wpm;
        }
        let zone_key = format!("{:?}", zone).to_lowercase();
        *stats.zone_counts.entry(zone_key).or_insert(0) += 1;
        *stats.switch_usage.entry(switch_id.to_string()).or_insert(0) += 1;
    }

    pub fn record_mouse_click(&self) {
        let mut stats = self.stats.lock();
        Self::rollover_if_needed(&mut stats);
        stats.mouse_clicks_today += 1;
        stats.mouse_clicks_all_time += 1;
    }

    pub fn record_scroll_tick(&self) {
        let mut stats = self.stats.lock();
        Self::rollover_if_needed(&mut stats);
        stats.scroll_ticks_today += 1;
        stats.scroll_ticks_all_time += 1;
    }

    pub fn snapshot(&self) -> TypingStats {
        let now = Instant::now();
        let current_wpm = {
            let mut q = self.recent_keys.lock();
            let cutoff = now - Duration::from_secs(WPM_WINDOW_SECS);
            while q.front().is_some_and(|&t| t < cutoff) {
                q.pop_front();
            }
            ((q.len() as f32 / 5.0) * (60.0 / WPM_WINDOW_SECS as f32)).round() as u32
        };
        let mut stats = self.stats.lock();
        Self::rollover_if_needed(&mut stats);
        stats.current_wpm = current_wpm;
        stats.clone()
    }

    pub fn reset(&self) -> TypingStats {
        self.recent_keys.lock().clear();
        let fresh = TypingStats::default();
        *self.stats.lock() = fresh.clone();
        fresh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_keystrokes_and_computes_wpm() {
        let tracker = StatsTracker::new(TypingStats::default());
        for _ in 0..25 {
            tracker.record_keystroke(KeyZone::Alpha, "leobog-hi75");
        }
        tracker.record_mouse_click();
        tracker.record_scroll_tick();

        let snap = tracker.snapshot();
        assert_eq!(snap.keystrokes_today, 25);
        assert_eq!(snap.keystrokes_all_time, 25);
        assert_eq!(snap.mouse_clicks_today, 1);
        assert_eq!(snap.scroll_ticks_today, 1);
        // 25 keys / 5 = 5 words in 15s = 20 WPM
        assert_eq!(snap.current_wpm, 20);
        assert_eq!(snap.peak_wpm, 20);
    }
}
