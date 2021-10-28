use crate::command::hoard_command::{HoardCommand, Parameterized};
use crate::config::HoardConfig;
use crate::gui::commands_gui::State;
use crate::util::translate_number_to_nth;
use termion::event::Key;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Paragraph, Wrap};
use tui::Terminal;

#[allow(clippy::too_many_lines)]
pub fn draw(
    app_state: &mut State,
    config: &HoardConfig,
    terminal: &mut Terminal<
        TermionBackend<AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>>,
    >,
) -> Result<(), eyre::Error> {
    terminal.draw(|rect| {
        let size = rect.size();
        // Overlay
        let overlay_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(40),
                ]
                .as_ref(),
            )
            .split(size);

        let mut query_string = config.query_prefix.clone();
        query_string.push_str(&app_state.input.clone()[..]);
        let title_string = format!(
            "Provide {} parameter",
            translate_number_to_nth(app_state.provided_parameter_count)
        );
        let input = Paragraph::new(query_string)
            .style(Style::default().fg(Color::Rgb(
                config.primary_color.unwrap().0,
                config.primary_color.unwrap().1,
                config.primary_color.unwrap().2,
            )))
            .block(
                Block::default()
                    .style(Style::default().fg(Color::Rgb(
                        config.command_color.unwrap().0,
                        config.command_color.unwrap().1,
                        config.command_color.unwrap().2,
                    )))
                    .title(title_string),
            );
        let parameterized_command = app_state.selected_command.clone().unwrap().command;
        let command = Paragraph::new(parameterized_command)
            .style(Style::default().fg(Color::Rgb(
                config.command_color.unwrap().0,
                config.command_color.unwrap().1,
                config.command_color.unwrap().2,
            )))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().style(Style::default().fg(Color::Rgb(
                config.primary_color.unwrap().0,
                config.primary_color.unwrap().1,
                config.primary_color.unwrap().2,
            ))));
        rect.render_widget(command, overlay_chunks[1]);
        rect.render_widget(input, overlay_chunks[2]);
    })?;
    Ok(())
}

pub fn key_handler(input: Key, app: &mut State) -> Option<HoardCommand> {
    match input {
        // Quit command
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        Key::Char('\n') => {
            let command = app.selected_command.clone().unwrap();
            let parameter = app.input.clone();
            let replaced_command = command.replace_parameters(&app.parameter_token, &[parameter]);
            app.input = String::from("");
            if replaced_command.get_parameter_count(&app.parameter_token) == 0 {
                return Some(replaced_command);
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
