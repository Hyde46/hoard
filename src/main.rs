extern crate crossterm;
extern crate serde_yaml;

#[macro_use]
extern crate prettytable;

use std::io::{self, Write};

mod command;
mod config;
mod gui;
mod hoard;
use hoard::Hoard;
fn main() -> io::Result<()> {
    let command = Hoard::default().with_config(None).load_trove().start();
    match command {
        // Prints to stdout for autocomplete functionality
        Ok((command, is_autocomplete)) => {
            if is_autocomplete {
                eprintln!("{}", command.clone());
            }
        }
        Err(err) => println!("{}", err),
    }
    Ok(())
}
