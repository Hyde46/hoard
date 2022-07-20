use crate::command::hoard_command::{HoardCommand, Parameterized};
use crate::gui::commands_gui::State;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn key_handler(input: KeyEvent, app: &mut State) -> Option<HoardCommand> {
    match input {
        // Quit command
        // ESC | Ctrl + c | Ctrl + d | Ctrl + g
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        }
        | KeyEvent {
            code: KeyCode::Char('c' | 'd' | 'g'),
            modifiers: KeyModifiers::CONTROL,
        } => {
            app.should_exit = true;
            None
        }
        // Enter
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
        } => {
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
        // Backspace
        KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
        } => {
            app.input.pop();
            None
        }
        // All char
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
        } => {
            app.input.push(c);
            None
        }
        _ => None,
    }
}
