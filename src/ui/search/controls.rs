use crate::core::HoardCmd;
use crate::ui::App;

use termion::event::Key;

#[allow(clippy::too_many_lines)]
pub fn draw_search_key_handler(input: Key, app: &mut App) -> Option<HoardCmd> {
    match input {
        // 
        // Exit the application
        // Does not return a command and just quits
        //
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        //
        // Remove the last typed search string character
        // 'This is a search' -> 'This is a searc'
        //
        Key::Backspace => {
            app.search_string.pop();
            app.filter_trove();
            None
        }
        //
        // Selects the currently highlighted command
        // Returns the command to the caller
        // If no command is selected, returns None
        //
        Key::Char('\n') => {
            app.commands.selected().map(|index| app.search_trove.commands[index].clone())
        }
        //
        // Switches context for currently shown collection
        // Iterates through [All, Local, <trove namespaces>]
        // All and Local are built in collections
        // Trove namespaces are user defined collections based on each command
        //
        Key::Char('\t') => {
            app.next_collection();
            None
        }
        //
        // Deletes the last word from the searchstring
        // 'This is a search' -> 'This is a'
        //
        Key::Ctrl('w') => {
            // Delete the last word from searchstring
            let mut search_string = app.search_string.split_whitespace();
            search_string.next_back();
            // Collect with spaces
            app.search_string = search_string.collect::<Vec<&str>>().join(" ");
            app.filter_trove();
            None
        }
        //
        // Deletes the entire search string
        // 'This is a search' -> ''
        //
        Key::Ctrl('u') => {
            // Deletes the entire search string
            app.search_string.clear();
            app.filter_trove();
            None
        }
        //
        // Moves the selection down by one command
        // If the selection is at the bottom, it wraps around to the top
        //
        Key::Down => {
            increment_selected_command(app);
            None
        }
        //
        // Moves the selection up by one command
        // If the selection is at the top, it wraps around to the bottom
        //
        Key::Up => {
            decrement_selected_command(app);
            None
        }
        // 
        // Adds the typed character to the search string
        // 'This is a search' -> 'This is a searchc'
        // After the search string is edited,
        // the trove is filtered to match the search string
        //
        Key::Char(c) => {
            app.search_string.push(c);
            app.filter_trove();
            None
        }
        _ => None,
    }
}

fn increment_selected_command(app: &mut App) {
    if app.search_trove.commands.is_empty() {
        return;
    }

    let current_selected = app.commands.selected().unwrap_or(0);
    let next_selected = next_index(current_selected, app.search_trove.commands.len());
    app.commands.select(Some(next_selected));
    // Update the scroll state based on how close the selected command is to the top or bottom
    let actual_position = current_selected - app.vertical_scroll;

    // If we increment at the last element, reset the scroll to 0
    if next_selected == 0 {
        app.vertical_scroll = 0;
    } else if actual_position >= app.frame_size.height as usize - 6 {
        app.vertical_scroll = app.vertical_scroll + 1;
    }
}

fn decrement_selected_command(app: &mut App) {
    if app.search_trove.commands.is_empty() {
        return;
    }
    if let Some(selected) = app.commands.selected() {
        let new_selected = previous_index(selected, app.search_trove.commands.len());
        app.commands.select(Some(new_selected));
        // If we jump to the end, just scroll all the way to the bottom
        if new_selected == app.search_trove.commands.len() - 1 {
            app.vertical_scroll = app.search_trove.commands.len().saturating_sub(1);
            return;
        }
    }

    // Update the scroll state based on how close the selected command is to the top or bottom
    let selected = app.commands.selected().unwrap_or(0);
    let max_scroll = app.search_trove.commands.len().saturating_sub(1);
    if selected < app.vertical_scroll {
        app.vertical_scroll = selected;
    } else if selected > app.vertical_scroll + max_scroll {
        app.vertical_scroll = selected.saturating_sub(max_scroll);
    }
}

pub const fn next_index(current_index: usize, collection_length: usize) -> usize {
    if current_index >= collection_length - 1 {
        0
    } else {
        current_index + 1
    }
}

pub const fn previous_index(current_index: usize, collection_length: usize) -> usize {
    if current_index == 0 {
        collection_length - 1
    } else {
        current_index - 1
    }
}
