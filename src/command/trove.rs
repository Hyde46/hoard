use log::info;
use serde::{Deserialize, Serialize};

use std::{fs, path::PathBuf};

use super::new_command::NewCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandTrove {
    pub version: String,
    pub commands: Vec<NewCommand>,
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

    pub fn add_command(&mut self, new_command: NewCommand) {
        self.commands.push(new_command);
    }
}
