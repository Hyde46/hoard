pub mod error;

use crate::command::HoardCommand;

use anyhow::{anyhow, Result};
use log::info;
use prettytable::{color, Attr, Cell, Row, Table};
use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::{fs, path::Path, path::PathBuf};

use self::error::TroveError;

const CARGO_VERSION: &str = env!("CARGO_PKG_VERSION");


/// Container for all stored hoard commands. 
/// A `treasure trove` of commands
/// 
/// A Trove can store the following parameters
/// - `version`: The hoard version with which the commands are being stored
///              To potentially support migrating older collections to new ones when breaking changes happen
/// - `commands`: Vector of `HoardCommand`s, the stored commands
/// - `namespaces`: Set of all namespaces used in the collection
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Trove {
    pub version: String,
    pub commands: Vec<HoardCommand>,
    pub namespaces: HashSet<String>,
}

impl Default for Trove {
    /// Create a new trove collection with the currently running hoard version
    fn default() -> Self {
        Self {
            version: CARGO_VERSION.to_string(),
            commands: Vec::new(),
            namespaces: HashSet::new(),
        }
    }
}

impl Trove {
    /// Create a new Trove from a vector of commands
    /// attaches the current hoard version to the collection
    pub fn from_commands(commands: &[HoardCommand]) -> Self {
        // Iterate through all commands, read out the namespace and collect them in the namespace hashset
        let namespaces: HashSet<String> = commands
            .iter()
            .map(|c| c.namespace.clone())
            .collect::<HashSet<String>>();

        Self {
            version: CARGO_VERSION.to_string(),
            commands: commands.to_vec(),
            namespaces,
        }
    }

    /// Loads a local trove file and tries to parse it to load it into memory
    pub fn load_trove_file(path: &Option<PathBuf>) -> Self {
        path.clone().map_or_else(
            || {
                info!("[DEBUG] No trove path available. Creating new trove file");
                Self::default()
            },
            |p| {
                if p.exists() {
                    let f = std::fs::File::open(p).ok().unwrap();
                    let parsed_trove = serde_yaml::from_reader::<_, Self>(f);
                    match parsed_trove {
                        Ok(trove) => trove,
                        Err(e) => {
                            println!("The supplied trove file is invalid!");
                            println!("{e}");
                            Self::default()
                        }
                    }
                } else {
                    info!("[DEBUG] No trove file found at {:?}", p);
                    Self::default()
                }
            },
        )
    }

    /// Loads a trove collection from a string and tries to parse it to load it into memory
    pub fn load_trove_from_string(trove_string: &str) -> Self {
        let parsed_trove = serde_yaml::from_str::<Self>(trove_string);
        match parsed_trove {
            Ok(trove) => trove,
            Err(e) => {
                println!("{e}");
                println!("The supplied trove file is invalid!");
                Self::default()
            }
        }
    }

    /// Serialize trove collection to yaml format and returns it as a string
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(&self).unwrap()
    }

    /// Save the trove collection to `path` as a yaml file
    pub fn yaml_save_trove(&self, path: &Path) {
        let s = self.to_yaml();
        fs::write(path, s).expect("Unable to write config file");
    }

    /// Given a `HoardCommand`, check if there is a command with the same name and namespace already in the collection
    /// If there is, return the colliding command
    /// If there is not, return `None`
    pub fn get_command_collision(&self, command: &HoardCommand) -> Option<HoardCommand> {
        let colliding_commands = self
            .commands
            .iter()
            .filter(|&c| c.namespace == command.namespace)
            .filter(|&c| c.name == command.name)
            .cloned();
        colliding_commands.into_iter().next()
    }

    /// Given a `HoardCommand`, check if there is a command with the same name, namespace and saved command already in the collection.
    /// A command with those same parameters is considered to be the same command
    /// If there is, return `true`
    /// If there is not, return `false`
    fn is_command_present(&self, command: &HoardCommand) -> bool {
        self.commands
            .iter()
            .filter(|&c| {
                c.namespace == command.namespace
                    && c.name == command.name
                    && c.command == command.command
            })
            .count()
            > 0
    }

    /// Remove a command from the trove collection
    /// Returns `Ok(())` if the command has been removed
    /// Returns `Err(anyhow::Error)` if the command to remove is not in the trove
    pub fn remove_command(&mut self, name: &str) -> Result<(), anyhow::Error> {
        let command_position = self.commands.iter().position(|x| x.name == name);
        if command_position.is_none() {
            return Err(anyhow!("Command not found [{}]", name));
        }
        self.commands.retain(|x| &*x.name != name);
        Ok(())
    }

    /// Add a command to trove file
    /// Returns `true` if the command has been added
    /// Returns `false` if the command has not been added due to a name collision that has been resolved where the trove did not change
    /// if overwrite_colliding is set to true, the name of the command will get a random string suffix to resolve the name collision before adding it to the trove
    /// if overwrite_colliding is set to false, the name collision will not be resolved and the command will not be added to the trove
    pub fn add_command(&mut self, new_command: HoardCommand, overwrite_colliding: bool) -> Result<bool, TroveError> {
        if !new_command.is_valid() {
            return Err(TroveError::new("cannot save invalid command"));
        }
        let dirty = match self.get_command_collision(&new_command) {
            // Collision is present, but its the same command, do nothing
            Some(_) if self.is_command_present(&new_command) => false,
            // collision is present, overwrite_colliding is true, resolve collision by overwriting
            Some(colliding_command) if overwrite_colliding => {
                self.commands.retain(|x| x != &colliding_command);
                self.commands.push(new_command);
                true
            }
            // collision is present, but overwrite_colliding is false, add random suffix before adding as a new comamnd
            Some(_) => {
                let c = new_command.with_random_name_suffix();
                self.commands.push(c);
                true
            }
            // If not collision, add the command
            None => {
                // no collision, maybe add the namespace
                self.add_namespace(&new_command.namespace);
                self.commands.push(new_command);
                true
            }
        };
        Ok(dirty)
    }

    /// try to add a namespace value to the namespaces if it is not present yet
    pub fn add_namespace(&mut self, namespace: &str) {
        if !self.namespaces.contains(namespace) {
            self.namespaces.insert(namespace.to_string());
        }
    }

    /// check if the trove collection is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn empty_trove() {
        // test creating an empty trove
        let trove = Trove::default();
        assert!(trove.is_empty());
    }

    #[test]
    fn add_invalid_command() {
        // test adding an invalid command
        // should return an error, since invalid commands should not be able to be added to the trove
        let mut trove = Trove::default();
        let command = HoardCommand::default();
        let val = trove.add_command(command, true);
        assert!(val.is_err());
    }

    #[test]
    fn add_valid_command() {
        // test adding a valid command
        // resulting trove cannot be empty
        let mut trove = Trove::default();
        let mut command = HoardCommand::new();
        command.name = "test".to_string();
        command.namespace = "test".to_string();
        command.command = "test".to_string();
        let val = trove.add_command(command, true);
        assert!(val.is_ok());
        assert!(!trove.is_empty());
        // namespace has to be present now
        assert!(trove.namespaces.contains("test"));
    }

    #[test]
    fn test_multiple_new_namespaces_added() {
        // create a new trove, add two new commands with different namespaces
        // test if the trove has both namespaces in the end
        let mut trove = Trove::default();
        let mut command1 = HoardCommand::new();
        command1.name = "test1".to_string();
        command1.namespace = "test1".to_string();
        command1.command = "test1".to_string();
        let mut command2 = HoardCommand::new();
        command2.name = "test2".to_string();
        command2.namespace = "test2".to_string();
        command2.command = "test2".to_string();
        let val1 = trove.add_command(command1, true);
        let val2 = trove.add_command(command2, true);
        assert!(val1.is_ok());
        assert!(val2.is_ok());
        assert!(!trove.is_empty());
        // namespace has to be present now
        assert!(trove.namespaces.contains("test1"));
        assert!(trove.namespaces.contains("test2"));
    }

    #[test]
    fn test_add_multiple_commands_same_namespace() {
        // create a new trove, add two new commands with the same namespace
        // test if the trove has the namespace once in the end
        let mut trove = Trove::default();
        let mut command1 = HoardCommand::new();
        command1.name = "test1".to_string();
        command1.namespace = "test".to_string();
        command1.command = "test1".to_string();
        let mut command2 = HoardCommand::new();
        command2.name = "test2".to_string();
        command2.namespace = "test".to_string();
        command2.command = "test2".to_string();
        let val1 = trove.add_command(command1, true);
        let val2 = trove.add_command(command2, true);
        assert!(val1.is_ok());
        assert!(val2.is_ok());
        assert!(!trove.is_empty());
        // namespace has to be present now
        assert!(trove.namespaces.contains("test"));
        // namespace only contains one value
        assert_eq!(trove.namespaces.len(), 1);
    }

    #[test]
    fn test_add_and_remove_command() {
        // create a new trove, add a command, verify the command is not empty,
        // then remove the command and verify the trove to be empty.
        // the namespace that got added should still exist
        let mut trove = Trove::default();
        let mut command = HoardCommand::new();
        command.name = "test".to_string();
        command.namespace = "test".to_string();
        command.command = "test".to_string();
        let val = trove.add_command(command, true);
        assert!(val.is_ok());
        assert!(!trove.is_empty());
        // namespace has to be present now
        assert!(trove.namespaces.contains("test"));
        // remove the command
        let val = trove.remove_command("test"); 
        assert!(val.is_ok());
        assert!(trove.is_empty());
        // namespace has to be present now
        assert!(trove.namespaces.contains("test"));
    }
}
