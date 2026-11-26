use crate::models::{SleepStatus, SleepTriggerSettings};
use crate::platform::OsContextSnapshot;

pub fn evaluate_sleep_status(
    triggers: &SleepTriggerSettings,
    ctx: &OsContextSnapshot,
) -> SleepStatus {
    if triggers.manual_sleep {
        return SleepStatus {
            is_sleeping: true,
            reason: Some("Manual Sleep enabled".to_string()),
            active_app: ctx.active_app.clone(),
        };
    }

    if let Some(app) = &ctx.active_app {
        let app_lower = app.to_lowercase();
        for excluded in &triggers.excluded_apps {
            let ex = excluded.trim().to_lowercase();
            if !ex.is_empty() && app_lower.contains(&ex) {
                return SleepStatus {
                    is_sleeping: true,
                    reason: Some(format!("Excluded app active ({app})")),
                    active_app: ctx.active_app.clone(),
                };
            }
        }
    }

    if triggers.on_screen_sharing && ctx.is_screen_sharing {
        return SleepStatus {
            is_sleeping: true,
            reason: Some("Screen sharing / call detected".to_string()),
            active_app: ctx.active_app.clone(),
        };
    }

    if triggers.on_gaming_fullscreen && ctx.is_fullscreen {
        return SleepStatus {
            is_sleeping: true,
            reason: Some("Fullscreen / gaming app detected".to_string()),
            active_app: ctx.active_app.clone(),
        };
    }

    if triggers.on_focus_dnd && ctx.is_focus_dnd {
        return SleepStatus {
            is_sleeping: true,
            reason: Some("Focus / Do Not Disturb active".to_string()),
            active_app: ctx.active_app.clone(),
        };
    }

    if triggers.on_media_playing && ctx.is_media_playing {
        return SleepStatus {
            is_sleeping: true,
            reason: Some("Media playback detected".to_string()),
            active_app: ctx.active_app.clone(),
        };
    }

    SleepStatus {
        is_sleeping: false,
        reason: None,
        active_app: ctx.active_app.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excluded_app_triggers_sleep() {
        let triggers = SleepTriggerSettings::default();
        let ctx = OsContextSnapshot {
            active_app: Some("Discord".to_string()),
            ..Default::default()
        };
        let status = evaluate_sleep_status(&triggers, &ctx);
        assert!(status.is_sleeping);
        assert!(status.reason.unwrap().contains("Discord"));
    }
}
