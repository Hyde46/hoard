use serde::{Deserialize, Serialize};

use dialoguer::{theme::ColorfulTheme, Input};
pub trait Parsable {
    fn parse_arguments(matches: &&clap::ArgMatches) -> Self;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoardCommand {
    pub name: String,
    pub namespace: String,
    pub tags: Option<Vec<String>>,
    pub command: String,
    pub description: Option<String>,
}

impl HoardCommand {
    pub fn default() -> Self {
        HoardCommand {
            name: String::from(""),
            namespace: String::from(""),
            tags: None,
            command: String::from(""),
            description: None,
        }
    }
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        if self.name.is_empty()
            || self.namespace.is_empty()
            || self.tags.is_none()
            || self.command.is_empty()
            || self.description.is_none()
        {
            return false;
        }
        true
    }

    pub fn tags_as_string(&self) -> String {
        self.tags
            .as_ref()
            .unwrap_or(&vec![String::from("")])
            .join(",")
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
            command: command_string,
            description: self.description,
        }
    }

    pub fn with_tags_raw(self, tags: String) -> Self {
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: Some(
                tags.chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
            ),
            command: self.command,
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
        self.with_tags_raw(tags)
    }

    pub fn with_namespace_input(self) -> Self {
        let namespace: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Namespace of the command")
            .with_initial_text("default".to_string())
            .interact_text()
            .unwrap();
        Self {
            name: self.name,
            namespace,
            tags: self.tags,
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_name_input(self) -> Self {
        let name: String = Input::with_theme(&ColorfulTheme::default())
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
            name,
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
            new_command.name = n.to_string();
        }
        // Defaults to 'default' namespace
        if let Some(ns) = matches.value_of("namespace") {
            // TODO: some validation for when we have it
            new_command.namespace = ns.to_string();
        }
        // "$ hoard test -t" was run
        // Expects comma seperated tags
        if let Some(tags) = matches.value_of("tags") {
            new_command.tags = Some(tags.split(',').map(|s| s.to_string()).collect());
        }
        if let Some(c) = matches.value_of("command") {
            // TODO: some validation for when we have it
            new_command.command = c.to_string();
        }
        new_command
    }
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn one_tag_as_string() {
        let command = HoardCommand::default().with_tags_raw(String::from("foo"));
        let expected = "foo";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn no_tag_as_string() {
        let command = HoardCommand::default();
        let expected = "";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn multiple_tags_as_string() {
        let command = HoardCommand::default().with_tags_raw(String::from("foo,bar"));
        let expected = "foo,bar";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn parse_single_tag() {
        let command = HoardCommand::default().with_tags_raw(String::from("foo"));
        let expected = Some(vec![String::from("foo")]);
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_no_tag() {
        let command = HoardCommand::default();
        let expected = None;
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_multiple_tags() {
        let command = HoardCommand::default().with_tags_raw(String::from("foo,bar"));
        let expected = Some(vec![String::from("foo"), String::from("bar")]);
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_whitespace_in_tags() {
        let command = HoardCommand::default().with_tags_raw(String::from("foo, bar"));
        let expected = Some(vec![String::from("foo"), String::from("bar")]);
        assert_eq!(expected, command.tags);
    }
}
