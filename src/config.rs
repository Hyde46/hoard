use anyhow::{anyhow, Error, Result};
use log::info;
use serde::{Deserialize, Serialize};
use shellexpand::full;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_HOARD_HOMEDIR: &str = ".config/hoard";
const DEFAULT_HOARD_FILE: &str = "trove.yml";
const DEFAULT_HOARD_CONFIG_FILE: &str = "config.yml";
const ENV_HOARD_CONFIG_PATH: &str = "HOARD_CONFIG";

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HoardConfig {
    pub version: String,
    pub default_namespace: String,
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
            trove_path: Some(hoard_home_path.join(DEFAULT_HOARD_FILE)),
            ..Self::default()
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

/// Loads hoard config file from $HOARD_CONFIG or from $HOME/.hoard/config.yml.
/// If no config file is found, a new one will be created at the specified path
#[allow(clippy::useless_let_if_seq)]
pub fn load_or_build_config() -> Result<HoardConfig, Error> {
    let (hoard_dir, hoard_config_path) = get_hoard_config_path()
        //Split up file path and parent dir, since this function requires it.
        .and_then(|config_path| {
            config_path
                .parent()
                .ok_or_else(|| anyhow!("Config does not have a parent dir"))
                .map(|parent| (parent.to_path_buf(), config_path.clone()))
        })
        .and_then(|(parent, config_path)| {
            if parent.exists() {
                Ok((parent, config_path))
            } else {
                info!("Creating {:?}", parent);
                fs::create_dir_all(&parent)
                    .map_err(|e| anyhow!(e))
                    .map(|_| (parent, config_path))
            }
        })?;

    info!("Loading or building in {:?}", hoard_dir);

    info!("Hoard config path: {:?}", hoard_config_path);

    // Check if path/to/<config>.yml exists. Create default config at path if it does not exist
    let config = if hoard_config_path.exists() {
        info!("Config file exists");
        let f = std::fs::File::open(&hoard_config_path)?;
        let mut loaded_config: HoardConfig = serde_yaml::from_reader::<_, HoardConfig>(f)?;

        append_missing_default_values_to_config(
            &mut loaded_config,
            &hoard_dir,
            &hoard_config_path,
        )?;

        let path_buf = Path::new(DEFAULT_HOARD_FILE).to_path_buf();
        if loaded_config.read_from_current_directory.unwrap() && path_buf.exists() {
            loaded_config.trove_path = Some(path_buf);
        }
        // Sanity check. If the config makes sense
        assert!(loaded_config.parameter_token != loaded_config.parameter_ending_token, "Your parameter token {} is equal to your ending token {}. Please set one of them to another character!", loaded_config.parameter_token.as_ref().unwrap(), loaded_config.parameter_ending_token.as_ref().unwrap());
        loaded_config.trove_path = loaded_config.trove_path.and_then(|p| {
            full(p.to_str().unwrap())
                .map(|p| PathBuf::from(p.into_owned()))
                .map_err(|e| anyhow!(e))
                .ok()
        });

        Ok(loaded_config)
    } else {
        info!("Config file does not exist. Creating new one");
        let mut new_config = HoardConfig::new(&hoard_dir);
        if !cfg!(test) {
            use crate::gui::prompts::prompt_input;
            new_config.default_namespace = prompt_input(
                "This is the first time running hoard.\nChoose a default namespace where you want to hoard your commands.",
                false,
                Some(new_config.default_namespace)
                )
        }
        save_config(&new_config, &hoard_config_path)?;
        Ok(new_config)
    };

    config
}
pub fn get_hoard_config_path() -> Result<PathBuf, Error> {
    env::var(ENV_HOARD_CONFIG_PATH)
        .map_err(|err| anyhow!(err))
        .and_then(|env| {
            full(&env)
                .map(|env| env.into_owned())
                .map_err(|err| anyhow!(err))
        })
        .and_then(|e| {
            info!("HOARD_CONFIG: {e:?}");
            if e.is_empty() {
                Err(anyhow!("HOARD_CONFIG is empty"))
            } else {
                Ok(e)
            }
        })
        .map(PathBuf::from)
        // Detect if the path in HOARD_CONFIG is a file or a directory
        .and_then(|p| {
            p.extension()
                .and_then(|_| p.file_name().and_then(|f| f.to_str().to_owned()))
                .zip(p.parent())
                .map(|(file, parent)| parent.to_path_buf().join(file))
                .ok_or_else(|| anyhow!("Not a file path, but a dir path, defaulting config.yml"))
                .or_else(|_| Ok(p.join(DEFAULT_HOARD_CONFIG_FILE)))
        })
        // Use default path if HOARD_CONFIG is not set
        .or_else(|_e| {
            dirs::home_dir()
                .ok_or_else(|| anyhow!("No $HOME directory found for hoard config"))
                .map(|p| {
                    p.join(DEFAULT_HOARD_HOMEDIR)
                        .join(DEFAULT_HOARD_CONFIG_FILE)
                })
        })
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
        loaded_config.trove_path = Some(hoard_dir.join(DEFAULT_HOARD_FILE));
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
    let path_buf = config_path.join(DEFAULT_HOARD_CONFIG_FILE);

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

pub fn save_hoard_config_file(config_to_save: &HoardConfig) -> Result<(), Error> {
    let config_dir = get_hoard_config_path()?;

    save_config(config_to_save, &config_dir)
}

#[cfg(test)]
mod test_config {
    use crate::config::{get_hoard_config_path, DEFAULT_HOARD_HOMEDIR};

    use super::{
        load_or_build_config, save_parameter_token, HoardConfig, DEFAULT_HOARD_CONFIG_FILE,
    };
    use std::{env, fs::File};
    //    use rand::{thread_rng, Rng};
    use tempfile::tempdir;

    fn gen_tmp_path(file_name: Option<&str>) -> std::path::PathBuf {
        use rand::Rng;
        let random_name = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(6)
            .map(char::from)
            .collect::<String>();

        let path = env::temp_dir().join("hoard_config").join(random_name);
        file_name.map_or(path.clone(), |f| path.join(f))
    }
    #[test]
    fn test_save_parameter_token() {
        let tmp_dir = tempdir().ok().unwrap();

        // write config file.
        let tmp_path = tmp_dir.path();
        let config = HoardConfig::new(tmp_path);
        assert!(save_parameter_token(&config, tmp_path, "@"));

        // read config file, and check parameter token.
        let tmp_file = tmp_dir.path().join(DEFAULT_HOARD_CONFIG_FILE);
        let f = File::open(tmp_file).ok().unwrap();
        let parsed_config = serde_yaml::from_reader::<_, HoardConfig>(f).ok().unwrap();
        assert_eq!(parsed_config.parameter_token, Some(String::from("@")));
    }

    #[test]
    fn test_config_path_with_env() {
        let tmp_path: std::path::PathBuf = gen_tmp_path(None);
        env::set_var("HOARD_CONFIG", &tmp_path);

        let result = get_hoard_config_path().unwrap();
        assert_eq!(result, tmp_path.clone().join(DEFAULT_HOARD_CONFIG_FILE),);

        let config_name = "my_config_name.yml".to_owned();
        let tmp_path = gen_tmp_path(Some(&config_name));
        env::set_var("HOARD_CONFIG", &tmp_path);

        let result = get_hoard_config_path().unwrap();
        assert_eq!(result, tmp_path.clone());
    }

    #[test]
    fn test_config_path_with_default() {
        env::remove_var("HOARD_CONFIG");
        let result = get_hoard_config_path().unwrap();
        let file_name = result.file_name().unwrap().to_str().unwrap();
        let parent_dir = result.parent().unwrap().to_str();
        assert_eq!(file_name, DEFAULT_HOARD_CONFIG_FILE);
        assert!(parent_dir.map_or(false, |s| s.ends_with(DEFAULT_HOARD_HOMEDIR)));
    }

    #[test]
    fn test_config_building_with_env() {
        let tmp_path: std::path::PathBuf = gen_tmp_path(Some("HoardeConfig.yml"));

        env::set_var("HOARD_CONFIG", &tmp_path);
        let x = load_or_build_config().unwrap();
        let f = File::open(tmp_path).ok().unwrap();
        let parsed_config = serde_yaml::from_reader::<_, HoardConfig>(f).ok().unwrap();
        assert_eq!(parsed_config, x);

        let tmp_path: std::path::PathBuf = gen_tmp_path(None);

        env::set_var("HOARD_CONFIG", &tmp_path);
        let x = load_or_build_config().unwrap();
        let f = File::open(tmp_path.join(DEFAULT_HOARD_CONFIG_FILE))
            .ok()
            .unwrap();
        let parsed_config = serde_yaml::from_reader::<_, HoardConfig>(f).ok().unwrap();
        assert_eq!(parsed_config, x);
    }
}
