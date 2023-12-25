#![warn(clippy::pedantic, clippy::nursery)]

extern crate serde_yaml;

#[macro_use]
extern crate prettytable;
extern crate dotenv;
extern crate rand;

extern crate array_tool;

mod cli_commands;
mod config;
mod core;
mod filter;
mod gpt;
mod gui;
mod hoard;
mod sync_models;
mod util;
use hoard::Hoard;

#[tokio::main]
async fn main() {
    let (command, is_autocomplete) = Hoard::default()
        .with_config(None)
        .load_trove()
        .start()
        .await;
    if is_autocomplete {
        eprintln!("{}", command.trim());
    } else {
        println!("{}", command.trim());
    }
}
