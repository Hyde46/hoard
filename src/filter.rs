use crate::core::HoardCmd;
use crate::core::trove::Trove;

pub fn query_trove(trove: &Trove, query_term: &str) -> Trove {
    // Filter out commands of `trove` based on `query_string`
    // Construct QueryString object from &str object to validate / extract special parameters
    let commands: Vec<HoardCmd> = trove
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
    Trove::from_commands(&commands)
}
