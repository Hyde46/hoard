use crate::core::parameters::Parameterized;
use crate::core::HoardCmd;
use crate::gui::commands_gui::State;
use termion::event::Key;

pub fn key_handler(input: Key, app: &mut State) -> Option<HoardCmd> {
    match input {
        // Quit command
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        Key::Char('\n') => {
            let command = app.selected_command.clone().unwrap();

            let mut safe_parameter = app.input.clone();
            safe_parameter = safe_parameter.replace(&app.parameter_token, "\u{E000}");
            if !app.parameter_ending_token.is_empty() {
                safe_parameter = safe_parameter.replace(&app.parameter_ending_token, "\u{E001}");
            }

            let replaced_command = command.replace_parameter(
                &app.parameter_token,
                &app.parameter_ending_token,
                &safe_parameter,
            );

            app.input = String::new();

            if replaced_command.get_parameter_count(&app.parameter_token) == 0 {
                let mut final_command = replaced_command
                    .cleanup_escapes(&app.parameter_token, &app.parameter_ending_token);

                let mut restored_cmd = final_command.command.clone();
                restored_cmd = restored_cmd.replace('\u{E000}', &app.parameter_token);
                if !app.parameter_ending_token.is_empty() {
                    restored_cmd = restored_cmd.replace('\u{E001}', &app.parameter_ending_token);
                }
                final_command.command = restored_cmd;

                return Some(final_command);
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
