use clap::{load_yaml, App};

use crate::config::load_or_build_config;

use super::config::HoardConfig;

#[derive(Debug)]
pub struct Hoard {
    config: Option<HoardConfig>,
}

impl Hoard {
    pub fn new() -> Self {
        Hoard { config: None }
    }

    pub fn with_config(&mut self, hoard_home_path: Option<String>) -> &mut Self {
        let _config = load_or_build_config(hoard_home_path);
        //self.config = Some(config);
        self
    }

    pub fn start(&self) -> Result<(), serde_yaml::Error> {
        let yaml = load_yaml!("resources/cli.yaml");
        let _matches = App::from(yaml).get_matches();

        Ok(())
    }
}
