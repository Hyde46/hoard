mod command;
use clap::{load_yaml, App};
use command::{command::Command, new_command::NewCommand};

fn main() {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("resources/cli.yaml");
    let matches = App::from(yaml).get_matches();

    if let Some(ref matches) = matches.subcommand_matches("new") {
        // "$ hoard new" was run
        let new_command = NewCommand::new(matches);
        println!("{:?}", new_command);
        if !new_command.is_complete() {
            println!("Start interactive dialogue to fillout missing arguments");
        }
        println!("command should be complete, save it now");
    }
}
