//! Pure application policy. No GPUI, platform APIs, threads, or filesystem access.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub schema_version: u32,
    pub workspace_name: String,
    pub compact: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            workspace_name: "My workspace".into(),
            compact: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("Unsupported settings version; use a compatible app.".into());
        }
        if self.workspace_name.trim().is_empty() || self.workspace_name.chars().count() > 64 {
            return Err("Workspace name must contain 1–64 characters.".into());
        }
        if self.workspace_name.chars().any(char::is_control) {
            return Err("Workspace name cannot contain control characters.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RuntimeState {
    pub last_result: Option<JobResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobResult {
    pub samples: u64,
    pub checksum: u64,
}

/// A reproducible CPU workload, chunked so cancellation has a bounded latency.
pub fn calculate_chunk(start: u64, end: u64) -> u64 {
    (start..end).fold(0_u64, |sum, n| {
        sum.wrapping_add(n.wrapping_mul(n).rotate_left((n % 63) as u32))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_unicode_names_and_rejects_future_schema() {
        let mut s = Settings {
            workspace_name: "研究 🦀".into(),
            ..Settings::default()
        };
        assert!(s.validate().is_ok());
        s.workspace_name = " ".into();
        assert!(s.validate().is_err());
        s.workspace_name = "valid".into();
        s.schema_version = 2;
        assert!(s.validate().is_err());
    }
    #[test]
    fn name_limits_count_unicode_characters_and_reject_control_characters() {
        let mut settings = Settings {
            workspace_name: "🦀".repeat(64),
            ..Settings::default()
        };
        assert!(settings.validate().is_ok());
        settings.workspace_name.push('🦀');
        assert!(settings.validate().is_err());
        for name in ["line\nbreak", "null\0byte", "tab\there", "\r"] {
            settings.workspace_name = name.into();
            assert!(settings.validate().is_err());
        }
    }
    #[test]
    fn chunks_have_the_same_result_as_one_pass() {
        assert_eq!(
            calculate_chunk(0, 100),
            calculate_chunk(0, 30).wrapping_add(calculate_chunk(30, 100))
        );
    }
}
