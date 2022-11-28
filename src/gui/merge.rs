use crate::gui::prompts::prompt_select_with_options;
use enum_iterator::{all, Sequence};

#[derive(Sequence)]
pub enum ConflictResolve {
    /// Describes the mode of how to handle a merge conflict by adding in a command to a trove with a name that is already present

    /// Replace the old command with the new one
    Replace,
    /// Keep the old command and discard the new one
    Keep,
    /// Find a new name for the new command and keep the old one
    New,
}

impl ConflictResolve {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Replace => "Replace your local command with the new one",
            Self::Keep => "Keep your local command and ignore the new one",
            Self::New => "Keep both, but choose a new name",
        }
    }
}

pub fn with_conflict_resolve_prompt(
    name: &str,
    namespace: &str,
    command_string: &str,
    colliding_command_string: &str,
) -> ConflictResolve {
    let conflict_prompt = format!(
        "You already have a command with the name: {name} in namespace: {namespace}\nYour local command: {colliding_command_string}\nIncoming command: {command_string}\nWhat do you want to do?"
    );
    let conflict_modes = all::<ConflictResolve>().collect::<Vec<_>>();
    let items: Vec<&str> = conflict_modes.iter().map(ConflictResolve::as_str).collect();
    let selection = prompt_select_with_options(&conflict_prompt, &items);
    conflict_modes.into_iter().nth(selection).unwrap()
}
