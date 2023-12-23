use crate::command::hoard_command::{string_to_tags, HoardCommand};
use crate::gui::commands_gui::{ControlState, EditSelection, State};
use termion::event::Key;

pub fn key_handler(input: Key, state: &mut State) -> Option<HoardCommand> {
    match input {
        // Quit command
        Key::Esc => {
            // Only exit the edit mode
            state.control_state = ControlState::Search;
            None
        }
        Key::Char('\n') => {
            let mut edited_command = state.selected_command.clone().unwrap();
            let new_string = state.string_to_edit.clone();
            match state.edit_selection {
                EditSelection::Description => edited_command.description = new_string,
                EditSelection::Command => edited_command.command = new_string,
                EditSelection::Tags => edited_command.tags = string_to_tags(&new_string),
                EditSelection::Name | EditSelection::Namespace => (),
            };
            Some(edited_command)
        }
        Key::Char('\t') => {
            state.edit_selection = state.edit_selection.next();
            state.update_string_to_edit();
            None
        }
        Key::Ctrl('c' | 'd' | 'g') => {
            // Definitely exit program
            state.should_exit = true;
            None
        }
        // Handle query input
        Key::Backspace => {
            state.string_to_edit.pop();
            None
        }
        Key::Char(c) => {
            state.string_to_edit.push(c);
            None
        }
        _ => None,
    }
}
