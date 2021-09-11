use anyhow::{anyhow, Error, Result};
use dialoguer::{theme::ColorfulTheme, Input};
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HOARD_HOMEDIR: &str = ".hoard";
const HOARD_FILE: &str = "trove.yml";
const HOARD_CONFIG: &str = "config.yml";

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize)]
pub struct HoardConfig {
    pub version: String,
    pub default_namespace: String,
    pub config_home_path: Option<PathBuf>,
    pub trove_home_path: Option<PathBuf>,
    pub query_prefix: String,
}

impl HoardConfig {
    pub fn new(hoard_home_path: &Path) -> Self {
        Self {
            version: VERSION.to_string(),
            default_namespace: String::from("default"),
            config_home_path: Some(hoard_home_path.to_path_buf()),
            trove_home_path: Some(hoard_home_path.join(HOARD_FILE)),
            query_prefix: String::from("  >"),
        }
    }

    pub fn with_default_namespace(self) -> Self {
        let default_namespace: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("This is the first time running hoard.\nChoose a default namespace where you want to hoard your commands.")
            .default(String::from("default"))
            .interact_text()
            .unwrap();
        Self {
            version: self.version,
            default_namespace,
            config_home_path: self.config_home_path,
            trove_home_path: self.trove_home_path,
            query_prefix: self.query_prefix,
        }
    }
}

/// Loads hoard config file at $HOME/.hoard/config.yml.
/// if `hoard_home_path` is set, try to read it from that custom path
///
/// If no `hoard_home_path` is found, a new config.yml will be created at the specified path
#[allow(clippy::module_name_repetitions)]
pub fn load_or_build_config(hoard_home_path: Option<String>) -> Result<HoardConfig> {
    // First check if custom path should be used
    match hoard_home_path {
        // If custom path is set
        Some(custom_path) => {
            info!("Found custom_path {:?}", custom_path);
            let path = PathBuf::from(custom_path);
            load_or_build(&path)
        }
        // If no custom path is set. Load or build config file at $HOME
        None => load_or_build_default_path(),
    }
}

fn load_or_build_default_path() -> Result<HoardConfig, Error> {
    match dirs::home_dir() {
        Some(home) => load_or_build(&home),
        None => Err(anyhow!("No $HOME directory found for hoard config")),
    }
}

fn load_or_build(path: &Path) -> Result<HoardConfig, Error> {
    info!("Loading or building in {:?}", path);
    let home_path = Path::new(&path);

    // Check if $HOME/.hoard directory exists. Create it if it does not exist
    let hoard_dir = home_path.join(HOARD_HOMEDIR);
    if !hoard_dir.exists() {
        info!("Creating {:?}", hoard_dir);
        fs::create_dir(&hoard_dir)?;
    }

    let hoard_config_path = hoard_dir.join(HOARD_CONFIG);
    info!("Hoard config path: {:?}", hoard_config_path);
    // Check if $HOME/.hoard/config.yml exists. Create default config if it does not exist
    let config = if hoard_config_path.exists() {
        info!("Config file exists");
        let f = std::fs::File::open(hoard_config_path)?;
        let mut loaded_config: HoardConfig = serde_yaml::from_reader::<_, HoardConfig>(f)?;
        if loaded_config.trove_home_path.is_none() {
            loaded_config.trove_home_path = Some(hoard_dir.join(HOARD_FILE));
        }
        Ok(loaded_config)
    } else {
        info!("Config file does not exist. Creating new one");
        let new_config = HoardConfig::new(&hoard_dir).with_default_namespace();
        let s = serde_yaml::to_string(&new_config)?;
        fs::write(hoard_config_path, s).expect("Unable to write config file");
        Ok(new_config)
    };

    config
}
