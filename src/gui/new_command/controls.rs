use crate::command::hoard_command::{string_to_tags, HoardCommand};
use crate::command::trove::CommandTrove;
use crate::gui::commands_gui::{DrawState, EditSelection, State};
use termion::event::Key;

pub fn key_handler(input: Key, app: &mut State, default_namespace: &str) -> Option<HoardCommand> {
    // Make sure there is an empty command set
    if app.new_command.is_none() {
        app.new_command = Some(HoardCommand::default());
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
                    let (_, msg) = HoardCommand::is_command_valid(&parameter);
                    command.command = parameter;
                    msg
                }
                EditSelection::Name => {
                    let (_, mut msg) = HoardCommand::is_name_valid(&parameter);
                    command.name = parameter;
                    let trove = CommandTrove::from_commands(&app.commands);
                    if trove.check_name_collision(&command).is_some() {
                        msg = String::from("Command with that name already exists in another namespace");
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
                    command.description = Some(parameter);
                    String::new()
                }
                EditSelection::Tags => {
                    let (is_valid, msg) = HoardCommand::are_tags_valid(&parameter);
                    if is_valid {
                        command.tags = Some(string_to_tags(&parameter));
                    }
                    msg
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
