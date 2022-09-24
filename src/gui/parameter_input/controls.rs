use crate::command::hoard_command::{HoardCommand, Parameterized};
use crate::gui::commands_gui::State;
use termion::event::Key;

pub fn key_handler(input: Key, app: &mut State) -> Option<HoardCommand> {
    match input {
        // Quit command
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        Key::Char('\n') => {
            let command = app.selected_command.clone().unwrap();
            let parameter = app.input.clone();
            let replaced_command = command.replace_parameter(&app.parameter_token, parameter);
            app.input = String::from("");
            if replaced_command.get_parameter_count(&app.parameter_token) == 0 {
                return Some(replaced_command);
            }
            app.selected_command = Some(replaced_command);
            app.provided_parameter_count += 1;
            None
        }
        // Handle query input
        Key::Backspace => {
            app.input.pop();
            None
        }
        Key::Char(c) => {
            app.input.push(c);
            None
        }
        _ => None,
    }
}
