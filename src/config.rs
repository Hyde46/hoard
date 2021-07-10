use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const HOARD_HOMEDIR: &'static str = ".hoard";
const HOARD_FILE: &'static str = "trove.yml";
const HOARD_CONFIG: &'static str = "config.yml";

#[derive(Debug, Serialize, Deserialize)]
pub struct HoardConfig {
    pub version: String,
    pub config_home_path: PathBuf,
    pub trove_home_path: PathBuf,
}

impl HoardConfig {
    pub fn new(hoard_home_path: PathBuf) -> HoardConfig {
        HoardConfig {
            version: VERSION.to_string(),
            config_home_path: hoard_home_path.clone(),
            trove_home_path: hoard_home_path.join(HOARD_FILE),
        }
    }
}

/// Loads hoard config file at $HOME/.hoard/config.yml.
/// if `hoard_home_path` is set, try to read it from that custom path
///
/// If no hoard_home_path is found, a new config.yml will be created at the specified path
pub fn load_or_build_config(hoard_home_path: Option<String>) -> Result<HoardConfig> {
    // First check if custom path should be used
    match hoard_home_path {
        // If custom path is set
        Some(custom_path) => {
            let path = PathBuf::from(custom_path);
            load_or_build(path)
        }
        // If no custom path is set. Load or build config file at $HOME
        None => load_or_build_default_path(),
    }
}

fn load_or_build_default_path() -> Result<HoardConfig, Error> {
    match dirs::home_dir() {
        Some(home) => load_or_build(home),
        None => Err(anyhow!("No $HOME directory found for hoard config")),
    }
}

fn load_or_build(path: PathBuf) -> Result<HoardConfig, Error> {
    let home_path = Path::new(&path);

    // Check if $HOME/.hoard directory exists. Create it if it does not exist
    let hoard_dir = home_path.join(HOARD_HOMEDIR);
    if !hoard_dir.exists() {
        fs::create_dir(&hoard_dir)?;
    }

    let hoard_config_path = hoard_dir.join(HOARD_CONFIG);

    // Check if $HOME/.hoard/config.yml exists. Create default config if it does not exist
    let config = if !hoard_config_path.exists() {
        let new_config = HoardConfig::new(hoard_dir);
        let s = serde_yaml::to_string(&new_config)?;
        fs::write(hoard_config_path, s).expect("Unable to write config file");
        Ok(new_config)
    } else {
        let f = std::fs::File::open(hoard_config_path)?;
        let loaded_config: HoardConfig = serde_yaml::from_reader::<_, HoardConfig>(f)?;
        Ok(loaded_config)
    };

    config
}
