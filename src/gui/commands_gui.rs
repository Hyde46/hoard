use super::super::command::hoard_command::HoardCommand;
use super::super::command::trove::CommandTrove;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap
    },
    Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
enum MenuItem {
    List,
    Search,
    Add,
    Delete,
    Quit,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::List => 0,
            MenuItem::Search => 1,
            MenuItem::Add => 2,
            MenuItem::Delete => 3,
            MenuItem::Quit => 4,
        }
    }
}

pub fn run(trove: &mut CommandTrove) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("Cant run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    
    //let menu_titles = vec!["List", "Search", "Add", "Delete", "Quit"];
    let menu_titles = vec!["List", "Quit"];
    let active_menu_item = MenuItem::List;
    let mut command_list_state = ListState::default();
    command_list_state.select(Some(0));

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
            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Hoard Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            
            let commands_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
            let command_detail_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(30), Constraint::Percentage(30), Constraint::Percentage(40)].as_ref(),
                        )
                        .split(commands_chunks[1]);
            let (commands, command, namespace, tags, description) = render_commands(trove.commands.clone(), &command_list_state);
            rect.render_stateful_widget(commands, commands_chunks[0], &mut command_list_state);
            rect.render_widget(namespace, command_detail_chunks[0]);
            rect.render_widget(tags, command_detail_chunks[1]);
            rect.render_widget(description, command_detail_chunks[2]);
            rect.render_widget(command, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Down => {
                    if let Some(selected) = command_list_state.selected() {
                        let amount_commands = trove.commands.clone().len();
                        if selected >= amount_commands - 1 {
                            command_list_state.select(Some(0));
                        } else {
                            command_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = command_list_state.selected() {
                        let amount_commands = trove.commands.clone().len();
                        if selected > 0 {
                            command_list_state.select(Some(selected - 1));
                        } else {
                            command_list_state.select(Some(amount_commands - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }
    Ok(())
}

fn render_commands<'a>(commands_list: Vec<HoardCommand>, command_list_state: &ListState) -> (List<'a>, Paragraph<'a>, Paragraph<'a>, Paragraph<'a>, Paragraph<'a>) {
    let commands = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Commands")
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
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let command = Paragraph::new(selected_command.command.clone())
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Hoarded command")
            .border_type(BorderType::Plain),
    );

    let namespace = Paragraph::new(selected_command.namespace.clone())
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Namespace")
            .border_type(BorderType::Plain),
    );

    let tags = Paragraph::new(selected_command.tags_as_string())
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Tags")
            .border_type(BorderType::Plain),
    );

    let description = Paragraph::new(selected_command.description.unwrap_or_default())
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Left)
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Description")
            .border_type(BorderType::Plain),
    );

    (list, command, namespace, tags, description)
}
