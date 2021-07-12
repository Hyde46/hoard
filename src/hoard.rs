use clap::{load_yaml, App};
use log::{info, warn};

use crate::config::load_or_build_config;

use super::command::hoard_command::HoardCommand;
use super::command::trove::CommandTrove;
use super::config::HoardConfig;

use std::{fs, path::PathBuf};
#[derive(Debug)]
pub struct Hoard {
    config: Option<HoardConfig>,
    trove: CommandTrove,
}

impl Default for Hoard {
    fn default() -> Self {
        Hoard {
            config: None,
            trove: CommandTrove::default(),
        }
    }
}
impl Hoard {
    pub fn with_config(&mut self, hoard_home_path: Option<String>) -> &mut Self {
        info!("Loading config");
        match load_or_build_config(hoard_home_path) {
            Ok(config) => self.config = Some(config),
            Err(err) => {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        };
        self
    }

    pub fn with_example_command(&mut self) -> &mut Self {
        let example_command = HoardCommand {
            name: Some(String::from("example")),
            namespace: Some(String::from("default")),
            tags: Some(vec![String::from("default"), String::from("example")]),
            command: Some(String::from("'cd ..'")),
        };
        let trove = CommandTrove {
            version: String::from("0.1.0"),
            commands: vec![example_command],
        };
        let s = serde_yaml::to_string(&trove).unwrap();
        let example_path = PathBuf::from(".trove.yml");
        fs::write(example_path.clone(), s).expect("Unable to write config file");

        let f = std::fs::File::open(example_path).unwrap();
        let loaded_trove: CommandTrove = serde_yaml::from_reader::<_, CommandTrove>(f).unwrap();
        info!("{:?}", loaded_trove);

        self
    }

    pub fn load_trove(&mut self) -> &mut Self {
        match &self.config {
            Some(config) => {
                self.trove = CommandTrove::load_trove_file(&config.trove_home_path);
            }
            None => {
                info!("[DEBUG] No command config loaded");
            }
        }
        self
    }

    pub fn save_trove(&self) {
        match &self.config {
            Some(config) => self
                .trove
                .save_trove_file(&config.trove_home_path.as_ref().unwrap()),
            None => info!("[DEBUG] No command config loaded"),
        };
    }

    pub fn start(&mut self) -> Result<(), serde_yaml::Error> {
        let yaml = load_yaml!("resources/cli.yaml");
        let _matches = App::from(yaml).get_matches();

        self.trove.add_command(HoardCommand {
            name: Some(String::from("example")),
            namespace: Some(String::from("default")),
            tags: Some(vec![String::from("default"), String::from("example")]),
            command: Some(String::from("'cd ..'")),
        });
        self.save_trove();
        Ok(())
    }
}
