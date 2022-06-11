use clap::{load_yaml, App, ArgMatches};
use log::info;
use reqwest::Url;
use url::ParseError;

use crate::config::{load_or_build_config, save_parameter_token};

use crate::command::hoard_command::HoardCommand;
use crate::command::trove::CommandTrove;
use crate::config::HoardConfig;
use crate::gui::commands_gui;
use crate::gui::prompts::prompt_multiselect_options;
use std::path::{Path, PathBuf};

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
                self.trove = CommandTrove::load_trove_file(&config.trove_path);
            }
            None => {
                info!("[DEBUG] No command config loaded");
            }
        }
        self
    }

    pub fn save_trove(&self, path: Option<&Path>) {
        match &self.config {
            Some(config) => {
                let path_to_save = path.unwrap_or_else(|| config.trove_path.as_ref().unwrap());
                self.trove.save_trove_file(path_to_save);
            }
            None => info!("[DEBUG] No command config loaded"),
        };
    }

    pub fn show_info(&self) {
        if let Some(config) = &self.config {
            if let Some(config_home_path) = &config.config_home_path {
                println!(
                    "✨ Config file is located at {}",
                    config_home_path.display()
                );
            }

            if let Some(trove_path) = &config.trove_path {
                println!("✨ Trove file is located at {}", trove_path.display());
            }
        }
    }

    pub fn set_parameter_token(&self, parameter_token: &str) {
        if let Some(config) = &self.config {
            if let Some(config_path) = &config.config_home_path {
                if !save_parameter_token(config, config_path, parameter_token) {
                    std::process::exit(1);
                }
            }
        }
    }

    fn export_command(&self, arguments: &ArgMatches) {
        if let Some(path) = arguments.value_of("path") {
            let target_path = PathBuf::from(path);
            if target_path.file_name().is_some() {
                let namespaces = self.trove.namespaces();

                let selected_namespaces = prompt_multiselect_options(
                    "Export specific namespaces?",
                    "Namespaces to export ( Space to select )",
                    &namespaces,
                    |namespace| *namespace,
                );

                if selected_namespaces.is_empty() {
                    println!("Nothing selected");
                    return;
                }

                let commands = self
                    .trove
                    .commands
                    .iter()
                    .filter(|command| selected_namespaces.contains(&command.namespace.as_str()))
                    .collect::<Vec<_>>();

                let selected_commands = prompt_multiselect_options(
                    "Export specific commands?",
                    "Commands to export ( Space to select )",
                    &commands,
                    |command| command.name.as_str(),
                );

                if selected_commands.is_empty() {
                    println!("Nothing selected");
                    return;
                }

                let mut trove_for_export = CommandTrove::default();
                for command in selected_commands {
                    trove_for_export.add_command(command.clone());
                }

                trove_for_export.save_trove_file(&target_path);
            } else {
                println!("No valid path with filename provided.");
            }
        } else {
            println!("No path provided.");
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn start(&mut self) -> (String, bool) {
        let yaml = load_yaml!("resources/cli.yaml");
        let matches = App::from(yaml).get_matches();

        let mut autocomplete_command = "".to_string();

        match matches.subcommand() {
            // Create new command and save it it in trove
            ("new", Some(_sub_m)) => {
                self.new_command();
            }
            // Fuzzy search through trove
            ("search", Some(_sub_m)) => {}
            // List all available commands
            ("list", Some(sub_m)) => {
                self.list_commands(sub_m, &mut autocomplete_command);
            }
            // Load command by name into clipboard, if available
            ("pick", Some(sub_m)) => {
                self.pick_command(sub_m);
            }
            // removes command from trove with a name supplied by input
            ("remove", Some(sub_m)) => {
                self.remove_command(sub_m);
            }
            ("remove_namespace", Some(sub_m)) => {
                self.remove_namespace(sub_m);
            }
            ("set_parameter_token", Some(sub_m)) => {
                set_parameter_token(sub_m);
            }
            // Load command by name
            ("copy", Some(_sub_m)) => {
                println!("Not yet implemented");
            }
            ("import", Some(sub_m)) => {
                self.import_trove(sub_m);
            }
            ("info", Some(_sub_m)) => {
                self.show_info();
            }
            ("export", Some(sub_m)) => {
                self.export_command(sub_m);
            }
            ("edit", Some(sub_m)) => {
                self.edit_command(sub_m);
            }
            _ => {}
        }
        (autocomplete_command, matches.is_present("autocomplete"))
    }

    fn edit_command(&mut self, sub_m: &ArgMatches) {
        if let Some(command_name) = sub_m.value_of("name") {
            println!("Editing {:?}", command_name);
            let command_to_edit = self
                .trove
                .pick_command(self.config.as_ref().unwrap(), command_name);
            match command_to_edit {
                Ok(c) => {
                    println!("{}", c.command);
                    let new_command = HoardCommand::default()
                        .with_command_string_input(
                            Some(c.command.clone()),
                            &self
                                .config
                                .as_ref()
                                .unwrap()
                                .parameter_token
                                .clone()
                                .unwrap(),
                        )
                        .with_name_input(Some(c.name.clone()), &self.trove)
                        .with_description_input(c.description.clone())
                        .with_tags_input(Some(c.tags_as_string()))
                        .with_namespace_input(Some(c.namespace));
                    self.trove.remove_command(command_name).ok();
                    self.trove.add_command(new_command);
                    self.save_trove(None);
                }
                Err(_e) => eprintln!("Could not find command {} to edit", command_name),
            }
        }
    }

    fn import_trove(&mut self, sub_m: &ArgMatches) {
        if let Some(path) = sub_m.value_of("uri") {
            match Url::parse(path) {
                Ok(url) => match reqwest_trove(url) {
                    Ok(trove_string) => {
                        let imported_trove =
                            CommandTrove::load_trove_from_string(&trove_string[..]);
                        self.trove.merge_trove(&imported_trove);
                        self.save_trove(None);
                    }
                    Err(e) => {
                        println!("Could not import trove from url: {:?}", e);
                    }
                },
                Err(err) => {
                    if let ParseError::RelativeUrlWithoutBase = err {
                        let imported_trove =
                            CommandTrove::load_trove_file(&Some(PathBuf::from(path)));
                        self.trove.merge_trove(&imported_trove);
                        self.save_trove(None);
                    } else {
                        eprintln!("Not a valid URL or file path");
                    }
                }
            }
        } else {
            println!("No arguments provided");
        }
    }

    fn remove_namespace(&mut self, sub_m: &ArgMatches) {
        if let Some(namespace) = sub_m.value_of("namespace") {
            let command_result = self.trove.remove_namespace_commands(namespace);
            match command_result {
                Ok(_) => {
                    println!("Removed all commands of namespace [{}]", namespace);
                }
                Err(e) => eprintln!("{}", e),
            }
            self.save_trove(None);
        } else {
            println!("No namespace provided!");
        }
    }

    fn remove_command(&mut self, sub_m: &ArgMatches) {
        if let Some(command_name) = sub_m.value_of("name") {
            let command_result = self.trove.remove_command(command_name);
            match command_result {
                Ok(_) => {
                    println!("Removed [{}]", command_name);
                }
                Err(e) => eprintln!("{}", e),
            }
            self.save_trove(None);
        } else {
            println!("No name provided!");
        }
    }

    fn pick_command(&mut self, sub_m: &ArgMatches) {
        if let Some(command_name) = sub_m.value_of("name") {
            let command_result = self
                .trove
                .pick_command(self.config.as_ref().unwrap(), command_name);
            match command_result {
                Ok(c) => {
                    println!("{}", c.command);
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    fn list_commands(&mut self, sub_m: &ArgMatches, autocomplete_command: &mut String) {
        if self.trove.is_empty() {
            println!("No command hoarded.\nRun [ hoard new ] first to hoard a command.");
        } else if sub_m.is_present("simple") {
            self.trove.print_trove();
        } else {
            match commands_gui::run(&mut self.trove, self.config.as_ref().unwrap()) {
                Ok(selected_command) => {
                    if let Some(c) = selected_command {
                        // Is set if a command is selected in GUI
                        if !c.command.is_empty() {
                            //TODO: If run as cli program, copy command into clipboard, else will be written to READLINE_LINE
                            *autocomplete_command = c.command;
                        }
                    }
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    }

    fn new_command(&mut self) {
        let default_namespace = self.config.as_ref().unwrap().default_namespace.clone();
        let new_command = HoardCommand::default()
            .with_command_string_input(
                None,
                &self
                    .config
                    .as_ref()
                    .unwrap()
                    .parameter_token
                    .clone()
                    .unwrap(),
            )
            .with_namespace_input(Some(default_namespace))
            .with_name_input(None, &self.trove)
            .with_description_input(None)
            .with_tags_input(None);
        self.trove.add_command(new_command);
        self.save_trove(None);
    }
}

fn set_parameter_token(sub_m: &ArgMatches) {
    sub_m.value_of("parameter_token").map_or_else(
        || println!("No parameter token provided!"),
        |parameter_token| {
            self.set_parameter_token(parameter_token);
        },
    );
}

#[tokio::main]
async fn reqwest_trove(url: Url) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.text().await?;
    Ok(resp)
}
