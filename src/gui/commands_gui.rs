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
    commands: Vec<HoardCommand>,
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
        commands: trove.commands.clone(),
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
                app_state.commands.clone(),
                &mut command_list_state,
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
                Key::Left | Key::Ctrl('h') => {
                    if let Some(selected) = namespace_tab_state.selected() {
                        let amount_ns = namespace_tabs.clone().len();
                        if selected > 0 {
                            namespace_tab_state.select(Some(selected - 1));
                        } else {
                            namespace_tab_state.select(Some(amount_ns - 1));
                        }
                        let selected_tab = namespace_tabs
                            .get(
                                namespace_tab_state
                                    .selected()
                                    .expect("Always a namespace selected"),
                            )
                            .expect("Always a tab selected")
                            .clone();
                        apply_search(&mut app_state, trove.commands.clone(), selected_tab);
                        let new_selection = if app_state.commands.is_empty() {
                            0
                        } else {
                            app_state.commands.len() - 1
                        };
                        command_list_state.select(Some(new_selection));
                    }
                }
                Key::Right | Key::Ctrl('l') => {
                    if let Some(selected) = namespace_tab_state.selected() {
                        let amount_ns = namespace_tabs.clone().len();
                        if selected >= amount_ns - 1 {
                            namespace_tab_state.select(Some(0));
                        } else {
                            namespace_tab_state.select(Some(selected + 1));
                        }
                        let selected_tab = namespace_tabs
                            .get(
                                namespace_tab_state
                                    .selected()
                                    .expect("Always a namespace selected"),
                            )
                            .expect("Always a tab selected")
                            .clone();
                        apply_search(&mut app_state, trove.commands.clone(), selected_tab);
                        let new_selection = if app_state.commands.is_empty() {
                            0
                        } else {
                            app_state.commands.len() - 1
                        };
                        command_list_state.select(Some(new_selection));
                    }
                }
                // Switch command
                Key::Up | Key::Ctrl('y') | Key::Ctrl('p') => {
                    if !app_state.commands.is_empty() {
                        if let Some(selected) = command_list_state.selected() {
                            let amount_commands = app_state.commands.clone().len();
                            if selected > 0 {
                                command_list_state.select(Some(selected - 1));
                            } else {
                                command_list_state.select(Some(amount_commands - 1));
                            }
                        }
                    }
                }
                Key::Down | Key::Ctrl('.') | Key::Ctrl('n') => {
                    if !app_state.commands.is_empty() {
                        if let Some(selected) = command_list_state.selected() {
                            let amount_commands = app_state.commands.clone().len();
                            if selected >= amount_commands - 1 {
                                command_list_state.select(Some(0));
                            } else {
                                command_list_state.select(Some(selected + 1));
                            }
                        }
                    }
                }
                // Select command
                Key::Char('\n') => {
                    if app_state.commands.is_empty() {
                        return Ok(None);
                    }
                    let selected_command = app_state
                        .commands
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
                    let selected_tab = namespace_tabs
                            .get(
                                namespace_tab_state
                                    .selected()
                                    .expect("Always a namespace selected"),
                            )
                            .expect("Always a tab selected")
                            .clone();
                    apply_search(&mut app_state, trove.commands.clone(), selected_tab);
                }
                Key::Char(c) => {
                    app_state.input.push(c);
                    let selected_tab = namespace_tabs
                            .get(
                                namespace_tab_state
                                    .selected()
                                    .expect("Always a namespace selected"),
                            )
                            .expect("Always a tab selected")
                            .clone();
                    apply_search(&mut app_state, trove.commands.clone(), selected_tab);
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }
    Ok(None)
}

fn apply_search(app: &mut State, all_commands: Vec<HoardCommand>, selected_tab: String) {
    let query_term = &app.input[..];
    app.commands = all_commands
        .clone()
        .into_iter()
        .filter(|c| {
            (c.name.contains(query_term)
                || c.namespace.contains(query_term)
                || c.tags_as_string().contains(query_term)
                || c.command.contains(query_term)
                || c.description
                    .clone()
                    .unwrap_or_default()
                    .contains(query_term)
            ) &&
            (c.namespace.clone() == selected_tab || selected_tab == "All")
        })
        .collect();
}

fn render_commands<'a>(
    commands_list: Vec<HoardCommand>,
    command_list_state: &mut ListState,
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

    let selected_command: HoardCommand = commands_list
        .get(
            command_list_state
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
        command_list_state.select(Some(new_selection));
    }

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
