use crate::core::HoardCmd;
use crate::ui::App;

use termion::event::Key;


#[allow(clippy::too_many_lines)]
pub fn key_handler(
    input: Key,
    app: &mut App,
    trove_commands: &[HoardCmd],
    namespace_tabs: &[&str],
) -> Option<HoardCmd> {
    match input {
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        _ => None,
    }

}
