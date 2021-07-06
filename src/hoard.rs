use std::fs;

use crate::command::{argument::Argument, command::Command};

use clap::{load_yaml, App};

#[derive(Debug)]
pub struct Hoard {}

impl Hoard {
    pub fn default() -> Self {
        Hoard {}
    }

    pub fn start(&self) -> Result<(), serde_yaml::Error> {
        let yaml = load_yaml!("resources/cli.yaml");
        let matches = App::from(yaml).get_matches();
        if let Some(ref matches) = matches.subcommand_matches("new") {
            // "$ hoard new" was run
            // TODO: Only testing saving yaml files!
            // Does not support multiple commands
            // Nor does it append to old files
            // Just testing the easy yaml creation
            let s = serde_yaml::to_string(
                Command::new(
                    String::from("new"),
                    vec![
                        Argument::new(String::from("new")),
                        Argument::new(String::from("tags")),
                    ],
                )
                .populate(matches)
                .complete_interactive(),
            )?;
            println!("{:?}", s);
            fs::write("./.hoard.yaml", s).expect("Unable to write file");
        }
        Ok(())
    }
}
