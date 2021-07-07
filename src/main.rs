extern crate serde;
extern crate serde_yaml;

mod command;
mod config;
mod hoard;

use hoard::Hoard;
fn main() {
    Hoard::new()
        .with_config(None)
        .start()
        .expect("Unable to start hoard");
}
