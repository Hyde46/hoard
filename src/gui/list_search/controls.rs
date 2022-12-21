use crate::command::hoard_command::{HoardCommand, Parameterized};
use crate::gui::commands_gui::{ControlState, DrawState, EditSelection, State};
use termion::event::Key;

pub fn key_handler(
    input: Key,
    state: &mut State,
    trove_commands: &[HoardCommand],
    namespace_tabs: &[&str],
) -> Option<HoardCommand> {
    match input {
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            // Definitely exit program
            state.control_state = ControlState::Search;
            state.should_exit = true;
            None
        }
        // Show help
        Key::F(1) => {
            state.draw_state = DrawState::Help;
            None
        }
        // Switch to edit command mode
        Key::Ctrl('e') | Key::Char('\t') => {
            let selected_command = state
                .commands
                .clone()
                .get(
                    state
                        .command_list_state
                        .selected()
                        .expect("there is always a selected command"),
                )
                .expect("exists")
                .clone();
            state.control_state = ControlState::Edit;
            state.selected_command = Some(selected_command);
            state.update_string_to_edit();
            None
        }
        // Switch namespace
        Key::Left | Key::Ctrl('h') => {
            if let Some(selected) = state.namespace_tab_state.selected() {
                let new_selected_tab = previous_index(selected, namespace_tabs.len());
                switch_namespace(state, new_selected_tab, namespace_tabs, trove_commands);
            }
            None
        }
        Key::Right | Key::Ctrl('l') => {
            if let Some(selected) = state.namespace_tab_state.selected() {
                let new_selected_tab = next_index(selected, namespace_tabs.len());
                switch_namespace(state, new_selected_tab, namespace_tabs, trove_commands);
            }
            None
        }
        // Switch command
        Key::Up | Key::Ctrl('y' | 'p') => {
            if !state.commands.is_empty() {
                if let Some(selected) = state.command_list_state.selected() {
                    let new_selected = previous_index(selected, state.commands.len());
                    state.command_list_state.select(Some(new_selected));
                }
            }
            None
        }
        Key::Down | Key::Ctrl('.' | 'n') => {
            if !state.commands.is_empty() {
                if let Some(selected) = state.command_list_state.selected() {
                    let new_selected = next_index(selected, state.commands.len());
                    state.command_list_state.select(Some(new_selected));
                }
            }
            None
        }
        // Select command
        Key::Char('\n') => {
            if state.commands.is_empty() {
                state.should_exit = true;
                return None;
            }
            let selected_command = state
                .commands
                .clone()
                .get(
                    state
                        .command_list_state
                        .selected()
                        .expect("there is always a selected command"),
                )
                .expect("exists")
                .clone();

            // Check if parameters need to be supplied
            if selected_command.get_parameter_count(&state.parameter_token) > 0 {
                // Set next state to draw
                state.draw_state = DrawState::ParameterInput;
                // Save which command to replace parameters for
                state.selected_command = Some(selected_command);
                // Empty input for next screen
                state.input = String::new();
                // return None, otherwise drawing will quit
                return None;
            }
            Some(selected_command)
        }
        // Handle query input
        Key::Backspace => {
            state.input.pop();
            apply_filter(state, namespace_tabs, trove_commands);
            None
        }
        Key::Char(c) => {
            state.input.push(c);
            apply_filter(state, namespace_tabs, trove_commands);
            None
        }
        _ => None,
    }
}

const fn next_index(current_index: usize, collection_length: usize) -> usize {
    if current_index >= collection_length - 1 {
        0
    } else {
        current_index + 1
    }
}

const fn previous_index(current_index: usize, collection_length: usize) -> usize {
    if current_index > 0 {
        current_index - 1
    } else {
        collection_length - 1
    }
}

fn switch_namespace(
    state: &mut State,
    index_to_select: usize,
    namespaces: &[&str],
    commands: &[HoardCommand],
) {
    state.namespace_tab_state.select(Some(index_to_select));

    let selected_namespace = namespaces
        .get(index_to_select)
        .expect("Always a tab selected");

    apply_search(state, commands, selected_namespace);

    let new_selected_command = if state.commands.is_empty() {
        0
    } else {
        state.commands.len() - 1
    };

    state.command_list_state.select(Some(new_selected_command));
}

fn apply_search(state: &mut State, all_commands: &[HoardCommand], selected_tab: &str) {
    let query_term = &state.input[..];
    state.commands = all_commands
        .iter()
        .cloned()
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

fn apply_filter(state: &mut State, namespaces: &[&str], commands: &[HoardCommand]) {
    let selected_tab = namespaces
        .get(
            state
                .namespace_tab_state
                .selected()
                .expect("Always a namespace selected"),
        )
        .expect("Always a tab selected");
    apply_search(state, commands, selected_tab);
}

#[cfg(test)]
mod test_controls {
    use super::*;
    use tui::widgets::ListState;

    const DEFAULT_NAMESPACE: &str = "default";

    fn create_command(name: &str, namespace: &str) -> HoardCommand {
        HoardCommand {
            name: name.to_string(),
            namespace: namespace.to_string(),
            tags: None,
            command: name.to_string(),
            description: None,
        }
    }

    fn create_state(commands: Vec<HoardCommand>) -> State {
        let mut state = State {
            input: String::new(),
            commands,
            command_list_state: ListState::default(),
            namespace_tab_state: ListState::default(),
            should_exit: false,
            draw_state: DrawState::Search,
            control_state: ControlState::Search,
            edit_selection: EditSelection::Command,
            string_to_edit: String::new(),
            parameter_token: "#".to_string(),
            parameter_ending_token: "!".to_string(),
            selected_command: None,
            provided_parameter_count: 0,
        };

        state.command_list_state.select(Some(0));
        state.namespace_tab_state.select(Some(0));

        state
    }

    fn test_change_command(key: Key, initial_index: usize, expected_index: usize) {
        let namespaces = vec![DEFAULT_NAMESPACE];
        let cmd1 = create_command("first", DEFAULT_NAMESPACE);
        let cmd2 = create_command("second", DEFAULT_NAMESPACE);
        let cmd3 = create_command("third", DEFAULT_NAMESPACE);
        let mut state = create_state(vec![cmd1, cmd2, cmd3]);
        state.command_list_state.select(Some(initial_index));

        let commands = state.commands.clone();
        key_handler(key, &mut state, &commands, &namespaces);
        let new_selected_index = state.command_list_state.selected();

        assert_eq!(expected_index, new_selected_index.unwrap());
    }

    fn test_change_namespace(key: Key, initial_index: usize, expected_index: usize) {
        let namespaces = vec!["first", "second", "third"];
        let mut state = create_state(vec![]);
        state.namespace_tab_state.select(Some(initial_index));

        let commands = state.commands.clone();
        key_handler(key, &mut state, &commands, &namespaces);
        let new_selected_index = state.namespace_tab_state.selected();

        assert_eq!(expected_index, new_selected_index.unwrap());
    }

    // Commands
    #[test]
    fn next_command() {
        test_change_command(Key::Down, 0, 1);
    }

    #[test]
    fn next_command_wrap() {
        test_change_command(Key::Down, 2, 0);
    }

    #[test]
    fn previous_command() {
        test_change_command(Key::Up, 2, 1);
    }

    #[test]
    fn previous_command_wrap() {
        test_change_command(Key::Up, 0, 2);
    }

    // Namespaces
    #[test]
    fn next_namespace() {
        test_change_namespace(Key::Right, 1, 2);
    }

    #[test]
    fn next_namespace_wrap() {
        test_change_namespace(Key::Right, 2, 0);
    }

    #[test]
    fn previous_namespace() {
        test_change_namespace(Key::Left, 2, 1);
    }

    #[test]
    fn previous_namespace_wrap() {
        test_change_namespace(Key::Left, 0, 2);
    }

    #[test]
    fn filter_commands_when_namespace_changed() {
        let namespace1 = "first_namespace";
        let namespace2 = "second_namespace";
        let all_namespaces = vec![namespace1, namespace2];

        let cmd2_name = "second_command";
        let cmd1 = create_command("first_command", namespace1);
        let cmd2 = create_command(cmd2_name, namespace2);
        let mut state = create_state(vec![cmd1, cmd2]);

        let commands = state.commands.clone();
        key_handler(Key::Right, &mut state, &commands, &all_namespaces);
        let filtered_commands = state.commands;

        assert_eq!(1, filtered_commands.len());
        assert_eq!(cmd2_name, filtered_commands.first().unwrap().name);
    }

    #[test]
    fn select_last_command_when_namespace_changed() {
        let namespace1 = "first_namespace";
        let namespace2 = "second_namespace";
        let all_namespaces = vec![namespace1, namespace2];

        let expected_command_index = 1;
        let cmd1 = create_command("first_command", namespace2);
        let cmd2 = create_command("second_command", namespace2);
        let mut state = create_state(vec![cmd1, cmd2]);

        let commands = state.commands.clone();
        key_handler(Key::Right, &mut state, &commands, &all_namespaces);
        let selected_command_index = state.command_list_state.selected().unwrap();

        assert_eq!(expected_command_index, selected_command_index);
    }

    #[test]
    fn pick_command_without_params() {
        let namespaces = vec![DEFAULT_NAMESPACE];
        let expected_command = "second_command";
        let command_index = 1;
        let cmd1 = create_command("first_command", DEFAULT_NAMESPACE);
        let cmd2 = create_command(expected_command, DEFAULT_NAMESPACE);

        let mut state = create_state(vec![cmd1, cmd2]);
        state.command_list_state.select(Some(command_index));

        let commands = state.commands.clone();
        let actual_command =
            key_handler(Key::Char('\n'), &mut state, &commands, &namespaces).unwrap();

        assert_eq!(expected_command, actual_command.command);
    }

    #[test]
    fn pick_command_with_params() {
        let namespaces = vec![DEFAULT_NAMESPACE];
        let cmd = create_command("first_command #", DEFAULT_NAMESPACE);

        let mut state = create_state(vec![cmd]);
        let commands = state.commands.clone();
        key_handler(Key::Char('\n'), &mut state, &commands, &namespaces);

        assert_eq!(DrawState::ParameterInput, state.draw_state);
    }

    #[test]
    fn quit_on_nothing_to_pick() {
        let mut state = create_state(vec![]);

        key_handler(Key::Char('\n'), &mut state, &[], &[]);

        assert!(state.should_exit);
    }

    #[test]
    fn quit() {
        let mut state = create_state(vec![]);

        key_handler(Key::Esc, &mut state, &[], &[]);

        assert!(state.should_exit);
    }

    #[test]
    fn show_help() {
        let mut state = create_state(vec![]);

        key_handler(Key::F(1), &mut state, &[], &[]);

        assert_eq!(DrawState::Help, state.draw_state);
    }
}
