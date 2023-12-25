use crate::command::{string_to_tags, HoardCmd};
use crate::command::trove::Trove;
use crate::gui::commands_gui::{DrawState, EditSelection, State};
use termion::event::Key;

pub fn key_handler(input: Key, app: &mut State, default_namespace: &str) -> Option<HoardCmd> {
    // Make sure there is an empty command set
    if app.new_command.is_none() {
        app.new_command = Some(HoardCmd::default());
    }
    match input {
        Key::Esc => {
            app.draw_state = DrawState::Search;
            app.new_command = None;
            app.edit_selection = EditSelection::Command;
            None
        }
        // Quit command
        Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            app.new_command = None;
            app.edit_selection = EditSelection::Command;
            None
        }
        Key::Char('\n') => {
            let mut command = app.new_command.clone().unwrap();
            let parameter = app.input.clone();
            app.error_message = match app.edit_selection {
                EditSelection::Command => {
                    command.command = parameter.clone();
                    // when HoardCmd::is_command_valid(&parameter) returns an error, read out the rror and return
                    // that else return empty string
                    match HoardCmd::is_command_valid(&parameter) {
                        Ok(()) => String::new(),
                        Err(error) => error.to_string(),
                    }
                }
                EditSelection::Name => {
                    command.name = parameter.clone();
                    let mut msg = match HoardCmd::is_name_valid(&parameter) {
                        Ok(()) => String::new(),
                        Err(error) => error.to_string(),
                    };
                    let trove = Trove::from_commands(&app.commands);
                    if trove.get_command_collision(&command).is_some() {
                        msg = String::from(
                            "Command with that name already exists in another namespace",
                        );
                    }
                    msg
                }
                EditSelection::Namespace => {
                    if parameter.is_empty() {
                        command.namespace = default_namespace.into();
                    } else {
                        command.namespace = parameter;
                    }
                    String::new()
                }
                EditSelection::Description => {
                    command.description = parameter;
                    String::new()
                }
                EditSelection::Tags => {
                    match HoardCmd::are_tags_valid(&parameter) {
                        Ok(()) => {command.tags = string_to_tags(&parameter); String::new()},
                        Err(e) => {
                            app.error_message = e.to_string();
                            e.to_string()
                        }
                        
                    }
                }
            };
            app.input = String::new();
            if !app.error_message.is_empty() {
                return None;
            }
            app.edit_selection = app.edit_selection.edit_next();
            if app.edit_selection == EditSelection::Command {
                return Some(command);
            }
            app.new_command = Some(command);
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
