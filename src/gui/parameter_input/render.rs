use crate::config::HoardConfig;
use crate::gui::commands_gui::State;
use crate::util::translate_number_to_nth;
use ratatui::backend::TermionBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Terminal;
use termion::screen::AlternateScreen;

pub fn draw(
    app_state: &State,
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

        let command_text = app_state
            .selected_command
            .as_ref()
            .unwrap()
            .command
            .as_str();

        let token = config.parameter_token.as_ref().unwrap().as_str();
        let ending_token = config.parameter_ending_token.as_ref().unwrap().as_str();

        let mut command_spans: Vec<Span> = Vec::new();

        let mut i = 0;
        let mut found_pos = None;
        let bytes = command_text.as_bytes();

        while i < command_text.len() {
            if bytes[i] == b'\\' {
                i += 1;
                if i < command_text.len() {
                    if command_text[i..].starts_with(token) {
                        i += token.len();
                    } else {
                        let ch = command_text[i..].chars().next().unwrap();
                        i += ch.len_utf8();
                    }
                }
                continue;
            }
            if command_text[i..].starts_with(token) {
                found_pos = Some(i);
                break;
            }
            let ch = command_text[i..].chars().next().unwrap();
            i += ch.len_utf8();
        }

        if let Some(pos) = found_pos {
            let mut full_param_len = token.len();

            if !ending_token.is_empty() {
                let rest = &command_text[pos + token.len()..];
                let mut search_idx = 0;
                let mut found_end_at = None;

                while search_idx < rest.len() {
                    if rest.as_bytes()[search_idx] == b'\\' {
                        search_idx += 1;
                        if search_idx < rest.len() {
                            let ch = rest[search_idx..].chars().next().unwrap();
                            search_idx += ch.len_utf8();
                        }
                        continue;
                    }
                    if rest[search_idx..].starts_with(token) {
                        break;
                    }
                    if rest[search_idx..].starts_with(ending_token) {
                        found_end_at = Some(search_idx + ending_token.len());
                        break;
                    }
                    if rest.as_bytes()[search_idx] == b' ' {
                        break;
                    }

                    let ch = rest[search_idx..].chars().next().unwrap();
                    search_idx += ch.len_utf8();
                }

                if let Some(offset) = found_end_at {
                    full_param_len = token.len() + offset;
                }
            }

            command_spans.push(Span::styled(&command_text[..pos], command_style));
            command_spans.push(Span::styled(
                &command_text[pos..pos + full_param_len],
                primary_style,
            ));
            command_spans.push(Span::styled(
                &command_text[pos + full_param_len..],
                command_style,
            ));
        } else {
            command_spans.push(Span::styled(command_text, command_style));
        }

        let command = Paragraph::new(Line::from(command_spans))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().style(primary_style));

        rect.render_widget(command, overlay_chunks[1]);
        rect.render_widget(input, overlay_chunks[2]);
    })?;
    Ok(())
}
