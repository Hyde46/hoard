mod command;
mod hoard;

use hoard::Hoard;

extern crate serde;
extern crate serde_yaml;

fn main() {
    Hoard::default().start().expect("Unable to start hoard");
}
