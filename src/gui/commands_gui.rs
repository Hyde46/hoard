use super::super::command::hoard_command::HoardCommand;
use super::super::command::trove::CommandTrove;
use super::super::config::HoardConfig;
use super::event::{Config, Event, Events};
use std::collections::HashSet;
use std::io::stdout;
use std::time::Duration;
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Terminal,
};
struct State {
    input: String,
}

pub fn run(
    trove: &mut CommandTrove,
    config: &HoardConfig,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(200),
    });

    let mut app_state = State {
        input: String::from(""),
    };

    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    //let menu_titles = vec!["List", "Search", "Add", "Delete", "Quit"];
    let mut namespace_tabs: Vec<String> = trove
        .commands
        .iter()
        .map(|command| command.namespace.clone())
        .collect::<HashSet<String>>()
        .into_iter()
        .collect();
    namespace_tabs.insert(0, String::from("All"));
    let mut command_list_state = ListState::default();
    command_list_state.select(Some(0));
    let mut namespace_tab_state = ListState::default();
    namespace_tab_state.select(Some(0));
    let mut filetered_trove_commands = trove.commands.clone();
    loop {
        // Draw GUI
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
                    ]
                    .as_ref(),
                )
                .split(size);
            let menu = namespace_tabs
                .iter()
                .map(|t| {
                    Spans::from(vec![Span::styled(
                        t,
                        Style::default().fg(Color::Rgb(242, 229, 188)),
                    )])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(
                    namespace_tab_state
                        .selected()
                        .expect("Always a namespace selected"),
                )
                .block(
                    Block::default()
                        .title(" Hoard Namespace ")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Rgb(242, 229, 188)))
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
            let (commands, command, tags, description, input) = render_commands(
                filetered_trove_commands.clone(),
                &command_list_state,
                &app_state,
                config,
            );
            rect.render_stateful_widget(commands, commands_chunks[0], &mut command_list_state);
            rect.render_widget(tags, command_detail_chunks[0]);
            rect.render_widget(description, command_detail_chunks[1]);
            rect.render_widget(command, command_detail_chunks[2]);
            rect.render_widget(input, chunks[2]);
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                // Quit command
                Key::Esc | Key::Ctrl('c') | Key::Ctrl('d') | Key::Ctrl('g') => {
                    terminal.show_cursor()?;
                    break;
                }
                // Switch namespace
                Key::Right | Key::Ctrl('h') => {
                    if let Some(selected) = namespace_tab_state.selected() {
                        let amount_ns = namespace_tabs.clone().len();
                        if selected > 0 {
                            namespace_tab_state.select(Some(selected - 1));
                        } else {
                            namespace_tab_state.select(Some(amount_ns - 1));
                        }
                        update_filtered_trove_commands(
                            &namespace_tabs,
                            &namespace_tab_state,
                            &mut filetered_trove_commands,
                            &trove,
                        );

                        command_list_state.select(Some(0));
                    }
                }
                Key::Left | Key::Ctrl('l') => {
                    if let Some(selected) = namespace_tab_state.selected() {
                        let amount_ns = namespace_tabs.clone().len();
                        if selected >= amount_ns - 1 {
                            namespace_tab_state.select(Some(0));
                        } else {
                            namespace_tab_state.select(Some(selected + 1));
                        }
                        update_filtered_trove_commands(
                            &namespace_tabs,
                            &namespace_tab_state,
                            &mut filetered_trove_commands,
                            &trove,
                        );

                        command_list_state.select(Some(0));
                    }
                }
                // Switch command
                Key::Up | Key::Ctrl('y') | Key::Ctrl('p') => {
                    if let Some(selected) = command_list_state.selected() {
                        let amount_commands = filetered_trove_commands.clone().len();
                        if selected > 0 {
                            command_list_state.select(Some(selected - 1));
                        } else {
                            command_list_state.select(Some(amount_commands - 1));
                        }
                    }
                }
                Key::Down | Key::Ctrl('.') | Key::Ctrl('n') => {
                    if let Some(selected) = command_list_state.selected() {
                        let amount_commands = filetered_trove_commands.clone().len();
                        if selected >= amount_commands - 1 {
                            command_list_state.select(Some(0));
                        } else {
                            command_list_state.select(Some(selected + 1));
                        }
                    }
                }
                // Select command
                Key::Char('\n') => {
                    let selected_command = filetered_trove_commands
                        .clone()
                        .get(
                            command_list_state
                                .selected()
                                .expect("there is always a selected command"),
                        )
                        .expect("exists")
                        .clone();
                    terminal.show_cursor()?;
                    return Ok(Some(selected_command.command));
                }
                // Handle query input
                Key::Backspace => {
                    app_state.input.pop();
                }
                Key::Char(c) => {
                    app_state.input.push(c);
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }
    Ok(None)
}

fn update_filtered_trove_commands(
    namespace_tabs: &[String],
    namespace_tab_state: &ListState,
    filetered_trove_commands: &mut Vec<HoardCommand>,
    trove: &&mut CommandTrove,
) {
    let selected_tab = namespace_tabs
        .get(
            namespace_tab_state
                .selected()
                .expect("Always a namespace selected"),
        )
        .expect("Always a tab selected")
        .clone();
    *filetered_trove_commands = trove
        .commands
        .clone()
        .into_iter()
        .filter(|command| command.namespace == selected_tab || selected_tab == "All")
        .collect();
}

fn render_commands<'a>(
    commands_list: Vec<HoardCommand>,
    command_list_state: &ListState,
    app: &State,
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
        .style(Style::default().fg(Color::Rgb(242, 229, 188)))
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

    let selected_command = commands_list
        .get(
            command_list_state
                .selected()
                .expect("there is always a selected command"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(commands).highlight_style(
        Style::default()
            .bg(Color::Rgb(181, 118, 20))
            .fg(Color::Rgb(50, 48, 47))
            .add_modifier(Modifier::BOLD),
    );

    let command = Paragraph::new(selected_command.command.clone())
        .style(Style::default().fg(Color::Rgb(181, 118, 20)))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(242, 229, 188)))
                .title(" Hoarded command ")
                .border_type(BorderType::Plain),
        );

    let tags = Paragraph::new(selected_command.tags_as_string())
        .style(Style::default().fg(Color::Rgb(242, 229, 188)))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(242, 229, 188)))
                .title(" Tags ")
                .border_type(BorderType::Plain),
        );

    let description = Paragraph::new(selected_command.description.unwrap_or_default())
        .style(Style::default().fg(Color::Rgb(242, 229, 188)))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Rgb(242, 229, 188)))
                .title(" Description ")
                .border_type(BorderType::Plain),
        );

    let mut query_string = config.query_prefix.clone();
    query_string.push_str(&app.input.clone()[..]);
    let input = Paragraph::new(query_string).block(
        Block::default()
            .style(Style::default().fg(Color::Rgb(242, 229, 188)))
            .borders(Borders::ALL)
            .title(" Query "),
    );

    (list, command, tags, description, input)
}
