use clap::{load_yaml, App};
use log::info;

use crate::config::load_or_build_config;

use super::command::hoard_command::HoardCommand;
use super::command::trove::CommandTrove;
use super::config::HoardConfig;
use super::gui::commands_gui;

#[derive(Debug)]
pub struct Hoard {
    config: Option<HoardConfig>,
    trove: CommandTrove,
}

impl Default for Hoard {
    fn default() -> Self {
        Self {
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
                .save_trove_file(config.trove_home_path.as_ref().unwrap()),
            None => info!("[DEBUG] No command config loaded"),
        };
    }

    pub async fn start(&mut self) -> Result<(String, bool), serde_yaml::Error> {
        let yaml = load_yaml!("resources/cli.yaml");
        let matches = App::from(yaml).get_matches();
        let default_namespace = self.config.as_ref().unwrap().default_namespace.clone();

        let mut autocomplete_command = String::from("");

        match matches.subcommand() {
            // Create new command and save it it in trove
            ("new", Some(_sub_m)) => {
                let new_command = HoardCommand::default()
                    .with_command_string_input()
                    .with_name_input()
                    .with_description_input()
                    .with_tags_input()
                    .with_namespace_input(default_namespace);
                self.trove.add_command(new_command);
                self.save_trove();
            }
            // Fuzzy search through trove
            ("search", Some(_sub_m)) => {}
            // List all available commands
            ("list", Some(sub_m)) => {
                if self.trove.is_empty() {
                    println!("No command hoarded.\nRun [ hoard new ] first to hoard a command.");
                } else if sub_m.is_present("simple") {
                    self.trove.print_trove();
                } else {
                    match commands_gui::run(&mut self.trove, self.config.as_ref().unwrap()) {
                        Ok(selected_command) => {
                            // Is set if a command is selected in GUI
                            if !selected_command.is_empty() {
                                //TODO: If run as cli program, copy command into clipboard, else will be written to READLINE_LINE
                                autocomplete_command = selected_command;
                            }
                        }
                        Err(error) => {
                            println!("{}", error);
                        }
                    }
                }
            }
            // Load command by name into clipboard, if available
            ("pick", Some(sub_m)) => {
                if let Some(command_name) = sub_m.value_of("name") {
                    let command_result = self.trove.pick_command(command_name.into());
                    match command_result {
                        Ok(c) => {
                            println!("{}", c.command);
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                }
            }
            // removes command from trove with a name supplied by input
            ("remove", Some(sub_m)) => {
                if let Some(command_name) = sub_m.value_of("name") {
                    let command_result = self.trove.remove_command(command_name.into());
                    match command_result {
                        Ok(_) => {
                            println!("Removed [{}]", command_name);
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                    self.save_trove();
                }
            }
            // Load command by name
            ("copy", Some(_sub_m)) => {
                println!("Not yet implemented");
            }
            _ => {}
        }

        Ok((autocomplete_command, matches.is_present("autocomplete")))
    }
}
