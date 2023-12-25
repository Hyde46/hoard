use crate::core::HoardCmd;
use crate::gui::commands_gui::{ControlState, DrawState, State};
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
        // Show help
        Key::F(1) => {
            state.draw_state = DrawState::Help;
            None
        }
        // Select command
        Key::Char('\n') => {
            state.query_gpt = true;
            None
        }
        // Handle query input
        Key::Backspace => {
            state.input.pop();
            None
        }
        Key::Char(c) => {
            state.input.push(c);
            None
        }
        _ => None,
    }
}