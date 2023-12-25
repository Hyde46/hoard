use crate::command::HoardCommand;
use crate::gui::commands_gui::{ControlState, State};
use termion::event::Key;

#[allow(clippy::too_many_lines)]
pub fn key_handler(
    input: Key,
    state: &mut State,
) -> Option<HoardCommand> {
    match input {
        Key::Esc => {
            // Definitely exit program
            state.control_state = ControlState::Search;
            state.query_gpt = false;
            None
        }
        _ => None,
    }
}