pub mod error;
pub mod parameters;
pub mod validate;

use std::time;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// The id of the namespace the command belongs to
    pub namespace_id: usize,
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
            namespace_id: 0,
        }
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