use crate::cli_commands::{Cli, Commands};
use clap::Parser;
use log::info;
use reqwest::{StatusCode, Url};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::ParseError;

use crate::cli_commands::Mode;
use crate::command::hoard_command::HoardCommand;
use crate::command::trove::CommandTrove;
use crate::config::{compare_with_latest_version, HoardConfig};
use crate::config::{load_or_build_config, save_hoard_config_file, save_parameter_token};
use crate::filter::query_trove;
use crate::gui::commands_gui;
use crate::gui::prompts::{
    prompt_input, prompt_multiselect_options, prompt_password, prompt_password_repeat,
    prompt_yes_or_no, Confirmation,
};
use crate::sync_models::TokenResponse;
use crate::util::rem_first_and_last;
use base64::encode;

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
                eprintln!("ERROR: {err}");
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {cause}"));
                std::process::exit(1);
            }
        };
        self
    }

    pub async fn start(&mut self) -> (String, bool) {
        if !compare_with_latest_version().await.0 {
            println!(
                "Running hoard a newer Version {} Available at https://github.com/Hyde46/hoard \nPlease update.", compare_with_latest_version().await.1
            );
        }
        let mut autocomplete_command = String::new();
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
            Commands::Sync { command } => {
                self.sync(*command);
            }
        }

        (autocomplete_command, cli.autocomplete)
    }

    pub fn show_info(&self) {
        // Print out path to hoard config file and path to where the trove file is stored
        if let Some(config) = &self.config {
            if let Some(config_home_path) = &config.config_home_path {
                println!(
                    "🔧 Config file is located at {}",
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
                &self
                    .config
                    .as_ref()
                    .unwrap()
                    .parameter_ending_token
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
                    self.save_trove(None);
                    if let Some(c) = selected_command {
                        // Is set if a command is selected in GUI
                        if !c.command.is_empty() {
                            //TODO: If run as cli program, copy command into clipboard, else will be written to READLINE_LINE
                            return Some(c.command);
                        }
                    }
                }
                Err(e) => {
                    println!("{e}");
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
            Err(e) => eprintln!("{e}"),
        }
    }

    fn remove_command(&mut self, command_name: &str) {
        let command_result = self.trove.remove_command(command_name);
        match command_result {
            Ok(_) => {
                println!("Removed [{command_name}]");
            }
            Err(e) => eprintln!("{e}"),
        }
        self.save_trove(None);
    }

    fn remove_namespace(&mut self, namespace: &str) {
        let command_result = self.trove.remove_namespace_commands(namespace);
        match command_result {
            Ok(_) => {
                println!("Removed all commands of namespace [{namespace}]");
            }
            Err(e) => eprintln!("{e}"),
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
                    println!("Could not import trove from url: {e}");
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
        println!("Editing {command_name}");
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
                        &self
                            .config
                            .as_ref()
                            .unwrap()
                            .parameter_ending_token
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
            Err(_e) => eprintln!("Could not find command {command_name} to edit"),
        }
    }

    fn shell_config_command(shell: &str) {
        let src = match shell {
            "bash" => include_str!("shell/hoard.bash"),
            "fish" => include_str!("shell/hoard.fish"),
            "zsh" => include_str!("shell/hoard.zsh"),
            s => {
                println!("Unknown shell '{s}'!\nMust be either bash, fish or zsh!");
                return;
            }
        };
        print!("{src}");
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
        self.config.as_ref().map_or_else(
            || info!("[DEBUG] No command config loaded"),
            |config| {
                let path_to_save = path.unwrap_or_else(|| config.trove_path.as_ref().unwrap());
                self.trove.save_trove_file(path_to_save);
            },
        );
    }

    fn save_backup_trove(&self, path: Option<&Path>) {
        self.config.as_ref().map_or_else(
            || info!("[DEBUG] No command config loaded"),
            |config| {
                let backup_trove_path_str = format!(
                    "{}.bk",
                    config.trove_path.as_ref().unwrap().to_str().unwrap()
                );
                let backup_trove_path = PathBuf::from_str(&backup_trove_path_str).ok().unwrap();
                let path_to_save = path.unwrap_or(&backup_trove_path);
                self.trove.save_trove_file(path_to_save);
            },
        );
    }

    fn revert_trove(&self) {
        self.config.as_ref().map_or_else(|| info!("[DEBUG] No command config loaded"), |config| {
            let trove_path = config.trove_path.as_ref().unwrap();
            let backup_trove_path_str = format!("{}.bk", trove_path.to_str().unwrap());
            let backup_trove_path = PathBuf::from_str(&backup_trove_path_str).ok().unwrap();
            if backup_trove_path.exists() {
                if let Confirmation::Yes = prompt_yes_or_no("Found a backup from just before the last time you ran `hoard sync`. Are you sure you want to revert to this state?") {
                    let e = fs::remove_file(trove_path);
                    // make clippy happy
                    drop(e);
                    fs::rename(backup_trove_path_str, trove_path).unwrap();
                    println!("Done!");
                } else {
                    println!("Keeping current trove file...");
                }
            }
        });
    }

    fn register_user(&mut self) {
        println!("Registering account..");
        let user_email = prompt_input("Email: ", false, None);
        let user_pw: String = prompt_password_repeat("Password: ");
        let client = reqwest::blocking::Client::new();
        let register_url = format!(
            "{}register",
            self.config.clone().unwrap().sync_server_url.unwrap()
        );
        let register_body = format!("{{\"password\": \"{user_pw}\",\"email\": \"{user_email}\"}}");
        let body = client
            .post(register_url)
            .body(register_body)
            .header("Content-Type", "application/json")
            .send()
            .unwrap();
        if body.status() == StatusCode::CREATED {
            println!("Created new user! Verification not needed for now. Run `hoard sync login` next.\n\nPlease consider supporting further development and help offset server costs here:\nbuy.stripe.com/9AQ9Bm6Nx4qb6YwaEE\nThis is the only time this message will pop up :)");
        } else {
            println!("Something went all wrong. Try another email.");
        }
    }

    fn login(&mut self) {
        println!("Logging in..");
        let user_email = prompt_input("Email: ", false, None);
        let user_pw: String = prompt_password("Password: ");
        let register_body = format!("{{\"password\": \"{user_pw}\",\"email\": \"{user_email}\"}}");
        let client = reqwest::blocking::Client::new();
        let register_url = format!(
            "{}token/new",
            self.config.clone().unwrap().sync_server_url.unwrap()
        );
        let body = client
            .get(register_url)
            .body(register_body)
            .header("Content-Type", "application/json")
            .send()
            .unwrap();
        if body.status() == StatusCode::CREATED {
            let response_text = body.text().unwrap();
            let token = serde_yaml::from_str::<TokenResponse>(&response_text).unwrap();
            let mut config = self.config.clone().unwrap();
            let b64_token = encode(token.token);
            config.api_token = Some(b64_token);
            save_hoard_config_file(&config, &config.clone().config_home_path.unwrap()).unwrap();
            println!("Success!");
        } else {
            println!("Invalid Email and password combination.");
        }
    }

    fn get_trove_file(&self) -> Option<CommandTrove> {
        println!("Syncing ...");
        let token = self.config.clone().unwrap().api_token;
        let client = reqwest::blocking::Client::new();
        let save_url = format!(
            "{}v1/trove",
            self.config.clone().unwrap().sync_server_url.unwrap()
        );
        let body = client
            .get(save_url)
            .bearer_auth(token.unwrap())
            .header("Content-Type", "text/plain")
            .send()
            .unwrap();
        if body.status() == 200 {
            // Replaced escaped new line char with byte code for linebreak
            let escaped_string = body.text().unwrap().replace("\\n", "\x0A");
            // Replace escaped " with unescaped version
            let unescaped_string = escaped_string.replace("\\\"", "\"");
            return Some(CommandTrove::load_trove_from_string(rem_first_and_last(
                &unescaped_string,
            )));
        }
        None
    }

    fn sync_safe(&self) {
        println!("Uploading trove...");
        let token = self.config.clone().unwrap().api_token;
        let client = reqwest::blocking::Client::new();
        let save_url = format!(
            "{}v1/trove",
            self.config.clone().unwrap().sync_server_url.unwrap()
        );
        let trove_file =
            fs::read_to_string(self.config.clone().unwrap().trove_path.unwrap()).unwrap();
        let body = client
            .put(save_url)
            .body(trove_file)
            .bearer_auth(token.unwrap())
            .header("Content-Type", "text/plain")
            .send()
            .unwrap();
        if body.status() == 201 {
            println!("Done!");
        } else {
            println!("Could not save trove. Is it a valid trove file?");
            println!("{}", body.text().unwrap());
        }
    }

    pub fn sync(&mut self, command: Mode) {
        // Check if user is logged in
        // Else inform the user to run `hoard sync login` first and break
        match command {
            Mode::Register => self.register_user(),
            Mode::Login => {
                if self.is_logged_in() {
                    println!("You are already logged in.");
                    return;
                }
                self.login();
            }
            Mode::Logout => {
                println!("Logging out..");
                let mut config = self.config.clone().unwrap();
                config.api_token = None;
                save_hoard_config_file(&config, &config.clone().config_home_path.unwrap()).unwrap();
            }
            Mode::Save => {
                if !self.is_logged_in() {
                    println!("Please log in [hoard sync login] or register an account [hoard sync register] to use the sync feature!");
                    return;
                }
                self.sync_safe();
            }
            Mode::Get => {
                // `hoard sync` is run
                // Pull trove
                if !self.is_logged_in() {
                    println!("Please log in [hoard sync login] or register an account [hoard sync register] to use the sync feature!");
                    return;
                }
                let trove = self.get_trove_file();
                if let Some(t) = trove {
                    // Prepare backup trove to enable reverting if merge goes all wrong, or user incorrectly removes commands they wanted to keep
                    self.save_backup_trove(None);
                    let was_dirty = self.trove.merge_trove(&t);
                    if was_dirty {
                        self.save_trove(None);
                        println!("All done!");
                        return;
                    }
                    println!("No changes");
                } else {
                    println!("Could not fetch trove file from your account!");
                }
            }
            Mode::Revert => {
                self.revert_trove();
            }
        }
    }

    fn is_logged_in(&self) -> bool {
        if let Some(config) = self.config.clone() {
            if config.api_token.is_none() {
                return false;
            }
        }
        true
    }
}

#[tokio::main]
async fn reqwest_trove(url: Url) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.text().await?;
    Ok(resp)
}
