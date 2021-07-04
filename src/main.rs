use clap::{App, load_yaml};


#[derive(Debug)]
struct NewCommand {
    name: Option<String>,
    namespace: Option<String>,
    tags: Option<Vec<String>>,
    command: Option<String>
}

impl NewCommand {
    pub fn default() -> NewCommand {
        NewCommand {
            name: None,
            namespace: None,
            tags: None,
            command: None
        }
    }
    pub fn is_complete(&self) -> bool {
        if self.name.is_none() ||
            self.namespace.is_none() ||
            self.tags.is_none() ||
            self.command.is_none() {
                return false;
        }
        true
    }
}
fn main() {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("resources/cli.yaml");
    let matches = App::from(yaml).get_matches();
    println!("{:?}", matches);

    if let Some(ref matches) = matches.subcommand_matches("new") {
        // "$ hoard new" was run
        let mut new_command = NewCommand::default();
        if matches.is_present("name") {
            // "$ hoard test -d" was run
            new_command.name = Some("abab".to_owned());
        }
        if matches.is_present("namespace") {
            // "$ hoard test -d" was run
            new_command.namespace = Some("abab".to_owned());
        }
        if matches.is_present("tags") {
            // "$ hoard test -d" was run
            new_command.tags = Some(vec!["abab".to_owned()]);
        }
        if matches.is_present("command") {
            // "$ hoard test -d" was run
            new_command.command = Some("abab".to_owned());
        }
        println!("{:?}",new_command);
        if !new_command.is_complete() {
            println!("Start interactive dialogue to fillout missing arguments");
        }
        println!("command should be complete, save it now");
    }
}