use clap::{load_yaml, App};
use log::info;

use crate::config::load_or_build_config;

use super::command::hoard_command::HoardCommand;
use super::command::trove::CommandTrove;
use super::config::HoardConfig;
use super::gui::commands_gui;
use std::path::PathBuf;

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

    pub fn start(&mut self) -> (String, bool) {
        let yaml = load_yaml!("resources/cli.yaml");
        let matches = App::from(yaml).get_matches();

        let mut autocomplete_command = String::from("");

        match matches.subcommand() {
            // Create new command and save it it in trove
            ("new", Some(_sub_m)) => {
                let default_namespace = self.config.as_ref().unwrap().default_namespace.clone();
                let new_command = HoardCommand::default()
                    .with_command_string_input(None)
                    .with_namespace_input(Some(default_namespace))
                    .with_name_input(None, &self.trove)
                    .with_description_input(None)
                    .with_tags_input(None);
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
                    let command_result = self.trove.pick_command(command_name);
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
                    let command_result = self.trove.remove_command(command_name);
                    match command_result {
                        Ok(_) => {
                            println!("Removed [{}]", command_name);
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                    self.save_trove();
                } else if let Some(namespace) = sub_m.value_of("namespace") {
                    let command_result = self.trove.remove_namespace_commands(namespace);
                    match command_result {
                        Ok(_) => {
                            println!("Removed all commands of namespace [{}]", namespace);
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                    self.save_trove();
                } else {
                    println!("No arguments provided!");
                }
            }
            // Load command by name
            ("copy", Some(_sub_m)) => {
                println!("Not yet implemented");
            }
            ("import", Some(sub_m)) => {
                //TODO: At somepoint make distinction based on whats being supplied
                // import by URL
                if let Some(url_string) = sub_m.value_of("url") {
                    match reqwest_trove(url_string) {
                        Ok(trove_string) => {
                            let imported_trove =
                                CommandTrove::load_trove_from_string(&trove_string[..]);
                            self.trove.merge_trove(imported_trove);
                            self.save_trove();
                        }
                        Err(e) => {
                            println!("Could not import trove from url: {:?}", e);
                        }
                    }
                }
                // import by file
                if let Some(file_path) = sub_m.value_of("file") {
                    //TODO If <name,namespace> has a conflict, ask for a new namespace or name
                    let imported_trove =
                        CommandTrove::load_trove_file(&Some(PathBuf::from(file_path)));
                    self.trove.merge_trove(imported_trove);
                    self.save_trove();
                }
            }
            ("edit", Some(sub_m)) => {
                if let Some(command_name) = sub_m.value_of("name") {
                    println!("Editing {:?}", command_name);
                    let command_to_edit = self.trove.pick_command(command_name);
                    match command_to_edit {
                        Ok(c) => {
                            println!("{}", c.command);
                            let new_command = HoardCommand::default()
                                .with_command_string_input(Some(c.command.clone()))
                                .with_name_input(Some(c.name.clone()), &self.trove)
                                .with_description_input(c.description.clone())
                                .with_tags_input(Some(c.tags_as_string()))
                                .with_namespace_input(Some(c.namespace));
                            self.trove.remove_command(command_name).ok();
                            self.trove.add_command(new_command);
                            self.save_trove();
                        }
                        Err(_e) => eprintln!("Could not find command {} to edit", command_name),
                    }
                }
            }
            _ => {}
        }
        (autocomplete_command, matches.is_present("autocomplete"))
    }
}

#[tokio::main]
async fn reqwest_trove(url_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url_path).await?.text().await?;
    Ok(resp)
}
