use crate::core::HoardCmd;
use crate::ui::App;

use termion::event::Key;


#[allow(clippy::too_many_lines)]
pub fn draw_search_key_handler(
    input: Key,
    app: &mut App,
) -> Option<HoardCmd> {
    match input {
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        Key::Char(c) => {
            app.search_string.push(c);
            None
        }
        Key::Backspace => {
            app.search_string.pop();
            None
        }
        Key::Ctrl('w') => {
            // Delete the last word from searchstring
            let mut search_string = app.search_string.split_whitespace();
            search_string.next_back();
            // Collect with spaces
            app.search_string = search_string.collect::<Vec<&str>>().join(" ");
            None
        }
        Key::Ctrl('u') => {
            // Deletes the entire search string
            app.search_string.clear();
            None
        }
        _ => None,
    }

}
