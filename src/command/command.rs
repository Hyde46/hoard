use super::argument::Argument;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub arguments: Vec<Argument>,
    is_complete: bool,
}

impl Command {
    pub fn new(name: String, arguments: Vec<Argument>) -> Self {
        Self {
            name,
            arguments,
            is_complete: true,
        }
    }

    pub fn populate(&mut self, matches: &&clap::ArgMatches) -> &mut Command {
        for argument in self.arguments.iter_mut() {
            if let Some(n) = matches.value_of(argument.name()) {
                argument.set_value(Some(n.to_string()));
                argument.validate()
            } else {
                self.is_complete = false;
            }
        }
        self
    }

    pub fn complete_interactive(&mut self) -> &mut Command {
        for argument in self.arguments.iter_mut() {
            if argument.value().is_none() {
                argument.interactive();
            }
        }
        self
    }
}
