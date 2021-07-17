use serde::{Deserialize, Serialize};

use dialoguer::{theme::ColorfulTheme, Input};
pub trait Parsable {
    fn parse_arguments(matches: &&clap::ArgMatches) -> Self;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HoardCommand {
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub tags: Option<Vec<String>>,
    pub command: Option<String>,
    pub description: Option<String>,
}

impl HoardCommand {
    pub fn default() -> Self {
        HoardCommand {
            name: None,
            namespace: None,
            tags: None,
            command: None,
            description: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        if self.name.is_none()
            || self.namespace.is_none()
            || self.tags.is_none()
            || self.command.is_none()
            || self.description.is_none()
        {
            return false;
        }
        true
    }

    pub fn with_command_string_input(self) -> Self {
        let command_string: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Command to hoard")
            .interact_text()
            .unwrap();
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: Some(command_string),
            description: self.description,
        }
    }

    pub fn with_tags_input(self) -> Self {
        let tags: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Give your command tags ( comma separated )")
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
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: Some(tags.split(',').map(|s| s.to_string()).collect()),
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_namespace_input(self) -> Self {
        let namespace: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Namespace of the command")
            .with_initial_text("default".to_string())
            .interact_text()
            .unwrap();
        Self {
            name: self.name,
            namespace: Some(namespace),
            tags: self.tags,
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_name_input(self) -> Self {
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
        Self {
            name: Some(command_name),
            namespace: self.namespace,
            tags: self.tags,
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_description_input(self) -> Self {
        let description_string: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Describe what the command does")
            .interact_text()
            .unwrap();
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: self.command,
            description: Some(description_string),
        }
    }
}

impl Parsable for HoardCommand {
    fn parse_arguments(matches: &&clap::ArgMatches) -> HoardCommand {
        let mut new_command = HoardCommand::default();

        if let Some(n) = matches.value_of("name") {
            // TODO: some name validation for when we have it
            new_command.name = Some(n.to_string());
        }
        // Defaults to 'default' namespace
        if let Some(ns) = matches.value_of("namespace") {
            // TODO: some validation for when we have it
            new_command.namespace = Some(ns.to_string());
        }
        // "$ hoard test -t" was run
        // Expects comma seperated tags
        if let Some(tags) = matches.value_of("tags") {
            new_command.tags = Some(tags.split(',').map(|s| s.to_string()).collect());
        }
        if let Some(c) = matches.value_of("command") {
            // TODO: some validation for when we have it
            new_command.command = Some(c.to_string());
        }
        new_command
    }
}
