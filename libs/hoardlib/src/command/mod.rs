pub mod error;
pub mod parameters;
pub mod validate;

use std::time;
use serde::{Deserialize, Serialize};
use rand::distributions::Alphanumeric;
use rand::Rng;

use self::error::CommandError;

/// Storage for the saved command structure
/// 
/// A HoardCommand can store the following parameters
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HoardCommand {
    /// The name of the command by which it is referenced
    pub name: String,

    /// The terminal command to be stored and executed
    pub command: String,

    /// A description of the command for the user
    pub description: String,

    /// A list of tags to be used for searching
    pub tags: Vec<String>,

    /// The date and time the command was created
    pub created: time::SystemTime,

    /// The date and time the command was last modified
    pub modified: time::SystemTime,

    /// The date and time the command was last used
    pub last_used: time::SystemTime,

    /// The number of times the command has been used
    pub usage_count: usize,

    /// A flag to indicate if the command is a favorite
    pub is_favorite: bool,

    /// A flag to indicate if the command is hidden
    pub is_hidden: bool,

    /// A flag to indicate if the command is deleted
    pub is_deleted: bool,

    /// The namespace the command belongs to
    pub namespace: String,
}

impl HoardCommand {
    /// Create a new HoardCommand with default values
    pub const fn default() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            description: String::new(),
            tags: Vec::new(),
            created: time::UNIX_EPOCH,
            modified: time::UNIX_EPOCH,
            last_used: time::UNIX_EPOCH,
            usage_count: 0,
            is_favorite: false,
            is_hidden: false,
            is_deleted: false,
            namespace: String::new(),
        }
    }

    /// Create a new HoardCommand with default values
    /// set created, modified and last_used to the current time
    pub fn new() -> Self {
        let mut command = Self::default();
        command.created = time::SystemTime::now();
        command.modified = time::SystemTime::now();
        command.last_used = time::SystemTime::now();
        command
    }

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

    /// Set the description the command belongs to
    pub fn with_description(self, description: &str) -> Self {
        Self {
            description: description.to_string(),
            ..self
        }
    }

    /// set the tags of the command from a vector of strings
    pub fn with_tags(self, tags: Vec<String>) -> Self {
        Self {
            tags,
            ..self
        }
    }   

    /// set the tags of the command from a string split by `,`
    pub fn with_tags_raw(self, tags: &str) -> Self {
        Self {
            tags: tags.split(',').map(|s| s.trim().to_string()).collect(),
            ..self
        }
    }

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

    /// increase the usage count of the command
    pub fn mut_increase_usage_count(&mut self) -> &mut Self {
        self.usage_count += 1;
        self
    }

    /// sets the favorite flag of the command
    pub fn mut_set_favorite(&mut self, is_favorite: bool) -> &mut Self {
        self.is_favorite = is_favorite;
        self
    }

    /// sets the hidden flag of the command
    pub fn mut_set_hidden(&mut self, is_hidden: bool) -> &mut Self {
        self.is_hidden = is_hidden;
        self
    }

    /// sets the deleted flag of the command
    pub fn mut_set_deleted(&mut self, is_deleted: bool) -> &mut Self {
        self.is_deleted = is_deleted;
        self
    }
    
    /// Check if a command is valid
    /// A valid command must have:
    /// - A name that is not empty
    /// - A command that is not empty
    /// - A namespace that is not empty
    /// - created/modified/last_used that is not the UNIX_EPOCH
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
            && !self.command.is_empty()
            && !self.namespace.is_empty()
            && self.created != time::UNIX_EPOCH
            && self.modified != time::UNIX_EPOCH
            && self.last_used != time::UNIX_EPOCH
    }

    /// Check if a command is valid for saving
    /// A valid command cant be an empty string
    /// Returns a Result with the error if the command is invalid
    pub fn is_command_valid(c: &str) -> Result<(), CommandError> {
        if c.is_empty() {
            return Err(CommandError::new("Command can't be empty"));
        }
        Ok(())
    }

    /// Check if a name is valid for saving
    /// A valid name cant be an empty string and can't contain whitespaces
    /// Returns a Result with the error if the name is invalid
    pub fn is_name_valid(c: &str) -> Result<(), CommandError>  {
        if c.is_empty() {
            return Err(CommandError::new("Name can't be empty"));
        }
        if c.contains(' ') {
            return Err(CommandError::new("Name can't contain whitespaces"));
        }
        Ok(())
    }

    /// Check if the tags are valid for saving
    /// A valid tag vector cant be empty
    /// Returns a Result with the error if the tags are invalid
    pub fn are_tags_valid(c: &str) -> Result<(), CommandError>  {
        if c.is_empty() {
            return Err(CommandError::new("Tags can't be empty"));
        }
        Ok(())
    }

    /// Return vector of tags as a string
    /// Tags are separated by a comma
    /// # Example  
    /// ```
    /// use hoardlib::command::HoardCommand;
    /// 
    /// let mut cmd = HoardCommand::default();
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
    
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn one_tag_as_string() {
        let command = HoardCommand::default().with_tags_raw("foo");
        let expected = "foo";
        assert_eq!(expected, command.get_tags_as_string());
    }

    #[test]
    fn no_tag_as_string() {
        let command = HoardCommand::default();
        let expected = "";
        assert_eq!(expected, command.get_tags_as_string());
    }

    #[test]
    fn multiple_tags_as_string() {
        let command = HoardCommand::default().with_tags_raw("foo,bar");
        let expected = "foo,bar";
        assert_eq!(expected, command.get_tags_as_string());
    }

    #[test]
    fn parse_single_tag() {
        let command = HoardCommand::default().with_tags_raw("foo");
        let expected = vec!["foo".to_string()];
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_multiple_tags() {
        let command = HoardCommand::default().with_tags_raw("foo,bar");
        let expected = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_whitespace_in_tags() {
        let command = HoardCommand::default().with_tags_raw("foo, bar");
        let expected = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(expected, command.tags);
    }
}