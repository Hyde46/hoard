use anyhow::{anyhow, Result};
use log::info;
use prettytable::{color, Attr, Cell, Row, Table};
use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::{fs, path::Path, path::PathBuf};

use crate::command::hoard_command::{HoardCommand, Parameterized};
use crate::config::HoardConfig;

const CARGO_VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CommandTrove {
    pub version: String,
    pub commands: Vec<HoardCommand>,
}

impl Default for CommandTrove {
    fn default() -> Self {
        Self {
            version: CARGO_VERSION.to_string(),
            commands: Vec::new(),
        }
    }
}
impl CommandTrove {
    pub fn from_commands(commands: &[HoardCommand]) -> Self {
        Self {
            version: CARGO_VERSION.to_string(),
            commands: commands.to_vec(),
        }
    }

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

    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(&self).unwrap()
    }

    pub fn save_trove_file(&self, path: &Path) {
        let s = self.to_yaml();
        fs::write(path, s).expect("Unable to write config file");
    }

    pub fn check_name_collision(&self, command: &HoardCommand) -> Option<HoardCommand> {
        let colliding_commands = self
            .commands
            .iter()
            .filter(|&c| c.namespace == command.namespace)
            .filter(|&c| c.name == command.name)
            .cloned();
        colliding_commands.into_iter().next()
    }

    fn is_same_command(&self, command: &HoardCommand) -> bool {
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

    pub fn add_command(&mut self, new_command: HoardCommand, check_name_collision: bool) -> bool {
        // Add a command to local trove file.
        // Returns dirty flag, whether something got added/changed or not
        // Returns true if there were changes
        // Returns false if synced troved file was either empty or the exact same
            let (to_add, to_remove): (Option<HoardCommand>, Option<HoardCommand>) = 
                if let Some(colliding_command) = self.check_name_collision(&new_command) {
                    if self.is_same_command(&new_command) {
                        // Dont add it to the trove since its the same command. Dont ask the user about it
                        (None, None)
                    } else if !check_name_collision {
                        let c = new_command.resolve_name_conflict_random();
                        (Some(c), None)
                    } else {
                        new_command.resolve_name_conflict(colliding_command, self)
                    }
                } else {
                    (Some(new_command), None)
                };
        if let Some(remove) = to_remove {
            match self.remove_command(&remove.name) {
                Ok(_) => {}
                Err(e) => println!("No idea how you ended up here {e}"),
            }
        }
        if let Some(c) = to_add {
            self.commands.push(c);
            return true;
        }
        false
    }

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

    pub fn pick_command(&self, config: &HoardConfig, name: &str) -> Result<HoardCommand> {
        let filtered_command: Option<&HoardCommand> = self.commands.iter().find(|c| c.name == name);
        filtered_command.map_or_else(
            || Err(anyhow!("No matching command found with name: {}", name)),
            |command| {
                let command = command
                    .clone()
                    .with_input_parameters(&config.parameter_token.clone().unwrap(), &config.parameter_ending_token.clone().unwrap());
                Ok(command)
            },
        )
    }

    pub fn update_command_by_name(&mut self, command: &HoardCommand) -> &mut Self {
        for c in &mut self.commands.iter_mut() {
            if c.name == command.name {
                *c = command.clone();
            }
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn merge_trove(&mut self, other: &Self) -> bool {
        other
            .commands
            .iter()
            .map(|c| self.add_command(c.clone(), true))
            .any(|c| c)
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
                Cell::new(&c.description.as_ref().unwrap()[..]),
                // tags
                Cell::new(&c.tags.as_ref().unwrap_or(&vec![String::new()]).join(",")[..]),
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
        let trove = CommandTrove::default();
        assert!(trove.is_empty());
    }

    #[test]
    fn not_empty_trove() {
        let mut trove = CommandTrove::default();
        let command = HoardCommand::default();
        trove.add_command(command, true);
        assert!(!trove.is_empty());
    }

    #[test]
    fn trove_namespaces() {
        let namespace1 = "NAMESPACE1";
        let namespace2 = "NAMESPACE2";

        let command1 = HoardCommand {
            name: "name1".to_string(),
            namespace: namespace1.to_string(),
            ..HoardCommand::default()
        };

        let command2 = HoardCommand {
            name: "name2".to_string(),
            namespace: namespace2.to_string(),
            ..HoardCommand::default()
        };

        let command3 = HoardCommand {
            name: "name3".to_string(),
            namespace: namespace1.to_string(),
            ..HoardCommand::default()
        };

        let mut trove = CommandTrove::default();
        trove.add_command(command1, true);
        trove.add_command(command2, true);
        trove.add_command(command3, true);

        assert_eq!(vec![namespace1, namespace2], trove.namespaces());
    }
}
