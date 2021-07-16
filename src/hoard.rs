use clap::{load_yaml, App};
use log::{info, warn};

use crate::{command::hoard_command::Parsable, config::load_or_build_config};

use dialoguer::{theme::ColorfulTheme, Input};

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
        let matches = App::from(yaml).get_matches();

        match matches.subcommand_name() {
            Some("new") => {
                let new_command = HoardCommand::default()
                    .with_command_string_input()
                    .with_name_input()
                    .with_description_input()
                    .with_tags_input()
                    .with_namespace_input();
                self.trove.add_command(new_command);
                self.save_trove();
            }
            // Rest of subcommands go here for now
            _ => {
                println!("Not implemented yet!")
            }
        }

        Ok(())
    }
}
