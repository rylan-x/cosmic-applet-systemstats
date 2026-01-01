use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_ms: u64,

    #[serde(default)]
    pub monitors: MonitorToggles,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitorToggles {
    #[serde(default = "default_true")]
    pub cpu_usage: bool,

    #[serde(default = "default_true")]
    pub cpu_temperature: bool,

    #[serde(default = "default_true")]
    pub gpu_temperature: bool,

    #[serde(default = "default_true")]
    pub memory: bool,

    #[serde(default = "default_true")]
    pub network: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval_ms: default_refresh_interval(),
            monitors: MonitorToggles::default(),
        }
    }
}

impl Default for MonitorToggles {
    fn default() -> Self {
        Self {
            cpu_usage: true,
            cpu_temperature: true,
            gpu_temperature: true,
            memory: true,
            network: true,
        }
    }
}

fn default_refresh_interval() -> u64 {
    1000 // Milliseconds
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load config from XDG config directory or create default if it doesn't exist
    pub fn load() -> Self {
        match Self::config_path() {
            Some(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(contents) => match toml::from_str(&contents) {
                            Ok(config) => {
                                log::info!("Loaded config from {}", path.display());
                                config
                            }
                            Err(e) => {
                                log::warn!("Failed to parse config file: {}. Using defaults.", e);
                                Self::default()
                            }
                        },
                        Err(e) => {
                            log::warn!("Failed to read config file: {}. Using defaults.", e);
                            Self::default()
                        }
                    }
                } else {
                    // Create default config file
                    let default_config = Self::default();
                    if let Err(e) = Self::create_default_config(&path, &default_config) {
                        log::warn!("Failed to create default config: {}. Using in-memory defaults.", e);
                    } else {
                        log::info!("Created default config at {}", path.display());
                    }
                    default_config
                }
            }
            None => {
                log::warn!("Could not determine config directory. Using defaults.");
                Self::default()
            }
        }
    }

    /// Get the config file path following XDG Base Directory spec
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("systemstats");
            path.push("config.toml");
            path
        })
    }

    /// Create default config file
    fn create_default_config(path: &PathBuf, config: &Config) -> std::io::Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_content = format!(
r#"# SystemStats Configuration

# Refresh interval in milliseconds (default: 1000 = 1 second)
refresh_interval_ms = {}

[monitors]
# Toggle individual monitors on/off

cpu_usage = {}

cpu_temperature = {}

gpu_temperature = {}

memory = {}

network = {}
"#,
            config.refresh_interval_ms,
            config.monitors.cpu_usage,
            config.monitors.cpu_temperature,
            config.monitors.gpu_temperature,
            config.monitors.memory,
            config.monitors.network
        );

        fs::write(path, config_content)
    }
}
