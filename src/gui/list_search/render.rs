use crate::command::hoard_command::HoardCommand;
use crate::config::HoardConfig;
use crate::gui::commands_gui::State;
use crate::gui::help::HELP_KEY;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Tabs, Wrap};
use tui::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn draw(
    app_state: &mut State,
    config: &HoardConfig,
    namespace_tabs: &mut [&str],
    terminal: &mut Terminal<
        TermionBackend<AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>>,
    >,
) -> Result<(), eyre::Error> {
    terminal.draw(|rect| {
        let size = rect.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(size);
        let menu = namespace_tabs
            .iter()
            .map(|t| {
                Spans::from(vec![Span::styled(
                    *t,
                    Style::default().fg(Color::Rgb(
                        config.primary_color.unwrap().0,
                        config.primary_color.unwrap().1,
                        config.primary_color.unwrap().2,
                    )),
                )])
            })
            .collect();

        let tabs = Tabs::new(menu)
            .select(
                app_state
                    .namespace_tab_state
                    .selected()
                    .expect("Always a namespace selected"),
            )
            .block(
                Block::default()
                    .title(" Hoard Namespace ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::Rgb(
                config.primary_color.unwrap().0,
                config.primary_color.unwrap().1,
                config.primary_color.unwrap().2,
            )))
            .highlight_style(
                Style::default()
                    .fg(Color::Rgb(181, 118, 20))
                    .add_modifier(Modifier::UNDERLINED),
            )
            .divider(Span::raw("|"));

        rect.render_widget(tabs, chunks[0]);

        let commands_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(chunks[1]);
        let command_detail_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Percentage(60),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(commands_chunks[1]);
        let (commands, command, tags_widget, description, input) =
            render_commands(&app_state.commands.clone(), app_state, config);
        rect.render_stateful_widget(
            commands,
            commands_chunks[0],
            &mut app_state.command_list_state,
        );
        rect.render_widget(tags_widget, command_detail_chunks[0]);
        rect.render_widget(description, command_detail_chunks[1]);
        rect.render_widget(command, command_detail_chunks[2]);
        rect.render_widget(input, chunks[2]);

        let help_hint = Paragraph::new(format!("Show help - {HELP_KEY} "))
            .style(Style::default().fg(Color::Rgb(
                config.primary_color.unwrap().0,
                config.primary_color.unwrap().1,
                config.primary_color.unwrap().2,
            )))
            .alignment(Alignment::Right);

        rect.render_widget(help_hint, chunks[3]);
    })?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn render_commands<'a>(
    commands_list: &[HoardCommand],
    app: &mut State,
    config: &HoardConfig,
) -> (
    List<'a>,
    Paragraph<'a>,
    Paragraph<'a>,
    Paragraph<'a>,
    Paragraph<'a>,
) {
    let commands = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Rgb(
            config.primary_color.unwrap().0,
            config.primary_color.unwrap().1,
            config.primary_color.unwrap().2,
        )))
        .title(" Commands ")
        .border_type(BorderType::Plain);

    let items: Vec<_> = commands_list
        .iter()
        .map(|command| {
            ListItem::new(Spans::from(vec![Span::styled(
                command.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_command: HoardCommand = commands_list
        .get(
            app.command_list_state
                .selected()
                .expect("there is always a selected command"),
        )
        .get_or_insert(&HoardCommand::default())
        .clone();

    if selected_command.name.is_empty() {
        // If somehow the selection is past the last index, set it to the last element
        let new_selection = if commands_list.is_empty() {
            0
        } else {
            commands_list.len() - 1
        };
        app.command_list_state.select(Some(new_selection));
    }

    let list = List::new(items).block(commands).highlight_style(
        Style::default()
            .bg(Color::Rgb(
                config.secondary_color.unwrap().0,
                config.secondary_color.unwrap().1,
                config.secondary_color.unwrap().2,
            ))
            .fg(Color::Rgb(
                config.tertiary_color.unwrap().0,
                config.tertiary_color.unwrap().1,
                config.tertiary_color.unwrap().2,
            ))
            .add_modifier(Modifier::BOLD),
    );

    let command = Paragraph::new(selected_command.command.clone())
        .style(Style::default().fg(Color::Rgb(
            config.command_color.unwrap().0,
            config.command_color.unwrap().1,
            config.command_color.unwrap().2,
        )))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(
                    config.primary_color.unwrap().0,
                    config.primary_color.unwrap().1,
                    config.primary_color.unwrap().2,
                )))
                .title(" Hoarded command ")
                .border_type(BorderType::Plain),
        );

    let tags = Paragraph::new(selected_command.tags_as_string())
        .style(Style::default().fg(Color::Rgb(
            config.primary_color.unwrap().0,
            config.primary_color.unwrap().1,
            config.primary_color.unwrap().2,
        )))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(
                    config.primary_color.unwrap().0,
                    config.primary_color.unwrap().1,
                    config.primary_color.unwrap().2,
                )))
                .title(" Tags ")
                .border_type(BorderType::Plain),
        );

    let description = Paragraph::new(selected_command.description.unwrap_or_default())
        .style(Style::default().fg(Color::Rgb(
            config.primary_color.unwrap().0,
            config.primary_color.unwrap().1,
            config.primary_color.unwrap().2,
        )))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(
                    config.primary_color.unwrap().0,
                    config.primary_color.unwrap().1,
                    config.primary_color.unwrap().2,
                )))
                .title(" Description ")
                .border_type(BorderType::Plain),
        );

    let mut query_string = config.query_prefix.clone();
    query_string.push_str(&app.input.clone()[..]);
    let query_title = format!(" hoard v{VERSION} ");
    let input = Paragraph::new(query_string).block(
        Block::default()
            .style(Style::default().fg(Color::Rgb(
                config.primary_color.unwrap().0,
                config.primary_color.unwrap().1,
                config.primary_color.unwrap().2,
            )))
            .borders(Borders::ALL)
            .title(query_title),
    );

    (list, command, tags, description, input)
}
