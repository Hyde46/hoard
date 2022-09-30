use crate::cli_commands::{Cli, Commands};
use clap::Parser;
use log::info;
use reqwest::Url;
use std::path::{Path, PathBuf};
use url::ParseError;

use crate::command::hoard_command::HoardCommand;
use crate::command::trove::CommandTrove;
use crate::config::HoardConfig;
use crate::config::{load_or_build_config, save_parameter_token};
use crate::filter::query_trove;
use crate::gui::commands_gui;
use crate::gui::prompts::prompt_multiselect_options;

#[derive(Default, Debug)]
pub struct Hoard {
    config: Option<HoardConfig>,
    trove: CommandTrove,
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

    pub fn start(&mut self) -> (String, bool) {
        let mut autocomplete_command = "".to_string();
        let cli = Cli::parse();

        match &cli.command {
            Commands::Info {} => {
                self.show_info();
            }
            Commands::New {
                name,
                tags,
                command,
                description,
            } => {
                self.new_command(
                    name.clone(),
                    tags.clone(),
                    command.clone(),
                    description.clone(),
                );
            }
            Commands::List {
                filter,
                json,
                simple,
            } => {
                let commands =
                    self.list_commands(simple.to_owned(), json.to_owned(), filter.clone());
                if let Some(c) = commands {
                    autocomplete_command = c;
                }
            }
            Commands::Pick { name } => {
                self.pick_command(name);
            }
            Commands::Remove { name } => {
                self.remove_command(name);
            }
            Commands::RemoveNamespace { namespace } => {
                self.remove_namespace(namespace);
            }
            Commands::SetParameterToken { name } => {
                self.set_parameter_token(name);
            }
            Commands::Import { uri } => {
                self.import_trove(uri);
            }
            Commands::Export { path } => {
                self.export_command(path);
            }
            Commands::Edit { name } => {
                self.edit_command(name);
            }
            Commands::ShellConfig { shell } => {
                Self::shell_config_command(shell);
            }
        }

        (autocomplete_command, cli.autocomplete)
    }

    pub fn show_info(&self) {
        // Print out path to hoard config file and path to where the trove file is stored
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

    fn new_command(
        &mut self,
        name: Option<String>,
        tags: Option<String>,
        command: Option<String>,
        description: Option<String>,
    ) {
        let default_namespace = self.config.as_ref().unwrap().default_namespace.clone();
        let new_command = HoardCommand::default()
            .with_command_string_input(
                command,
                &self
                    .config
                    .as_ref()
                    .unwrap()
                    .parameter_token
                    .clone()
                    .unwrap(),
            )
            .with_namespace_input(Some(default_namespace))
            .with_name_input(name, &self.trove)
            .with_description_input(description)
            .with_tags_input(tags);
        self.trove.add_command(new_command);
        self.save_trove(None);
    }

    fn list_commands(
        &mut self,
        is_simple: bool,
        is_structured: bool,
        filter: Option<String>,
    ) -> Option<String> {
        if self.trove.is_empty() {
            println!("No command hoarded.\nRun [ hoard new ] first to hoard a command.");
        } else if is_simple {
            self.trove.print_trove();
        } else if is_structured {
            // Return list of commands in json format, filtered by `filter`
            let query_string: String = filter.unwrap_or_default();
            let filtered_trove = query_trove(&self.trove, &query_string);
            return Some(filtered_trove.to_yaml());
        } else {
            match commands_gui::run(&mut self.trove, self.config.as_ref().unwrap()) {
                Ok(selected_command) => {
                    if let Some(c) = selected_command {
                        // Is set if a command is selected in GUI
                        if !c.command.is_empty() {
                            //TODO: If run as cli program, copy command into clipboard, else will be written to READLINE_LINE
                            return Some(c.command);
                        }
                    }
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
        None
    }

    fn pick_command(&mut self, name: &str) {
        let command_result = self.trove.pick_command(self.config.as_ref().unwrap(), name);
        match command_result {
            Ok(c) => {
                println!("{}", c.command);
            }
            Err(e) => eprintln!("{}", e),
        }
    }

    fn remove_command(&mut self, command_name: &str) {
        let command_result = self.trove.remove_command(command_name);
        match command_result {
            Ok(_) => {
                println!("Removed [{}]", command_name);
            }
            Err(e) => eprintln!("{}", e),
        }
        self.save_trove(None);
    }

    fn remove_namespace(&mut self, namespace: &str) {
        let command_result = self.trove.remove_namespace_commands(namespace);
        match command_result {
            Ok(_) => {
                println!("Removed all commands of namespace [{}]", namespace);
            }
            Err(e) => eprintln!("{}", e),
        }
        self.save_trove(None);
    }

    fn import_trove(&mut self, path: &str) {
        match Url::parse(path) {
            Ok(url) => match reqwest_trove(url) {
                Ok(trove_string) => {
                    let imported_trove = CommandTrove::load_trove_from_string(&trove_string[..]);
                    self.trove.merge_trove(&imported_trove);
                    self.save_trove(None);
                }
                Err(e) => {
                    println!("Could not import trove from url: {:?}", e);
                }
            },
            Err(err) => {
                if err == ParseError::RelativeUrlWithoutBase {
                    let imported_trove = CommandTrove::load_trove_file(&Some(PathBuf::from(path)));
                    self.trove.merge_trove(&imported_trove);
                    self.save_trove(None);
                } else {
                    eprintln!("Not a valid URL or file path");
                }
            }
        }
    }

    fn export_command(&self, path: &str) {
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

    fn edit_command(&mut self, command_name: &str) {
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

    fn shell_config_command(shell: &str) {
        let src = match shell {
            "bash" => include_str!("shell/hoard.bash"),
            "fish" => include_str!("shell/hoard.fish"),
            "zsh" => include_str!("shell/hoard.zsh"),
            s => {
                println!("Unknown shell '{}'!\nMust be either bash, fish or zsh!", s);
                return;
            }
        };
        print!("{}", src);
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
}

#[tokio::main]
async fn reqwest_trove(url: Url) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.text().await?;
    Ok(resp)
}
