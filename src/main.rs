#![warn(clippy::pedantic, clippy::nursery)]

extern crate serde_yaml;

#[macro_use]
extern crate prettytable;

use std::io;

mod command;
mod config;
mod gui;
mod hoard;
use hoard::Hoard;

#[tokio::main]
async fn main() -> io::Result<()> {
    let command = Hoard::default().with_config(None).load_trove().start().await;
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
