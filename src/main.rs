extern crate serde;
extern crate serde_yaml;

use simple_logger::SimpleLogger;

mod command;
mod config;
mod hoard;

use hoard::Hoard;
fn main() {
    SimpleLogger::new().init().unwrap();

    Hoard::default()
        .with_config(None)
        .load_trove()
        .start()
        .expect("Unable to start hoard");
}
