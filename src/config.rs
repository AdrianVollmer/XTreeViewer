use crate::error::{Result, XtvError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// UI settings
    #[serde(default)]
    pub ui: UiConfig,

    /// Streaming settings
    #[serde(default)]
    pub streaming: StreamingConfig,

    /// Navigation settings
    #[serde(default)]
    pub navigation: NavigationConfig,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Color theme: "dark" or "light"
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Default expanded depth (0 = collapsed, -1 = fully expanded)
    #[serde(default = "default_expanded_depth")]
    pub default_expanded_depth: i32,
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Threshold in bytes for switching to streaming mode (default: 100MB)
    #[serde(default = "default_streaming_threshold")]
    pub threshold_bytes: u64,

    /// Enable streaming mode
    #[serde(default = "default_streaming_enabled")]
    pub enabled: bool,
}

/// Navigation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationConfig {
    /// Number of lines to scroll for page up/down
    #[serde(default = "default_page_scroll_lines")]
    pub page_scroll_lines: usize,
}

// Default value functions
fn default_theme() -> String {
    "dark".to_string()
}

fn default_expanded_depth() -> i32 {
    0
}

fn default_streaming_threshold() -> u64 {
    100 * 1024 * 1024 // 100MB
}

fn default_streaming_enabled() -> bool {
    true
}

fn default_page_scroll_lines() -> usize {
    10
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            default_expanded_depth: default_expanded_depth(),
        }
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            threshold_bytes: default_streaming_threshold(),
            enabled: default_streaming_enabled(),
        }
    }
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self {
            page_scroll_lines: default_page_scroll_lines(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            streaming: StreamingConfig::default(),
            navigation: NavigationConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            XtvError::Config(format!("Failed to read config file {:?}: {}", path, e))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            XtvError::Config(format!("Failed to parse config file {:?}: {}", path, e))
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from XDG config directory (~/.config/xtv/config.toml)
    /// Falls back to default configuration if file doesn't exist
    pub fn load() -> Result<Self> {
        if let Some(config_path) = Self::xdg_config_path() {
            if config_path.exists() {
                return Self::from_file(&config_path);
            }
        }

        // Return default config if no config file exists
        Ok(Self::default())
    }

    /// Load configuration with optional custom path
    /// If custom_path is provided, load from there
    /// Otherwise, fall back to XDG config path
    pub fn load_with_custom_path(custom_path: Option<&Path>) -> Result<Self> {
        if let Some(path) = custom_path {
            return Self::from_file(path);
        }

        Self::load()
    }

    /// Get the XDG config path (~/.config/xtv/config.toml)
    pub fn xdg_config_path() -> Option<PathBuf> {
        if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
            Some(PathBuf::from(config_dir).join("xtv").join("config.toml"))
        } else if let Ok(home) = std::env::var("HOME") {
            Some(
                PathBuf::from(home)
                    .join(".config")
                    .join("xtv")
                    .join("config.toml"),
            )
        } else {
            None
        }
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate theme
        if self.ui.theme != "dark" && self.ui.theme != "light" {
            return Err(XtvError::Config(format!(
                "Invalid theme '{}'. Must be 'dark' or 'light'",
                self.ui.theme
            )));
        }

        // Validate expanded depth
        if self.ui.default_expanded_depth < -1 {
            return Err(XtvError::Config(format!(
                "Invalid default_expanded_depth {}. Must be >= -1",
                self.ui.default_expanded_depth
            )));
        }

        // Validate streaming threshold
        if self.streaming.threshold_bytes == 0 {
            return Err(XtvError::Config(
                "Invalid streaming threshold: must be > 0".to_string(),
            ));
        }

        // Validate page scroll lines
        if self.navigation.page_scroll_lines == 0 {
            return Err(XtvError::Config(
                "Invalid page_scroll_lines: must be > 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate a sample configuration file content
    pub fn sample_config() -> String {
        toml::to_string_pretty(&Self::default()).unwrap_or_else(|_| String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ui.theme, "dark");
        assert_eq!(config.ui.default_expanded_depth, 0);
        assert_eq!(config.streaming.threshold_bytes, 100 * 1024 * 1024);
        assert!(config.streaming.enabled);
        assert_eq!(config.navigation.page_scroll_lines, 10);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid theme should fail
        config.ui.theme = "invalid".to_string();
        assert!(config.validate().is_err());
        config.ui.theme = "dark".to_string();

        // Invalid expanded depth should fail
        config.ui.default_expanded_depth = -2;
        assert!(config.validate().is_err());
        config.ui.default_expanded_depth = 0;

        // Invalid streaming threshold should fail
        config.streaming.threshold_bytes = 0;
        assert!(config.validate().is_err());
        config.streaming.threshold_bytes = 100;

        // Invalid page scroll lines should fail
        config.navigation.page_scroll_lines = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sample_config() {
        let sample = Config::sample_config();
        assert!(sample.contains("theme"));
        assert!(sample.contains("threshold_bytes"));
        assert!(sample.contains("page_scroll_lines"));
    }
}
