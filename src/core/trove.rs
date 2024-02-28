use anyhow::{anyhow, Result};
use log::info;
use prettytable::{color, Attr, Cell, Row, Table};
use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::{fs, path::Path, path::PathBuf};

use crate::config::HoardConfig;
use crate::core::error::HoardErr;
use crate::core::parameters::Parameterized;
use crate::core::HoardCmd;

const CARGO_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Container for all stored hoard commands.
/// A `treasure trove` of commands
///
/// A Trove can store the following parameters
/// - `version`: The hoard version with which the commands are being stored
///              To potentially support migrating older collections to new ones when breaking changes happen
/// - `commands`: Vector of `HoardCmd`s, the stored commands
/// - `namespaces`: Set of all namespaces used in the collection
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Trove {
    pub version: String,
    pub commands: Vec<HoardCmd>,
    #[serde(default)]
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
    pub fn from_commands(commands: &[HoardCmd]) -> Self {
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
        let mut trove = path.clone().map_or_else(
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
                            eprintln!("The supplied trove file is invalid!");
                            eprintln!("{e}");
                            Self::default()
                        }
                    }
                } else {
                    info!("[DEBUG] No trove file found at {:?}", p);
                    Self::default()
                }
            },
        );
        trove.namespaces = trove.namespaces().into_iter().map(std::string::ToString::to_string).collect();
        trove
    }

    /// Loads a trove collection from a string and tries to parse it to load it into memory
    pub fn load_trove_from_string(trove_string: &str) -> Self {
        let parsed_trove = serde_yaml::from_str::<Self>(trove_string);
        let mut trove = match parsed_trove {
            Ok(trove) => trove,
            Err(e) => {
                eprintln!("{e}");
                eprintln!("The supplied trove file is invalid!");
                Self::default()
            }
        };
        trove.namespaces = trove.namespaces().into_iter().map(std::string::ToString::to_string).collect();
        trove
    }

    /// Serialize trove collection to yaml format and returns it as a string
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(&self).unwrap()
    }

    /// Save the trove collection to `path` as a yaml file
    pub fn save_trove_file(&self, path: &Path) {
        let s = self.to_yaml();
        fs::write(path, s).expect("Unable to write config file");
    }

    /// Given a `HoardCmd`, check if there is a command with the same name and namespace already in the collection
    /// If there is, return the colliding command
    /// If there is not, return `None`
    pub fn get_command_collision(&self, command: &HoardCmd) -> Option<HoardCmd> {
        let colliding_commands = self
            .commands
            .iter()
            .filter(|&c| c.namespace == command.namespace)
            .filter(|&c| c.name == command.name)
            .cloned();
        colliding_commands.into_iter().next()
    }

    /// Given a `HoardCmd`, check if there is a command with the same name, namespace and saved command already in the collection.
    /// A command with those same parameters is considered to be the same command
    /// If there is, return `true`
    /// If there is not, return `false`
    fn is_command_present(&self, command: &HoardCmd) -> bool {
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

    /// Adds a command to trove file
    /// 
    /// Returns `true` if the command has been added
    /// 
    /// Returns `false` if the command has not been added due to a name collision that has been resolved where the trove did not change
    /// 
    /// if `overwrite_colliding` is set to true, the name of the command will get a random string suffix to resolve the name collision before adding it to the trove
    /// 
    /// if `overwrite_colliding` is set to false, the name collision will not be resolved and the command will not be added to the trove
    pub fn add_command(
        &mut self,
        new_command: HoardCmd,
        overwrite_colliding: bool,
    ) -> Result<bool, HoardErr> {
        if !new_command.is_valid() {
            return Err(HoardErr::new("cannot save invalid command"));
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

    /// Remove a command from the trove collection
    /// 
    /// Returns `Ok(())` if the command has been removed
    /// 
    /// Returns `Err(anyhow::Error)` if the command to remove is not in the trove
    pub fn remove_command(&mut self, name: &str) -> Result<(), anyhow::Error> {
        let command_position = self.commands.iter().position(|x| x.name == name);
        if command_position.is_none() {
            return Err(anyhow!("Command not found [{}]", name));
        }
        self.commands.retain(|x| &*x.name != name);
        Ok(())
    }

    pub fn remove_namespace_commands(&mut self, namespace: &str) -> Result<(), anyhow::Error> {
        let command_position = self.commands.iter().position(|x| x.namespace == namespace);
        if command_position.is_none() {
            return Err(anyhow!("No Commands found in namespace [{}]", namespace));
        }
        self.commands.retain(|x| &*x.namespace != namespace);
        Ok(())
    }

    pub fn namespaces(&self) -> Vec<&str> {
        // Returns all namespaces in the trove
        let mut namespaces: Vec<_> = self
            .commands
            .iter()
            .map(|command| command.namespace.as_str())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        namespaces.sort_unstable();
        namespaces
    }

    pub fn pick_command(&self, config: &HoardConfig, name: &str) -> Result<HoardCmd> {
        let filtered_command: Option<&HoardCmd> = self.commands.iter().find(|c| c.name == name);
        filtered_command.map_or_else(
            || Err(anyhow!("No matching command found with name: {}", name)),
            |command| {
                let command = command.clone().with_input_parameters(
                    &config.parameter_token.clone().unwrap(),
                    &config.parameter_ending_token.clone().unwrap(),
                );
                Ok(command)
            },
        )
    }

    pub fn update_command_by_name(&mut self, command: &HoardCmd) -> &mut Self {
        for c in &mut self.commands.iter_mut() {
            if c.name == command.name {
                *c = command.clone();
            }
        }
        self
    }

    /// check if the trove collection is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn merge_trove(&mut self, other: &Self) -> bool {
        other
            .commands
            .iter()
            .map(|c| self.add_command(c.clone(), true))
            .any(|x| x.is_ok())
    }

    pub fn print_trove(&self) {
        // Create the table
        let mut table = Table::new();
        // Add header
        table.add_row(row!["Name", "namespace", "command", "description", "tags"]);
        // Iterate through trove and populate table
        self.commands.iter().for_each(|c| {
            table.add_row(Row::new(vec![
                // Name
                Cell::new(&c.name[..])
                    .with_style(Attr::Bold)
                    .with_style(Attr::ForegroundColor(color::GREEN)),
                // namespace
                Cell::new(&c.namespace[..]),
                // command
                Cell::new(&c.command[..]),
                // description
                Cell::new(&c.description[..]),
                // tags
                Cell::new(&c.get_tags_as_string()),
            ]));
        });
        // Print the table to stdout
        table.printstd();
    }
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn empty_trove() {
        let trove = Trove::default();
        assert!(trove.is_empty());
    }

    #[test]
    fn not_empty_trove() {
        let mut trove = Trove::default();
        let command = HoardCmd::default()
            .with_name("test")
            .with_namespace("test-namespace")
            .with_command("echo 'test'");
        let val = trove.add_command(command, true);
        assert!(val.is_ok());
        assert!(!trove.is_empty());
    }

    #[test]
    fn trove_namespaces() {
        let namespace1 = "NAMESPACE1";
        let namespace2 = "NAMESPACE2";

        let command1 = HoardCmd {
            name: "name1".to_string(),
            namespace: namespace1.to_string(),
            command: "command1".to_string(),
            ..HoardCmd::default()
        };

        let command2 = HoardCmd {
            name: "name2".to_string(),
            namespace: namespace2.to_string(),
            command: "command2".to_string(),
            ..HoardCmd::default()
        };

        let command3 = HoardCmd {
            name: "name3".to_string(),
            namespace: namespace1.to_string(),
            command: "command3".to_string(),
            ..HoardCmd::default()
        };

        let mut trove = Trove::default();
        let res1 = trove.add_command(command1, true);
        assert!(res1.is_ok());
        let res2 = trove.add_command(command2, true);
        assert!(res2.is_ok());
        let res3 = trove.add_command(command3, true);
        assert!(res3.is_ok());

        assert_eq!(vec![namespace1, namespace2], trove.namespaces());
    }

    #[test]
    fn add_valid_command() {
        // test adding a valid command
        // resulting trove cannot be empty
        let mut trove = Trove::default();
        let mut command = HoardCmd::default();
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
        let mut command1 = HoardCmd::default();
        command1.name = "test1".to_string();
        command1.namespace = "test1".to_string();
        command1.command = "test1".to_string();
        let mut command2 = HoardCmd::default();
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
        let mut command1 = HoardCmd::default();
        command1.name = "test1".to_string();
        command1.namespace = "test".to_string();
        command1.command = "test1".to_string();
        let mut command2 = HoardCmd::default();
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
        let mut command = HoardCmd::default();
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

    #[test]
    fn test_add_command_with_same_name() {
        // create a new trove, add a command, verify the command is not empty,
        // then add the same command again and check the result.
        let mut trove = Trove::default();
        let mut command = HoardCmd::default();
        command.name = "test".to_string();
        command.namespace = "test".to_string();
        command.command = "test".to_string();
        let val1 = trove.add_command(command.clone(), true);
        assert!(val1.is_ok());
        assert!(!trove.is_empty());
        // namespace has to be present now
        assert!(trove.namespaces.contains("test"));
        // add the same command again
        let val2 = trove.add_command(command, true);
        // check the result of adding the same command again
        // if it overwrites the existing command, val2 should be Ok
        // if it returns an error, val2 should be Err
        assert!(val2.is_ok());
    }

    #[test]
    fn test_remove_nonexistent_command() {
        // create a new trove and try to remove a command that doesn't exist
        let mut trove = Trove::default();
        let val = trove.remove_command("nonexistent");
        // check the result of removing a nonexistent command
        // if it returns an error, val should be Err
        // if it silently fails, val should be Ok
        assert!(val.is_err());
    }

    #[test]
    fn test_add_remove_commands_different_namespaces() {
        // create a new trove, add commands in different namespaces, verify the commands are not empty,
        // then remove the commands and check the result.
        let mut trove = Trove::default();
        let mut command1 = HoardCmd::default();
        command1.name = "test1".to_string();
        command1.namespace = "namespace1".to_string();
        command1.command = "test1".to_string();
        let val1 = trove.add_command(command1, true);
        assert!(val1.is_ok());
        assert!(!trove.is_empty());
        // namespace1 has to be present now
        assert!(trove.namespaces.contains("namespace1"));

        let mut command2 = HoardCmd::default();
        command2.name = "test2".to_string();
        command2.namespace = "namespace2".to_string();
        command2.command = "test2".to_string();
        let val2 = trove.add_command(command2, true);
        assert!(val2.is_ok());
        assert!(!trove.is_empty());
        // namespace2 has to be present now
        assert!(trove.namespaces.contains("namespace2"));

        // remove the commands
        let val3 = trove.remove_command("test1");
        assert!(val3.is_ok());
        let val4 = trove.remove_command("test2");
        assert!(val4.is_ok());

        // check if trove is empty after removing the commands
        assert!(trove.is_empty());
    }

    #[test]
    fn test_is_empty_new_trove() {
        // create a new trove and check if it is empty
        let trove = Trove::default();
        assert!(trove.is_empty());
    }

    #[test]
    fn test_contains_new_trove() {
        // create a new trove and check if it contains a command
        let trove = Trove::default();
        // Should not contain a command
        assert!(!trove.namespaces.contains("test"));
    }
}
