use crate::config::HoardConfig;
use crate::gui::commands_gui::State;
use termion::screen::AlternateScreen;
use ratatui::backend::TermionBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;

pub fn draw(
    app_state: &State,
    config: &HoardConfig,
    terminal: &mut Terminal<
        TermionBackend<AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>>,
    >,
    default_namespace: &str,
) -> Result<(), eyre::Error> {
    terminal.draw(|rect| {
        let size = rect.size();
        // Overlay
        let overlay_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                    Constraint::Percentage(10),
                    Constraint::Percentage(20),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(size);

        let mut query_string = config.query_prefix.clone();
        query_string.push_str(&app_state.input.clone()[..]);
        let title_string = format!("Provide {} for the command", app_state.edit_selection);

        let command_style = Style::default().fg(Color::Rgb(
            config.command_color.unwrap().0,
            config.command_color.unwrap().1,
            config.command_color.unwrap().2,
        ));

        let primary_style = Style::default().fg(Color::Rgb(
            config.primary_color.unwrap().0,
            config.primary_color.unwrap().1,
            config.primary_color.unwrap().2,
        ));

        let input = Paragraph::new(query_string)
            .style(primary_style)
            .block(Block::default().style(command_style).title(title_string));
        let new_command = app_state.new_command.clone().unwrap();
        let command_render = format!(
            "Command: {}\nNamespace: {}(\"{}\" if empty)\nName: {}\nDescription: {}\nTags: {}",
            new_command.command,
            new_command.namespace,
            default_namespace,
            new_command.name,
            new_command.clone().description,
            new_command.get_tags_as_string()
        );
        let new_command = Paragraph::new(command_render)
            .style(primary_style)
            .block(Block::default().style(command_style).title("New command:"));

        let error_message = Paragraph::new(app_state.error_message.clone())
            .style(primary_style)
            .block(Block::default().style(command_style).title("Error:"));

        rect.render_widget(new_command, overlay_chunks[1]);
        rect.render_widget(input, overlay_chunks[2]);
        if !app_state.error_message.is_empty() {
            rect.render_widget(error_message, overlay_chunks[3]);
        }
    })?;
    Ok(())
}
