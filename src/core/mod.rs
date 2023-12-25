pub mod error;
pub mod parameters;
pub mod trove;

use crate::core::error::HoardErr;
use crate::core::trove::Trove;
use crate::gui::merge::{with_conflict_resolve_prompt, ConflictResolve};
use crate::gui::prompts::{prompt_input, prompt_input_validate, prompt_select_with_options};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time;

fn default_time() -> time::SystemTime {
    time::SystemTime::now()
}

/// Storage for the saved command structure
///
/// A `HoardCmd` can store the following parameters
/// - `name`: The name of the command by which it is referenced
/// - `command`: The terminal command to be stored and executed
/// - `description`: A description of the command for the user
/// - `tags`: A list of tags to be used for searching
/// - `created`: The date and time the command was created
/// - `modified`: The date and time the command was last modified
/// - `last_used`: The date and time the command was last used
/// - `usage_count`: The number of times the command has been used
/// - `is_favorite`: A flag to indicate if the command is a favorite
/// - `is_hidden`: A flag to indicate if the command is hidden
/// - `is_deleted`: A flag to indicate if the command is deleted
/// - `namespace`: The namespace the command belongs to
/// - `namespace_id`: The id of the namespace the command belongs to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoardCmd {
    /// The name of the command by which it is referenced
    pub name: String,

    /// The terminal command to be stored and executed
    pub command: String,

    /// A description of the command for the user
    pub description: String,

    /// A list of tags to be used for searching
    pub tags: Vec<String>,

    /// The date and time the command was created
    #[serde(default = "default_time")]
    pub created: time::SystemTime,

    /// The date and time the command was last modified
    #[serde(default = "default_time")]
    pub modified: time::SystemTime,

    /// The date and time the command was last used
    #[serde(default = "default_time")]
    pub last_used: time::SystemTime,

    /// The number of times the command has been used
    #[serde(default)]
    pub usage_count: usize,

    /// A flag to indicate if the command is a favorite
    #[serde(default)]
    pub is_favorite: bool,

    /// A flag to indicate if the command is hidden
    #[serde(default)]
    pub is_hidden: bool,

    /// A flag to indicate if the command is deleted
    #[serde(default)]
    pub is_deleted: bool,

    /// The namespace the command belongs to
    pub namespace: String,
}

impl PartialEq for HoardCmd {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.namespace == other.namespace
            && self.command == other.command
            && self.description == other.description
            && self.tags == other.tags
    }
}

impl HoardCmd {
    /// Create a new `HoardCmd` with default values
    pub fn default() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            description: String::new(),
            tags: Vec::new(),
            created: time::SystemTime::now(),
            modified: time::SystemTime::now(),
            last_used: time::SystemTime::now(),
            usage_count: 0,
            is_favorite: false,
            is_hidden: false,
            is_deleted: false,
            namespace: String::new(),
        }
    }

    #[allow(dead_code)]
    /// set the name of the command
    pub fn with_name(self, name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..self
        }
    }

    /// Set the command to be stored and executed
    pub fn with_command(self, command: &str) -> Self {
        Self {
            command: command.to_string(),
            ..self
        }
    }

    #[allow(dead_code)]
    /// Set the description the command belongs to
    pub fn with_description(self, description: &str) -> Self {
        Self {
            description: description.to_string(),
            ..self
        }
    }

    #[allow(dead_code)]
    /// set the tags of the command from a vector of strings
    pub fn with_tags(self, tags: Vec<String>) -> Self {
        Self { tags, ..self }
    }

    /// Check if a command is valid for saving
    /// A valid command cant be an empty string
    /// Returns a Result with the error if the command is invalid
    pub fn is_command_valid(c: &str) -> Result<(), HoardErr> {
        if c.is_empty() {
            return Err(HoardErr::new("Command can't be empty"));
        }
        Ok(())
    }

    /// Check if a command is valid
    /// A valid command must have:
    /// - A name that is not empty
    /// - A command that is not empty
    /// - A namespace that is not empty
    /// - `created/modified/last_used` that is not the `UNIX_EPOCH`
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
            && !self.command.is_empty()
            && !self.namespace.is_empty()
            && self.created != time::UNIX_EPOCH
            && self.modified != time::UNIX_EPOCH
            && self.last_used != time::UNIX_EPOCH
    }

    /// Check if a name is valid for saving
    /// A valid name cant be an empty string and can't contain whitespaces
    /// Returns a Result with the error if the name is invalid
    pub fn is_name_valid(c: &str) -> Result<(), HoardErr> {
        if c.is_empty() {
            return Err(HoardErr::new("Name can't be empty"));
        }
        if c.contains(' ') {
            return Err(HoardErr::new("Name can't contain whitespaces"));
        }
        Ok(())
    }

    /// Check if the tags are valid for saving
    /// A valid tag vector cant be empty
    /// Returns a Result with the error if the tags are invalid
    pub fn are_tags_valid(c: &str) -> Result<(), HoardErr> {
        if c.is_empty() {
            return Err(HoardErr::new("Tags can't be empty"));
        }
        Ok(())
    }

    /// Return vector of tags as a string
    /// Tags are separated by a comma
    /// # Example  
    /// ```
    /// use hoardlib::command::HoardCmd;
    ///
    /// let mut cmd = HoardCmd::default();
    /// cmd.tags.push("tag1".to_string());
    /// cmd.tags.push("tag2".to_string());
    /// cmd.tags.push("tag3".to_string());
    ///
    /// assert_eq!(cmd.get_tags_as_string(), "tag1,tag2,tag3");
    /// ```
    pub fn get_tags_as_string(&self) -> String {
        let mut tags = String::new();
        for tag in &self.tags {
            tags.push_str(tag);
            tags.push(',');
        }
        tags.pop();
        tags
    }

    #[allow(dead_code)]
    pub fn with_command_raw(self, command_string: &str) -> Self {
        Self {
            command: command_string.to_string(),
            ..self
        }
    }
    /// Prompts the user for a command string, with optional default value and parameter tokens.
    ///
    /// This function prompts the user for a command string. The user can mark unknown parameters with a specified token
    /// and name the parameter with any string, ending it with a specified ending token. An optional default value can be provided.
    ///
    /// # Arguments
    ///
    /// * `default_value` - An Option that holds a default value for the command string.
    /// * `parameter_token` - A string slice that holds the token to mark unknown parameters.
    /// * `parameter_ending_token` - A string slice that holds the token to end the parameter name.
    ///
    /// # Returns
    ///
    /// This function returns a new instance of the command with the user-inputted command string.
    ///
    /// # Example
    ///
    /// ```
    /// let command = HoardCmd::default();
    /// let command_with_input = command.with_command_string_input(None, "#", "$");
    /// // The user is prompted for a command string.
    /// // The command string is updated with the user's input.
    /// ```
    pub fn with_command_string_input(
        self,
        default_value: Option<String>,
        parameter_token: &str,
        parameter_ending_token: &str,
    ) -> Self {
        let base_prompt = format!(
            "Command to hoard ( Mark unknown parameters with '{parameter_token}'. Name the parameter with any string and end it with '{parameter_ending_token}' )\n"
        );
        let command_string: String = prompt_input(&base_prompt, false, default_value);
        Self {
            command: command_string,
            ..self
        }
    }

    #[allow(dead_code)]
    /// set the namespace of the command
    pub fn with_namespace(self, namespace: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            ..self
        }
    }

    /// set a random suffix to the name of the command
    pub fn with_random_name_suffix(self) -> Self {
        let rng = rand::thread_rng();
        let random_string: String = rng
            .sample_iter(&Alphanumeric)
            .take(4)
            .map(char::from)
            .collect();
        Self {
            name: format!("{}-{random_string}", self.name),
            ..self
        }
    }

    /// set the tags of the command from a string split by `,`
    pub fn with_tags_raw(self, tags: &str) -> Self {
        // If tags are empty, just return self
        if tags.trim().is_empty() {
            return self;
        }
        Self {
            tags: tags.split(',').map(|s| s.trim().to_string()).collect(),
            ..self
        }
    }

    /// Prompts the user for tags, with an optional default value, and validates the input.
    ///
    /// This function prompts the user for tags, which are comma-separated. The input is validated to ensure that
    /// it does not contain any whitespaces. An optional default value can be provided.
    ///
    /// # Arguments
    ///
    /// * `default_value` - An Option that holds a default value for the tags.
    ///
    /// # Returns
    ///
    /// This function returns a new instance of the command with the user-inputted tags.
    ///
    /// # Example
    ///
    /// ```
    /// let command = HoardCmd::default();
    /// let command_with_tags = command.with_tags_input(Some("default-tag"));
    /// // The user is prompted for tags.
    /// // The tags are updated with the user's input.
    /// ```
    pub fn with_tags_input(self, default_value: Option<String>) -> Self {
        let tag_validator = move |input: &String| -> Result<(), String> {
            if input.contains(' ') {
                Err("Tags can't contain whitespaces".to_string())
            } else {
                Ok(())
            }
        };
        let tags: String = prompt_input_validate(
            "Give your command some optional tags ( comma separated )",
            true,
            default_value,
            Some(tag_validator),
        );
        self.with_tags_raw(&tags)
    }

    pub fn with_namespace_input(self, selection: &[&str]) -> Self {
        // Add "New namespace" option to selction
        let mut selection = selection.to_vec();
        selection.push("New namespace");

        let selected: usize = prompt_select_with_options("Namespace of the command", &selection);

        let mut selected_namespace: String = (*selection.get(selected).unwrap()).to_string();
        if selected_namespace == "New namespace" {
            selected_namespace = prompt_input(
                "Namespace of the command",
                false,
                Some(String::from("default")),
            );
        }

        Self {
            namespace: selected_namespace,
            ..self
        }
    }

    fn with_name_input_prompt(
        self,
        default_value: Option<String>,
        trove: &Trove,
        prompt_string: &str,
    ) -> Self {
        let namespace = self.namespace.clone();
        let command_names = trove.commands.clone();
        let validator = move |input: &String| -> Result<(), String> {
            if input.contains(' ') {
                Err("The name can't contain whitespaces".to_string())
            } else if command_names
                .iter()
                .filter(|x| x.namespace == namespace)
                .any(|x| x.name == *input)
            {
                Err(
                    "A command with same name exists in the this namespace. Input a different name"
                        .to_string(),
                )
            } else {
                Ok(())
            }
        };
        let name = prompt_input_validate(prompt_string, false, default_value, Some(validator));
        Self { name, ..self }
    }

    pub fn with_name_input(self, default_value: Option<String>, trove: &Trove) -> Self {
        self.with_name_input_prompt(default_value, trove, "Name your command")
    }

    #[allow(dead_code)]
    pub fn resolve_name_conflict_random(self) -> Self {
        let rng = rand::thread_rng();
        let random_string: String = rng
            .sample_iter(&Alphanumeric)
            .take(3)
            .map(char::from)
            .collect();
        Self {
            name: format!("{}-{random_string}", self.name),
            ..self
        }
    }

    #[allow(dead_code)]
    /// Resolves a name conflict when a command should be added to a trove file.
    ///
    /// This function takes a command with a conflicting name and a reference to a trove. It prompts the user to resolve the conflict
    /// by either replacing the existing command, keeping the existing command, or providing a new name for the new command.
    ///
    /// # Arguments
    ///
    /// * `collision` - A command that has a name conflict with the current command.
    /// * `trove` - A reference to a trove where the command should be added.
    ///
    /// # Returns
    ///
    /// This function returns a tuple of options. If the first option is set, the new command should be added. If the second option is set,
    /// the existing command should be removed.
    ///
    /// # Example
    ///
    /// ```
    /// let command = HoardCmd::default().with_command("echo Hello, world!");
    /// let colliding_command = HoardCmd::default().with_command("echo Hello, world!");
    /// let trove = Trove::new();
    /// let (add_command, remove_command) = command.resolve_name_conflict(colliding_command, &trove);
    /// // The user is prompted to resolve the conflict.
    /// // The commands to add and remove are determined based on the user's input.
    /// ```
    pub fn resolve_name_conflict(
        self,
        collision: Self,
        trove: &Trove,
    ) -> (Option<Self>, Option<Self>) {
        // Behaviour if a command should be added to a trove file
        // Returns a tuple of options
        // If the first is set, add this as a new command
        // If the second is set, remove this exact command
        let name = self.name.clone();
        let command = self.command.clone();
        let namespace = self.namespace.clone();
        let colliding_command = collision.command.clone();
        // Ask user how to resolve conflict
        let mode: ConflictResolve =
            with_conflict_resolve_prompt(&name, &namespace, &command, &colliding_command);

        match mode {
            ConflictResolve::Replace => {
                // Add new command, remove colliding command in the local trove
                (Some(self), Some(collision))
            }
            ConflictResolve::Keep => {
                // Do nothing
                (None, None)
            }
            ConflictResolve::New => {
                (Some(self.with_name_input_prompt(
                    None,
                    trove,
                    &format!(
                        "Enter a new name for command: '{command}'\nOld name: {name} in namespace: {namespace}\nEnter new name: "
                    ),
                )) , None)
            }
        }
    }

    pub fn with_description_input(self, default_value: String) -> Self {
        let description_string: String =
            prompt_input("Describe what the command does", false, Some(default_value));
        Self {
            description: description_string,
            ..self
        }
    }

    #[allow(dead_code)]
    /// increase the usage count of the command
    pub fn mut_increase_usage_count(&mut self) -> &mut Self {
        self.usage_count += 1;
        self
    }

    #[allow(dead_code)]
    /// sets the favorite flag of the command
    pub fn mut_set_favorite(&mut self, is_favorite: bool) -> &mut Self {
        self.is_favorite = is_favorite;
        self
    }

    #[allow(dead_code)]
    /// sets the hidden flag of the command
    pub fn mut_set_hidden(&mut self, is_hidden: bool) -> &mut Self {
        self.is_hidden = is_hidden;
        self
    }

    #[allow(dead_code)]
    /// sets the deleted flag of the command
    pub fn mut_set_deleted(&mut self, is_deleted: bool) -> &mut Self {
        self.is_deleted = is_deleted;
        self
    }
}

pub fn string_to_tags(tags: &str) -> Vec<String> {
    tags.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .split(',')
        .map(std::string::ToString::to_string)
        .collect()
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn one_tag_as_string() {
        let command = HoardCmd::default().with_tags_raw("foo");
        let expected = "foo";
        assert_eq!(expected, command.get_tags_as_string());
    }

    #[test]
    fn no_tag_as_string() {
        let command = HoardCmd::default();
        let expected = "";
        assert_eq!(expected, command.get_tags_as_string());
    }

    #[test]
    fn multiple_tags_as_string() {
        let command = HoardCmd::default().with_tags_raw("foo,bar");
        let expected = "foo,bar";
        assert_eq!(expected, command.get_tags_as_string());
    }

    #[test]
    fn parse_single_tag() {
        let command = HoardCmd::default().with_tags_raw("foo");
        let expected = vec!["foo".to_string()];
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_multiple_tags() {
        let command = HoardCmd::default().with_tags_raw("foo,bar");
        let expected = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_whitespace_in_tags() {
        let command = HoardCmd::default().with_tags_raw("foo, bar");
        let expected = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(expected, command.tags);
    }
    #[test]
    fn parse_no_whitespace_in_tags() {
        let command = HoardCmd::default().with_tags_raw("foo,bar");
        let expected = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_multiple_whitespace_in_tags() {
        let command = HoardCmd::default().with_tags_raw("foo,   bar");
        let expected = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_special_characters_in_tags() {
        let command = HoardCmd::default().with_tags_raw("foo@, bar#");
        let expected = vec!["foo@".to_string(), "bar#".to_string()];
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_empty_string() {
        let command = HoardCmd::default().with_tags_raw("");
        let expected: Vec<String> = Vec::new();
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_string_with_only_whitespaces() {
        let command = HoardCmd::default().with_tags_raw("   ");
        let expected: Vec<String> = Vec::new();
        assert_eq!(expected, command.tags);
    }
}
