use serde::{Deserialize, Serialize};

use dialoguer::{theme::ColorfulTheme, Input};
pub trait Parsable {
    fn parse_arguments(matches: &clap::ArgMatches) -> Self;
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
        Self {
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

    pub fn with_command_string_input(self, default_value: Option<String>) -> Self {
        let command_string: String = Input::with_theme(&ColorfulTheme::default())
            .default(default_value.unwrap_or(String::from("")))
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

    pub fn with_tags_raw(self, tags: &str) -> Self {
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: Some(
                tags.chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    .split(',')
                    .map(std::string::ToString::to_string)
                    .collect(),
            ),
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_tags_input(self, default_value: Option<String>) -> Self {
        let tags: String = Input::with_theme(&ColorfulTheme::default())
            .default(default_value.unwrap_or(String::from("")))
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
        self.with_tags_raw(&tags)
    }

    pub fn with_namespace_input(self, default_namespace: Option<String>) -> Self {
        let namespace = Input::with_theme(&ColorfulTheme::default())
            .default(default_namespace.unwrap_or(String::from("")))
            .with_prompt("Namespace of the command")
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

    pub fn with_name_input(self, default_value: Option<String>) -> Self {
        let name: String = Input::with_theme(&ColorfulTheme::default())
            .default(default_value.unwrap_or(String::from("")))
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

    pub fn with_description_input(self, default_value: Option<String>) -> Self {
        let description_string: String = Input::with_theme(&ColorfulTheme::default())
            .default(default_value.unwrap_or(String::from("")))
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
    fn parse_arguments(matches: &clap::ArgMatches) -> Self {
        let mut new_command = Self::default();

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
            new_command.tags = Some(
                tags.split(',')
                    .map(std::string::ToString::to_string)
                    .collect(),
            );
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
        let command = HoardCommand::default().with_tags_raw("foo");
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
        let command = HoardCommand::default().with_tags_raw("foo,bar");
        let expected = "foo,bar";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn parse_single_tag() {
        let command = HoardCommand::default().with_tags_raw("foo");
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
        let command = HoardCommand::default().with_tags_raw("foo,bar");
        let expected = Some(vec![String::from("foo"), String::from("bar")]);
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_whitespace_in_tags() {
        let command = HoardCommand::default().with_tags_raw("foo, bar");
        let expected = Some(vec![String::from("foo"), String::from("bar")]);
        assert_eq!(expected, command.tags);
    }
}
