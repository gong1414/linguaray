use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::Settings;

impl Settings {
    pub fn load(file_path: impl AsRef<Path>) -> Result<Self, String> {
        let path = file_path.as_ref();
        eprintln!("[Settings::load] path: {}", path.display());
        if !path.exists() {
            eprintln!("[Settings::load] file not found, returning defaults");
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path).map_err(|error| {
            format!("failed to read settings file `{}`: {error}", path.display())
        })?;
        let settings: Self = serde_json::from_str(&content).map_err(|error| {
            format!(
                "failed to parse settings file `{}`: {error}",
                path.display()
            )
        })?;
        eprintln!(
            "[Settings::load] loaded {} providers",
            settings.providers.len()
        );
        Ok(settings)
    }

    pub fn save(&self, file_path: impl AsRef<Path>) -> Result<(), String> {
        let path = file_path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create settings directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
        let content = self.to_pretty_json()?;
        fs::write(path, content).map_err(|error| {
            format!(
                "failed to write settings file `{}`: {error}",
                path.display()
            )
        })
    }

    pub fn to_pretty_json(&self) -> Result<String, String> {
        let root = serde_json::to_value(self)
            .map_err(|error| format!("failed to encode settings: {error}"))?;
        if !root.is_object() {
            return Err("settings root must encode to a JSON object".to_owned());
        }
        serde_json::to_string_pretty(&root)
            .map_err(|error| format!("failed to render settings json: {error}"))
    }

    pub fn touch_last_updated(&mut self) -> Result<(), String> {
        self.last_updated = current_timestamp_millis()?;
        Ok(())
    }
}

fn current_timestamp_millis() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before unix epoch: {error}"))?
        .as_millis()
        .try_into()
        .map_err(|_| "current timestamp does not fit in u64".to_owned())
}
