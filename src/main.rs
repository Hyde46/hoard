extern crate serde;
extern crate serde_yaml;

mod command;
mod config;
mod hoard;
use hoard::Hoard;
fn main() {
    Hoard::default()
        .with_config(None)
        .load_trove()
        .start()
        .expect("Unable to start hoard");
}
