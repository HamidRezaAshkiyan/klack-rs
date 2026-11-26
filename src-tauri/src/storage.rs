use crate::models::{AppConfig, TypingStats};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&data_dir);
        Self { data_dir }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    fn config_path(&self) -> PathBuf {
        self.data_dir.join("config.json")
    }

    fn stats_path(&self) -> PathBuf {
        self.data_dir.join("stats.json")
    }

    pub fn load_config(&self) -> AppConfig {
        let path = self.config_path();
        match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => AppConfig::default(),
        }
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<(), String> {
        let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(self.config_path(), json).map_err(|e| e.to_string())
    }

    pub fn load_stats(&self) -> TypingStats {
        let path = self.stats_path();
        match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => TypingStats::default(),
        }
    }

    pub fn save_stats(&self, stats: &TypingStats) -> Result<(), String> {
        let json = serde_json::to_string_pretty(stats).map_err(|e| e.to_string())?;
        fs::write(self.stats_path(), json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn config_and_stats_roundtrip() {
        let dir = tempdir().unwrap();
        let store = Storage::new(dir.path().to_path_buf());

        let mut cfg = AppConfig::default();
        cfg.volume = 0.63;
        cfg.active_switch_id = "nuphy-halo".into();
        cfg.tone_pad.x = 0.4;
        store.save_config(&cfg).unwrap();

        let loaded = store.load_config();
        assert!((loaded.volume - 0.63).abs() < 1e-5);
        assert_eq!(loaded.active_switch_id, "nuphy-halo");
        assert!((loaded.tone_pad.x - 0.4).abs() < 1e-5);
    }
}
