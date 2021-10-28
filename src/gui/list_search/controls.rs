use crate::command::hoard_command::{HoardCommand, Parameterized};
use crate::gui::commands_gui::{DrawState, State};
use termion::event::Key;

#[allow(clippy::too_many_lines)]
pub fn key_handler(
    input: Key,
    app: &mut State,
    trove_commands: &[HoardCommand],
    namespace_tabs: &[&str],
) -> Option<HoardCommand> {
    match input {
        // Quit command
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        Key::F(1) => {
            app.draw_state = DrawState::Help;
            None
        }
        // Switch namespace
        Key::Left | Key::Ctrl('h') => {
            if let Some(selected) = app.namespace_tab_state.selected() {
                let amount_ns = namespace_tabs.len();
                if selected > 0 {
                    app.namespace_tab_state.select(Some(selected - 1));
                } else {
                    app.namespace_tab_state.select(Some(amount_ns - 1));
                }
                let selected_tab = namespace_tabs
                    .get(
                        app.namespace_tab_state
                            .selected()
                            .expect("Always a namespace selected"),
                    )
                    .expect("Always a tab selected");
                apply_search(app, trove_commands, selected_tab);
                let new_selection = if app.commands.is_empty() {
                    0
                } else {
                    app.commands.len() - 1
                };
                app.command_list_state.select(Some(new_selection));
            }
            None
        }
        Key::Right | Key::Ctrl('l') => {
            if let Some(selected) = app.namespace_tab_state.selected() {
                let amount_ns = namespace_tabs.len();
                if selected >= amount_ns - 1 {
                    app.namespace_tab_state.select(Some(0));
                } else {
                    app.namespace_tab_state.select(Some(selected + 1));
                }
                let selected_tab = namespace_tabs
                    .get(
                        app.namespace_tab_state
                            .selected()
                            .expect("Always a namespace selected"),
                    )
                    .expect("Always a tab selected");
                apply_search(app, trove_commands, selected_tab);
                let new_selection = if app.commands.is_empty() {
                    0
                } else {
                    app.commands.len() - 1
                };
                app.command_list_state.select(Some(new_selection));
            }
            None
        }
        // Switch command
        Key::Up | Key::Ctrl('y' | 'p') => {
            if !app.commands.is_empty() {
                if let Some(selected) = app.command_list_state.selected() {
                    let amount_commands = app.commands.clone().len();
                    if selected > 0 {
                        app.command_list_state.select(Some(selected - 1));
                    } else {
                        app.command_list_state.select(Some(amount_commands - 1));
                    }
                }
            }
            None
        }
        Key::Down | Key::Ctrl('.' | 'n') => {
            if !app.commands.is_empty() {
                if let Some(selected) = app.command_list_state.selected() {
                    let amount_commands = app.commands.clone().len();
                    if selected >= amount_commands - 1 {
                        app.command_list_state.select(Some(0));
                    } else {
                        app.command_list_state.select(Some(selected + 1));
                    }
                }
            }
            None
        }
        // Select command
        Key::Char('\n') => {
            if app.commands.is_empty() {
                app.should_exit = true;
                return None;
            }
            let selected_command = app
                .commands
                .clone()
                .get(
                    app.command_list_state
                        .selected()
                        .expect("there is always a selected command"),
                )
                .expect("exists")
                .clone();

            // Check if parameters need to be supplied
            if selected_command.get_parameter_count(&app.parameter_token) > 0 {
                // Set next state to draw
                app.draw_state = DrawState::ParameterInput;
                // Save which command to replace parameters for
                app.selected_command = Some(selected_command);
                // Empty input for next screen
                app.input = "".to_string();
                // return None, otherwise drawing will quit
                return None;
            }
            Some(selected_command)
        }
        // Handle query input
        Key::Backspace => {
            app.input.pop();
            let selected_tab = namespace_tabs
                .get(
                    app.namespace_tab_state
                        .selected()
                        .expect("Always a namespace selected"),
                )
                .expect("Always a tab selected");
            apply_search(app, trove_commands, selected_tab);
            None
        }
        Key::Char(c) => {
            app.input.push(c);
            let selected_tab = namespace_tabs
                .get(
                    app.namespace_tab_state
                        .selected()
                        .expect("Always a namespace selected"),
                )
                .expect("Always a tab selected");
            apply_search(app, trove_commands, selected_tab);
            None
        }
        _ => None,
    }
}

#[allow(clippy::ptr_arg)]
fn apply_search(app: &mut State, all_commands: &[HoardCommand], selected_tab: &str) {
    let query_term = &app.input[..];
    app.commands = all_commands
        .to_owned()
        .into_iter()
        .filter(|c| {
            (c.name.contains(query_term)
                || c.namespace.contains(query_term)
                || c.tags_as_string().contains(query_term)
                || c.command.contains(query_term)
                || c.description
                    .clone()
                    .unwrap_or_default()
                    .contains(query_term))
                && (c.namespace.clone() == *selected_tab || selected_tab == "All")
        })
        .collect();
}
