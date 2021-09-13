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

fn main() -> io::Result<()> {
    let (command, is_autocomplete) = Hoard::default().with_config(None).load_trove().start();
    if is_autocomplete {
        eprintln!("{}", command.clone());
    }
    Ok(())
}
