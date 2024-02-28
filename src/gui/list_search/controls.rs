use crate::core::parameters::Parameterized;
use crate::core::HoardCmd;
use crate::gui::commands_gui::{ControlState, DrawState, EditSelection, State};
use termion::event::Key;

#[allow(clippy::too_many_lines)]
pub fn key_handler(
    input: Key,
    state: &mut State,
    trove_commands: &[HoardCmd],
    namespace_tabs: &[&str],
) -> Option<HoardCmd> {
    match input {
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            // Definitely exit program
            state.control = ControlState::Search;
            state.should_exit = true;
            None
        }
        // Show help
        Key::F(1) => {
            state.draw = DrawState::Help;
            None
        }
        // Show help
        Key::Ctrl('w') => {
            state.draw = DrawState::Create;
            state.edit_selection = EditSelection::Command;
            state.new_command = Some(HoardCmd::default());
            None
        }
        // Enter GPT mode
        Key::Ctrl('a') => {
            // Same drawing state, only update how control works
            state.draw = DrawState::Search;
            if state.openai_key_set {
                state.control = ControlState::Gpt;
            } else {
                state.control = ControlState::KeyNotSet;
                state.query_gpt = true;
            }
            state.new_command = Some(HoardCmd::default());
            None
        }
        // Switch to edit command mode
        Key::Ctrl('e') | Key::Char('\t') => {
            let selected_command = state
                .commands
                .clone()
                .get(
                    state
                        .command_list
                        .selected()
                        .expect("there is always a selected command"),
                )
                .expect("exists")
                .clone();
            state.control = ControlState::Edit;
            state.selected_command = Some(selected_command);
            state.update_string_to_edit();
            None
        }
        // Switch namespace
        Key::Left | Key::Ctrl('h') => {
            if let Some(selected) = state.namespace_tab.selected() {
                let new_selected_tab = previous_index(selected, namespace_tabs.len());
                switch_namespace(state, new_selected_tab, namespace_tabs, trove_commands);
            }
            None
        }
        Key::Right | Key::Ctrl('l') => {
            if let Some(selected) = state.namespace_tab.selected() {
                let new_selected_tab = next_index(selected, namespace_tabs.len());
                switch_namespace(state, new_selected_tab, namespace_tabs, trove_commands);
            }
            None
        }
        // Switch command
        Key::Up | Key::Ctrl('y' | 'p') => {
            if !state.commands.is_empty() {
                if let Some(selected) = state.command_list.selected() {
                    let new_selected = previous_index(selected, state.commands.len());
                    state.command_list.select(Some(new_selected));
                }
            }
            None
        }
        Key::Down | Key::Ctrl('.' | 'n') => {
            if !state.commands.is_empty() {
                if let Some(selected) = state.command_list.selected() {
                    let new_selected = next_index(selected, state.commands.len());
                    state.command_list.select(Some(new_selected));
                }
            }
            None
        }
        Key::Ctrl('x') => {
            if state.commands.is_empty() {
                return None;
            }
            let selected_command = state
                .commands
                .clone()
                .get(
                    state
                        .command_list
                        .selected()
                        .expect("there is always a selected command"),
                )
                .expect("exists")
                .clone();
            state.should_delete = true;
            Some(selected_command)
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
                        .command_list
                        .selected()
                        .expect("there is always a selected command"),
                )
                .expect("exists")
                .clone();
            // Check if parameters need to be supplied
            if selected_command.get_parameter_count(&state.parameter_token) > 0 {
                // Set next state to draw
                state.draw = DrawState::ParameterInput;
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
    commands: &[HoardCmd],
) {
    state.namespace_tab.select(Some(index_to_select));

    let selected_namespace = namespaces
        .get(index_to_select)
        .expect("Always a tab selected");

    apply_search(state, commands, selected_namespace);

    let new_selected_command = if state.commands.is_empty() {
        0
    } else {
        state.commands.len() - 1
    };

    state.command_list.select(Some(new_selected_command));
}

fn apply_search(state: &mut State, all_commands: &[HoardCmd], selected_tab: &str) {
    let query_term = &state.input[..];
    state.commands = all_commands
        .iter()
        .filter(|&c| {
            (c.name.contains(query_term)
                || c.namespace.contains(query_term)
                || c.get_tags_as_string().contains(query_term)
                || c.command.contains(query_term)
                || c.description.contains(query_term))
                && (c.namespace.clone() == *selected_tab || selected_tab == "All")
        })
        .cloned()
        .collect();
    state.commands.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
}

fn apply_filter(state: &mut State, namespaces: &[&str], commands: &[HoardCmd]) {
    let selected_tab = namespaces
        .get(
            state
                .namespace_tab
                .selected()
                .expect("Always a namespace selected"),
        )
        .expect("Always a tab selected");
    apply_search(state, commands, selected_tab);
}

#[cfg(test)]
mod test_controls {
    use super::*;
    use ratatui::widgets::ListState;

    const DEFAULT_NAMESPACE: &str = "default";

    fn create_command(name: &str, command: &str, namespace: &str) -> HoardCmd {
        HoardCmd::default()
            .with_name(name)
            .with_command(command)
            .with_namespace(namespace)
    }

    fn create_state(commands: Vec<HoardCmd>) -> State {
        let mut state = State {
            input: String::new(),
            commands,
            command_list: ListState::default(),
            namespace_tab: ListState::default(),
            should_exit: false,
            should_delete: false,
            draw: DrawState::Search,
            control: ControlState::Search,
            new_command: None,
            edit_selection: crate::gui::commands_gui::EditSelection::Command,
            string_to_edit: String::new(),
            parameter_token: "#".to_string(),
            parameter_ending_token: "!".to_string(),
            selected_command: None,
            provided_parameter_count: 0,
            error_message: String::new(),
            query_gpt: false,
            buffered_tick: false,
            popup_message: State::get_default_popupmsg(),
            openai_key_set: false,
        };

        state.command_list.select(Some(0));
        state.namespace_tab.select(Some(0));

        state
    }

    fn test_change_command(key: Key, initial_index: usize, expected_index: usize) {
        let namespaces = vec![DEFAULT_NAMESPACE];
        let cmd1 = create_command("first", "", DEFAULT_NAMESPACE);
        let cmd2 = create_command("second", "", DEFAULT_NAMESPACE);
        let cmd3 = create_command("third", "", DEFAULT_NAMESPACE);
        let mut state = create_state(vec![cmd1, cmd2, cmd3]);
        state.command_list.select(Some(initial_index));

        let commands = state.commands.clone();
        key_handler(key, &mut state, &commands, &namespaces);
        let new_selected_index = state.command_list.selected();

        assert_eq!(expected_index, new_selected_index.unwrap());
    }

    fn test_change_namespace(key: Key, initial_index: usize, expected_index: usize) {
        let namespaces = vec!["first", "second", "third"];
        let mut state = create_state(vec![]);
        state.namespace_tab.select(Some(initial_index));

        let commands = state.commands.clone();
        key_handler(key, &mut state, &commands, &namespaces);
        let new_selected_index = state.namespace_tab.selected();

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
        let cmd1 = create_command("first_command", "", namespace1);
        let cmd2 = create_command(cmd2_name, "", namespace2);
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
        let cmd1 = create_command("first_command", "", namespace2);
        let cmd2 = create_command("second_command", "", namespace2);
        let mut state = create_state(vec![cmd1, cmd2]);

        let commands = state.commands.clone();
        key_handler(Key::Right, &mut state, &commands, &all_namespaces);
        let selected_command_index = state.command_list.selected().unwrap();

        assert_eq!(expected_command_index, selected_command_index);
    }

    #[test]
    fn pick_command_without_params() {
        let namespaces = vec![DEFAULT_NAMESPACE];
        let expected_command = "second_command";
        let command_index = 1;
        let cmd1 = create_command("first_command", "", DEFAULT_NAMESPACE);
        let cmd2 = create_command(expected_command, "", DEFAULT_NAMESPACE);

        let mut state = create_state(vec![cmd1, cmd2]);
        state.command_list.select(Some(command_index));

        let commands = state.commands.clone();
        let actual_command =
            key_handler(Key::Char('\n'), &mut state, &commands, &namespaces).unwrap();

        assert_eq!(expected_command, actual_command.name);
    }

    #[test]
    fn pick_command_with_params() {
        let namespaces = vec![DEFAULT_NAMESPACE];
        let cmd = create_command("First", "first_command #", DEFAULT_NAMESPACE);

        let mut state = create_state(vec![cmd]);
        let commands = state.commands.clone();
        key_handler(Key::Char('\n'), &mut state, &commands, &namespaces);

        assert_eq!(DrawState::ParameterInput, state.draw);
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

        assert_eq!(DrawState::Help, state.draw);
    }
}
