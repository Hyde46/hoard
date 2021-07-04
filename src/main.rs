use clap::{App, load_yaml};

fn main() {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("resources/cli.yaml");
    let matches = App::from(yaml).get_matches();
    println!("{:?}", matches)
    // Same as previous examples...
}