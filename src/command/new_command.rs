use super::command;

#[derive(Debug)]
pub struct CommandTrove {
    version: String,
    commands: Option<Vec<NewCommand>>,
}
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

        if let Some(n) = matches.value_of("name") {
            // TODO: some name validation for when we have it
            new_command.name = Some(n.to_string());
        }
        // Defaults to 'default' namespace
        if let Some(ns) = matches.value_of("namespace") {
            // TODO: some validation for when we have it
            new_command.namespace = Some(ns.to_string());
        }
        // "$ hoard test -t" was run
        // Expects comma seperated tags
        if let Some(tags) = matches.value_of("tags") {
            new_command.tags = Some(tags.split(',').map(|s| s.to_string()).collect());
        }
        if let Some(c) = matches.value_of("command") {
            // TODO: some validation for when we have it
            new_command.command = Some(c.to_string());
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
