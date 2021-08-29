use anyhow::{anyhow, Result};
use log::info;
use prettytable::{color, Attr, Cell, Row, Table};
use serde::{Deserialize, Serialize};

use std::{any, fs, path::PathBuf};

use super::hoard_command::HoardCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandTrove {
    pub version: String,
    pub commands: Vec<HoardCommand>,
}

impl Default for CommandTrove {
    fn default() -> Self {
        CommandTrove {
            version: String::from("0.1.0"),
            commands: Vec::new(),
        }
    }
}
impl CommandTrove {
    pub fn load_trove_file(path: &Option<PathBuf>) -> Self {
        match path {
            Some(p) => {
                if p.exists() {
                    let f = std::fs::File::open(p).ok().unwrap();
                    serde_yaml::from_reader::<_, CommandTrove>(f).unwrap()
                } else {
                    info!("[DEBUG] No trove file found at {:?}", p);
                    CommandTrove::default()
                }
            }
            None => {
                info!("[DEBUG] No trove path available. Creating new trove file");
                CommandTrove::default()
            }
        }
    }

    pub fn save_trove_file(&self, path: &PathBuf) {
        let s = serde_yaml::to_string(&self).unwrap();
        fs::write(path, s).expect("Unable to write config file");
    }

    pub fn add_command(&mut self, new_command: HoardCommand) {
        self.commands.push(new_command);
    }

    pub fn pick_command(&self, name: String) -> Result<HoardCommand> {
        let filtered_command: Option<&HoardCommand> =
            self.commands.iter().filter(|c| c.name == name).nth(0);
        if let Some(command) = filtered_command {
            return Ok(command.clone());
        } else {
            return Err(anyhow!("No matching command found with name: {}", name));
        }
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
                Cell::new(&c.tags.as_ref().unwrap_or(&vec![String::from("")]).join(",")[..]),
            ]));
        });
        // Print the table to stdout
        table.printstd();
    }
}