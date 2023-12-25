use crate::command::HoardCmd;
use crate::gui::commands_gui::{ControlState, State};
use termion::event::Key;

#[allow(clippy::too_many_lines)]
pub fn key_handler(
    input: Key,
    state: &mut State,
) -> Option<HoardCmd> {
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