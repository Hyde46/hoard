use super::command;

#[derive(Debug)]
pub struct NewCommand {
    name: Option<String>,
    namespace: Option<String>,
    tags: Option<Vec<String>>,
    command: Option<String>,
}

impl command::Command for NewCommand {
    fn new(matches: &&clap::ArgMatches) -> NewCommand {
        let mut new_command = NewCommand::default();
        if matches.is_present("name") {
            // "$ hoard test -n" was run
            new_command.name = Some("abab".to_owned());
        }
        if matches.is_present("namespace") {
            // "$ hoard test -s" was run
            new_command.namespace = Some("abab".to_owned());
        }
        if matches.is_present("tags") {
            // "$ hoard test -t" was run
            new_command.tags = Some(vec!["abab".to_owned()]);
        }
        if matches.is_present("command") {
            // "$ hoard test -c" was run
            new_command.command = Some("abab".to_owned());
        }
        new_command
    }

    fn default() -> Self {
        NewCommand {
            name: None,
            namespace: None,
            tags: None,
            command: None,
        }
    }

    fn is_complete(&self) -> bool {
        if self.name.is_none()
            || self.namespace.is_none()
            || self.tags.is_none()
            || self.command.is_none()
        {
            return false;
        }
        true
    }
}
