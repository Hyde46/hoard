use super::super::command::hoard_command::HoardCommand;
use super::super::command::trove::CommandTrove;
use super::super::config::HoardConfig;
use super::event::{Config, Event, Events};
use eyre::Result;
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

const VERSION: &str = env!("CARGO_PKG_VERSION");
struct State {
    input: String,
    commands: Vec<HoardCommand>,
    command_list_state: ListState,
    namespace_tab_state: ListState,
    should_exit: bool,
}

#[allow(clippy::too_many_lines)]
pub fn run(trove: &mut CommandTrove, config: &HoardConfig) -> Result<String> {
    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(200),
    });

    let mut app_state = State {
        input: String::from(""),
        commands: trove.commands.clone(),
        command_list_state: ListState::default(),
        namespace_tab_state: ListState::default(),
        should_exit: false,
    };

    app_state.command_list_state.select(Some(0));
    app_state.namespace_tab_state.select(Some(0));

    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    //let menu_titles = vec!["List", "Search", "Add", "Delete", "Quit"];
    let mut namespace_tabs: Vec<&str> = trove.namespaces();
    namespace_tabs.insert(0, "All");
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
                        *t,
                        Style::default().fg(Color::Rgb(242, 229, 188)),
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
            let (commands, command, tags_widget, description, input) =
                render_commands(&app_state.commands.clone(), &mut app_state, config);
            rect.render_stateful_widget(
                commands,
                commands_chunks[0],
                &mut app_state.command_list_state,
            );
            rect.render_widget(tags_widget, command_detail_chunks[0]);
            rect.render_widget(description, command_detail_chunks[1]);
            rect.render_widget(command, command_detail_chunks[2]);
            rect.render_widget(input, chunks[2]);
        })?;

        if let Event::Input(input) = events.next()? {
            if let Some(output) =
                key_handler(input, &mut app_state, &trove.commands, &namespace_tabs)
            {
                terminal.show_cursor()?;
                return Ok(output);
            }
        }
    }
}

#[allow(clippy::too_many_lines, clippy::ptr_arg)]
fn key_handler(
    input: Key,
    app: &mut State,
    trove_commands: &Vec<HoardCommand>,
    namespace_tabs: &Vec<&str>,
) -> Option<String> {
    match input {
        // Quit command
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            Some(String::from(""))
        }
        // Switch namespace
        Key::Left | Key::Ctrl('h') => {
            if let Some(selected) = app.namespace_tab_state.selected() {
                let amount_ns = namespace_tabs.clone().len();
                if selected > 0 {
                    app.namespace_tab_state.select(Some(selected - 1));
                } else {
                    app.namespace_tab_state.select(Some(amount_ns - 1));
                }
                let selected_tab = namespace_tabs
                    .get(
                        app.namespace_tab_state
                            .selected()
                            .expect("Always a namespace selected"),
                    )
                    .expect("Always a tab selected");
                apply_search(app, trove_commands, selected_tab);
                let new_selection = if app.commands.is_empty() {
                    0
                } else {
                    app.commands.len() - 1
                };
                app.command_list_state.select(Some(new_selection));
            }
            None
        }
        Key::Right | Key::Ctrl('l') => {
            if let Some(selected) = app.namespace_tab_state.selected() {
                let amount_ns = namespace_tabs.clone().len();
                if selected >= amount_ns - 1 {
                    app.namespace_tab_state.select(Some(0));
                } else {
                    app.namespace_tab_state.select(Some(selected + 1));
                }
                let selected_tab = namespace_tabs
                    .get(
                        app.namespace_tab_state
                            .selected()
                            .expect("Always a namespace selected"),
                    )
                    .expect("Always a tab selected");
                apply_search(app, trove_commands, selected_tab);
                let new_selection = if app.commands.is_empty() {
                    0
                } else {
                    app.commands.len() - 1
                };
                app.command_list_state.select(Some(new_selection));
            }
            None
        }
        // Switch command
        Key::Up | Key::Ctrl('y' | 'p') => {
            if !app.commands.is_empty() {
                if let Some(selected) = app.command_list_state.selected() {
                    let amount_commands = app.commands.clone().len();
                    if selected > 0 {
                        app.command_list_state.select(Some(selected - 1));
                    } else {
                        app.command_list_state.select(Some(amount_commands - 1));
                    }
                }
            }
            None
        }
        Key::Down | Key::Ctrl('.' | 'n') => {
            if !app.commands.is_empty() {
                if let Some(selected) = app.command_list_state.selected() {
                    let amount_commands = app.commands.clone().len();
                    if selected >= amount_commands - 1 {
                        app.command_list_state.select(Some(0));
                    } else {
                        app.command_list_state.select(Some(selected + 1));
                    }
                }
            }
            None
        }
        // Select command
        Key::Char('\n') => {
            if app.commands.is_empty() {
                return Some(String::from(""));
            }
            let selected_command = app
                .commands
                .clone()
                .get(
                    app.command_list_state
                        .selected()
                        .expect("there is always a selected command"),
                )
                .expect("exists")
                .clone();
            Some(selected_command.command)
        }
        // Handle query input
        Key::Backspace => {
            app.input.pop();
            let selected_tab = namespace_tabs
                .get(
                    app.namespace_tab_state
                        .selected()
                        .expect("Always a namespace selected"),
                )
                .expect("Always a tab selected");
            apply_search(app, trove_commands, selected_tab);
            None
        }
        Key::Char(c) => {
            app.input.push(c);
            let selected_tab = namespace_tabs
                .get(
                    app.namespace_tab_state
                        .selected()
                        .expect("Always a namespace selected"),
                )
                .expect("Always a tab selected");
            apply_search(app, trove_commands, selected_tab);
            None
        }
        _ => None,
    }
}

#[allow(clippy::ptr_arg)]
fn apply_search(app: &mut State, all_commands: &Vec<HoardCommand>, selected_tab: &str) {
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
                    .contains(query_term))
                && (c.namespace.clone() == *selected_tab || selected_tab == "All")
        })
        .collect();
}

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
            .bg(Color::Rgb(181, 118, 20))
            .fg(Color::Rgb(50, 48, 47))
            .add_modifier(Modifier::BOLD),
    );

    let command = Paragraph::new(selected_command.command.clone())
        .style(Style::default().fg(Color::Rgb(181, 118, 20)))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
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
    let query_title = format!(" hoard v{} ", VERSION);
    let input = Paragraph::new(query_string).block(
        Block::default()
            .style(Style::default().fg(Color::Rgb(242, 229, 188)))
            .borders(Borders::ALL)
            .title(query_title),
    );

    (list, command, tags, description, input)
}
