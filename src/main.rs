mod command;

use std::fs;

use clap::{load_yaml, App};
use dialoguer::{theme::ColorfulTheme, Input};

use command::{command::Command, new_command::NewCommand};

extern crate serde;
extern crate serde_yaml;

fn main() -> Result<(), serde_yaml::Error> {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("resources/cli.yaml");
    let matches = App::from(yaml).get_matches();

    if let Some(ref matches) = matches.subcommand_matches("new") {
        // "$ hoard new" was run
        let mut new_command = NewCommand::new(matches);

        // TODO:
        // Move this to separate module
        if !new_command.is_complete() {
            println!("Start interactive dialogue to fillout missing arguments");
            if new_command.command.is_none() {
                let command_string: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Command to hoard")
                    .interact_text()
                    .unwrap();
                new_command.command = Some(command_string);
            }
            if new_command.name.is_none() {
                let command_name: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Name your command")
                    .validate_with({
                        move |input: &String| -> Result<(), &str> {
                            if input.contains(' ') {
                                Err("The name cant contain whitespaces")
                            } else {
                                Ok(())
                            }
                        }
                    })
                    .interact_text()
                    .unwrap();
                new_command.name = Some(command_name);
            }
            if new_command.tags.is_none() {
                let tags: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Give your command tags ( commar separated )")
                    .validate_with({
                        move |input: &String| -> Result<(), &str> {
                            if input.contains(' ') {
                                Err("Tags cant contain whitespaces")
                            } else {
                                Ok(())
                            }
                        }
                    })
                    .interact_text()
                    .unwrap();
                new_command.tags = Some(tags.split(',').map(|s| s.to_string()).collect());
            }
            if new_command.namespace.is_none() {
                let namespace: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Namespace of the command")
                    .with_initial_text("default".to_string())
                    .interact_text()
                    .unwrap();
                new_command.namespace = Some(namespace);
            }
        }
        println!("command should be complete, save it now");

        // TODO: Only testing saving yaml files!
        // Does not support multiple commands
        // Nor does it append to old files
        // Just testing the easy yaml creation
        let s = serde_yaml::to_string(&new_command)?;
        println!("{:?}", s);
        fs::write("./.hoard.yaml", s).expect("Unable to write file");
    }
    Ok(())
}
