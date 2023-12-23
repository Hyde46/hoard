use crate::command::hoard_command::HoardCommand;
use crate::command::trove::CommandTrove;

pub fn query_trove(trove: &CommandTrove, query_term: &str) -> CommandTrove {
    // Filter out commands of `trove` based on `query_string`
    // Construct QueryString object from &str object to validate / extract special parameters
    let commands: Vec<HoardCommand> = trove
        .commands
        .clone()
        .into_iter()
        .filter(|c| {
            c.name.contains(query_term)
                || c.namespace.contains(query_term)
                || c.get_tags_as_string().contains(query_term)
                || c.command.contains(query_term)
                || c.description
                    .contains(query_term)
        })
        .collect();
    CommandTrove::from_commands(&commands)
}
