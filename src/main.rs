extern crate crossterm;
extern crate termion;
extern crate serde;
extern crate serde_yaml;

#[macro_use]
extern crate prettytable;

mod command;
mod config;
mod gui;
mod hoard;
use hoard::Hoard;
fn main() {
    Hoard::default()
        .with_config(None)
        .load_trove()
        .start()
        .expect("Unable to start hoard");
}
