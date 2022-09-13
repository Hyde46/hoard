use crate::command::hoard_command::{HoardCommand, Parameterized};
use crate::command::trove::CommandTrove;


//fn filter_trove(state: &mut State, commands: &[HoardCommand], selected_tab: &str) {
fn filter_trove(trove: &CommandTrove, query_string: &str) -> CommandTrove {
    // Filter out commands of `trove` based on `query_string`
    // Construct QueryString object from &str object to validate / extract special parameters
    CommandTrove::from_commands(trove.commands
        .to_owned()
        .into_iter()
        .filter(|c| {
            (c.name.contains(query_term)
                || c.namespace.contains(query_term)
                || c.tags_as_string().contains(query_term)
                || c.command.contains(query_term)
                || c.description
                    .clone()
                    .unwrap_or_default()
                    .contains(query_term))
                && (c.namespace.clone() == *selected_tab || selected_tab == "All")
        })
        .collect())
}