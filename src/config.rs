use crate::control::prompts::prompt_input;
use anyhow::{anyhow, Error, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HOARD_HOMEDIR: &str = ".config/hoard";
const HOARD_FILE: &str = "trove.yml";
pub const HOARD_CONFIG: &str = "config.yml";

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoardConfig {
    pub version: String,
    pub default_namespace: String,
    pub config_home_path: Option<PathBuf>,
    pub trove_path: Option<PathBuf>,
    pub query_prefix: String,
    // Color settings
    pub primary_color: Option<(u8, u8, u8)>,
    pub secondary_color: Option<(u8, u8, u8)>,
    pub tertiary_color: Option<(u8, u8, u8)>,
    pub command_color: Option<(u8, u8, u8)>,
    // Parameter settings
    pub parameter_token: Option<String>,
    // Token to indicate the end of a named parameter
    pub parameter_ending_token: Option<String>,
    pub read_from_current_directory: Option<bool>,
    // URL to trove sync server
    pub sync_server_url: Option<String>,
    pub api_token: Option<String>,
    pub gpt_api_key: Option<String>,
}

impl Default for HoardConfig {
    fn default() -> Self {
        Self {
            version: VERSION.to_string(),
            default_namespace: "default".to_string(),
            config_home_path: None,
            trove_path: None,
            query_prefix: "  >".to_string(),
            primary_color: Some(Self::default_colors(0)),
            secondary_color: Some(Self::default_colors(1)),
            tertiary_color: Some(Self::default_colors(2)),
            command_color: Some(Self::default_colors(3)),
            parameter_token: Some(Self::default_parameter_token()),
            parameter_ending_token: Some(Self::default_ending_parameter_token()),
            read_from_current_directory: Some(Self::default_read_from_current_directory()),
            sync_server_url: Some(Self::default_sync_server_url()),
            api_token: None,
            gpt_api_key: None,
        }
    }
}

impl HoardConfig {
    pub fn new(hoard_home_path: &Path) -> Self {
        Self {
            version: VERSION.to_string(),
            default_namespace: "default".to_string(),
            config_home_path: Some(hoard_home_path.to_path_buf()),
            trove_path: Some(hoard_home_path.join(HOARD_FILE)),
            query_prefix: "  >".to_string(),
            primary_color: Some(Self::default_colors(0)),
            secondary_color: Some(Self::default_colors(1)),
            tertiary_color: Some(Self::default_colors(2)),
            command_color: Some(Self::default_colors(3)),
            parameter_token: Some(Self::default_parameter_token()),
            parameter_ending_token: Some(Self::default_ending_parameter_token()),
            read_from_current_directory: Some(Self::default_read_from_current_directory()),
            sync_server_url: Some(Self::default_sync_server_url()),
            api_token: None,
            gpt_api_key: None,
        }
    }

    pub fn with_default_namespace(self) -> Self {
        let default_namespace = prompt_input(
            "This is the first time running hoard.\nChoose a default namespace where you want to hoard your commands.",
            false,
            Some("default".to_string())
        );
        Self {
            version: self.version,
            default_namespace,
            config_home_path: self.config_home_path,
            trove_path: self.trove_path,
            query_prefix: self.query_prefix,
            primary_color: self.primary_color,
            secondary_color: self.secondary_color,
            tertiary_color: self.tertiary_color,
            command_color: self.command_color,
            parameter_token: self.parameter_token,
            parameter_ending_token: self.parameter_ending_token,
            read_from_current_directory: self.read_from_current_directory,
            sync_server_url: self.sync_server_url,
            api_token: self.api_token,
            gpt_api_key: self.gpt_api_key,
        }
    }

    fn default_parameter_token() -> String {
        "#".to_string()
    }

    fn default_ending_parameter_token() -> String {
        "!".to_string()
    }

    fn default_sync_server_url() -> String {
        "https://troveserver.herokuapp.com/".to_string()
    }

    const fn default_read_from_current_directory() -> bool {
        true
    }

    const fn default_colors(color_level: u8) -> (u8, u8, u8) {
        match color_level {
            0 => (242, 229, 188),
            1 => (181, 118, 20),
            2 => (50, 48, 47),
            _ => (180, 118, 20),
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
    hoard_home_path.map_or_else(load_or_build_default_path, |custom_path| {
        info!("Found custom_path {:?}", custom_path);
        let path = PathBuf::from(custom_path);
        load_or_build(&path)
    })
}

fn load_or_build_default_path() -> Result<HoardConfig, Error> {
    dirs::home_dir().map_or_else(
        || Err(anyhow!("No $HOME directory found for hoard config")),
        |home| load_or_build(&home),
    )
}

#[allow(clippy::useless_let_if_seq)]
fn load_or_build(path: &Path) -> Result<HoardConfig, Error> {
    info!("Loading or building in {:?}", path);
    let home_path = Path::new(&path);

    // Check if $HOME/.hoard directory exists. Create it if it does not exist
    let hoard_dir = home_path.join(HOARD_HOMEDIR);
    if !hoard_dir.exists() {
        info!("Creating {:?}", hoard_dir);
        fs::create_dir_all(&hoard_dir)?;
    }

    let hoard_config_path = hoard_dir.join(HOARD_CONFIG);
    info!("Hoard config path: {:?}", hoard_config_path);
    // Check if $HOME/.hoard/config.yml exists. Create default config if it does not exist
    let config = if hoard_config_path.exists() {
        info!("Config file exists");
        let f = std::fs::File::open(&hoard_config_path)?;
        let mut loaded_config: HoardConfig = serde_yaml::from_reader::<_, HoardConfig>(f)?;

        append_missing_default_values_to_config(
            &mut loaded_config,
            &hoard_dir,
            &hoard_config_path,
        )?;

        let path_buf = Path::new(HOARD_FILE).to_path_buf();
        if loaded_config.read_from_current_directory.unwrap() && path_buf.exists() {
            loaded_config.trove_path = Some(path_buf);
        }
        // Sanity check. If the config makes sense
        assert!(loaded_config.parameter_token != loaded_config.parameter_ending_token, "Your parameter token {} is equal to your ending token {}. Please set one of them to another character!", loaded_config.parameter_token.as_ref().unwrap(), loaded_config.parameter_ending_token.as_ref().unwrap());

        Ok(loaded_config)
    } else {
        info!("Config file does not exist. Creating new one");
        let new_config = HoardConfig::new(&hoard_dir).with_default_namespace();
        save_config(&new_config, &hoard_config_path)?;
        Ok(new_config)
    };

    config
}

fn append_missing_default_values_to_config(
    loaded_config: &mut HoardConfig,
    hoard_dir: &Path,
    hoard_config_path: &Path,
) -> Result<(), Error> {
    // Adds configuration fields and sets the values to their default value if they are missing.
    // Mostly for legacy configuration support when new configuration options are added
    // If any of the defaults are loaded and set, save the hoard configuration to disk
    let is_config_dirty = if loaded_config.primary_color.is_none() {
        loaded_config.primary_color = Some(HoardConfig::default_colors(0));
        true
    } else if loaded_config.secondary_color.is_none() {
        loaded_config.secondary_color = Some(HoardConfig::default_colors(1));
        true
    } else if loaded_config.tertiary_color.is_none() {
        loaded_config.tertiary_color = Some(HoardConfig::default_colors(2));
        true
    } else if loaded_config.command_color.is_none() {
        loaded_config.command_color = Some(HoardConfig::default_colors(3));
        true
    } else if loaded_config.trove_path.is_none() {
        loaded_config.trove_path = Some(hoard_dir.join(HOARD_FILE));
        true
    } else if loaded_config.parameter_token.is_none() {
        loaded_config.parameter_token = Some(HoardConfig::default_parameter_token());
        true
    } else if loaded_config.parameter_ending_token.is_none() {
        loaded_config.parameter_ending_token = Some(HoardConfig::default_ending_parameter_token());
        true
    } else if loaded_config.read_from_current_directory.is_none() {
        loaded_config.read_from_current_directory = Some(false);
        true
    } else if loaded_config.sync_server_url.is_none() {
        loaded_config.sync_server_url = Some(HoardConfig::default_sync_server_url());
        true
    } else {
        false
    };

    if is_config_dirty {
        save_config(&*loaded_config, hoard_config_path)?;
    }
    Ok(())
}

pub fn save_parameter_token(
    config: &HoardConfig,
    config_path: &Path,
    parameter_token: &str,
) -> bool {
    let mut new_config = config.clone();
    let path_buf = config_path.join(HOARD_CONFIG);
    new_config.parameter_token = Some(String::from(parameter_token));
    match save_config(&new_config, path_buf.as_path()) {
        Ok(()) => true,
        Err(err) => {
            eprintln!("ERROR: {err}");
            err.chain()
                .skip(1)
                .for_each(|cause| eprintln!("because: {cause}"));
            false
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ClientResponse {
    pub tag_name: String,
}

// pub async fn compare_with_latest_version() -> (bool, String) {
//     let client = reqwest::Client::builder()
//         .user_agent(env!("CARGO_PKG_NAME"))
//         .build()
//         .unwrap();
//     if let Ok(client_response) = client
//         .get("https://api.github.com/repos/Hyde46/hoard/releases/latest")
//         .send()
//         .await
//     {
//         if let Ok(release) = client_response.json::<ClientResponse>().await {
//             let tag_name = release.tag_name;
//             if !tag_name.is_empty() {
//                 return (VERSION == &tag_name[1..], tag_name);
//             }
//         }
//     }
//     (true, String::new())
// }

fn save_config(config_to_save: &HoardConfig, config_path: &Path) -> Result<(), Error> {
    let s = serde_yaml::to_string(&config_to_save)?;
    fs::write(config_path, s).expect("Unable to write config file");
    Ok(())
}

pub fn save_hoard_config_file(config_to_save: &HoardConfig, base_path: &Path) -> Result<(), Error> {
    let config_dir = base_path.join(HOARD_CONFIG);
    save_config(config_to_save, &config_dir)
}

#[cfg(test)]
mod test_config {
    use super::{save_parameter_token, HoardConfig, HOARD_CONFIG};
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_save_parameter_token() {
        let tmp_dir = tempdir().ok().unwrap();

        // write config file.
        let tmp_path = tmp_dir.path();
        let config = HoardConfig::new(tmp_path);
        assert!(save_parameter_token(&config, tmp_path, "@"));

        // read config file, and check parameter token.
        let tmp_file = tmp_dir.path().join(HOARD_CONFIG);
        let f = File::open(tmp_file).ok().unwrap();
        let parsed_config = serde_yaml::from_reader::<_, HoardConfig>(f).ok().unwrap();
        assert_eq!(parsed_config.parameter_token, Some(String::from("@")));
    }
}
