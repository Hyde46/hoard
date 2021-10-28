use crate::command::hoard_command::{HoardCommand, Parameterized};
use crate::command::trove::CommandTrove;
use crate::config::HoardConfig;
use crate::gui::event::{Config, Event, Events};
use crate::util::translate_number_to_nth;
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

const HELP_CONTENT: &[(&str, &str)] = &[
    ("Next item in command list", "<Ctrl-N> / <Down-Arrow>"),
    (
        "Previous item in command list",
        "<Ctrl-P> / <Ctrl-Y> / <Up-Arrow>",
    ),
    ("Next namespace tab", "<Ctrl-L> / <Right-Arrow>"),
    ("Previous namespace tab", "<Ctrl-H> / <Left-Arrow>"),
    ("Select command", "<Enter>"),
    ("Quit", "<Ctrl-D> / <Ctrl-C> / <Ctrl-G>"),
    ("Show help", "<F1>"),
];

const VERSION: &str = env!("CARGO_PKG_VERSION");
struct State {
    input: String,
    commands: Vec<HoardCommand>,
    command_list_state: ListState,
    namespace_tab_state: ListState,
    should_exit: bool,
    draw_state: DrawState,
    parameter_token: String,
    selected_command: Option<HoardCommand>,
    provided_parameter_count: u16,
}

#[derive(Eq, PartialEq)]
enum DrawState {
    Search,
    ParameterInput,
    Help,
}

#[allow(clippy::too_many_lines)]
pub fn run(trove: &mut CommandTrove, config: &HoardConfig) -> Result<Option<HoardCommand>> {
    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(200),
    });

    let mut app_state = State {
        input: "".to_string(),
        commands: trove.commands.clone(),
        command_list_state: ListState::default(),
        namespace_tab_state: ListState::default(),
        should_exit: false,
        draw_state: DrawState::Search,
        parameter_token: config.parameter_token.as_ref().unwrap().clone(),
        selected_command: None,
        provided_parameter_count: 0,
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
        match app_state.draw_state {
            DrawState::Search => {
                draw_list_search(&mut app_state, config, &mut namespace_tabs, &mut terminal)?;
            }
            DrawState::ParameterInput => {
                draw_parameter_input(&mut app_state, config, &mut terminal)?;
            }
            DrawState::Help => {
                draw_help(config, &mut terminal)?;
            }
        }

        if let Event::Input(input) = events.next()? {
            let command = match app_state.draw_state {
                DrawState::Search => {
                    key_handler_list_search(input, &mut app_state, &trove.commands, &namespace_tabs)
                }
                DrawState::ParameterInput => key_handler_parameter_input(input, &mut app_state),
                DrawState::Help => key_handler_help(input, &mut app_state),
            };

            if let Some(output) = command {
                terminal.show_cursor()?;
                return Ok(Some(output));
            }

            if app_state.should_exit {
                terminal.show_cursor()?;
                return Ok(None);
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn draw_list_search(
    app_state: &mut State,
    config: &HoardConfig,
    namespace_tabs: &mut Vec<&str>,
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
    })?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn draw_parameter_input(
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

fn draw_help(
    config: &HoardConfig,
    terminal: &mut Terminal<
        TermionBackend<AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>>,
    >,
) -> Result<(), eyre::Error> {
    terminal.draw(|rect| {
        let help = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Rgb(
                config.primary_color.unwrap().0,
                config.primary_color.unwrap().1,
                config.primary_color.unwrap().2,
            )))
            .title(" Help ")
            .border_type(BorderType::Plain);

        let items: Vec<_> = HELP_CONTENT
            .iter()
            .map(|item| {
                ListItem::new(vec![
                    Spans::from(Span::styled(
                        item.0,
                        Style::default().fg(Color::Rgb(
                            config.command_color.unwrap().0,
                            config.command_color.unwrap().1,
                            config.command_color.unwrap().2,
                        )),
                    )),
                    Spans::from(Span::styled(
                        format!("    {}", item.1),
                        Style::default().fg(Color::Rgb(
                            config.primary_color.unwrap().0,
                            config.primary_color.unwrap().1,
                            config.primary_color.unwrap().2,
                        )),
                    )),
                    Spans::from(""),
                ])
            })
            .collect();

        let list = List::new(items).block(help);
        rect.render_widget(list, rect.size());
    })?;
    Ok(())
}

#[allow(clippy::too_many_lines, clippy::ptr_arg)]
fn key_handler_list_search(
    input: Key,
    app: &mut State,
    trove_commands: &Vec<HoardCommand>,
    namespace_tabs: &Vec<&str>,
) -> Option<HoardCommand> {
    match input {
        // Quit command
        Key::Esc | Key::Ctrl('c' | 'd' | 'g') => {
            app.should_exit = true;
            None
        }
        Key::F(1) => {
            app.draw_state = DrawState::Help;
            None
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
                app.should_exit = true;
                return None;
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

            // Check if parameters need to be supplied
            if selected_command.get_parameter_count(&app.parameter_token) > 0 {
                // Set next state to draw
                app.draw_state = DrawState::ParameterInput;
                // Save which command to replace parameters for
                app.selected_command = Some(selected_command);
                // Empty input for next screen
                app.input = "".to_string();
                // return None, otherwise drawing will quit
                return None;
            }
            Some(selected_command)
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

fn key_handler_parameter_input(input: Key, app: &mut State) -> Option<HoardCommand> {
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

fn key_handler_help(_input: Key, app: &mut State) -> Option<HoardCommand> {
    app.draw_state = DrawState::Search;
    None
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
    let query_title = format!(" hoard v{} (F1 for help)", VERSION);
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
