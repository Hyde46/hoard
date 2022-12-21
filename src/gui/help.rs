use crate::command::hoard_command::HoardCommand;
use crate::config::HoardConfig;
use crate::gui::commands_gui::{DrawState, State};
use termion::event::Key;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, List, ListItem};
use tui::Terminal;

pub const HELP_KEY: &str = "<F1>";
const HELP_CONTENT: &[(&str, &str)] = &[
    ("Next item in command list", "<Ctrl-N> / <Down-Arrow>"),
    (
        "Previous item in command list",
        "<Ctrl-P> / <Ctrl-Y> / <Up-Arrow>",
    ),
    ("Next namespace tab", "<Ctrl-L> / <Right-Arrow>"),
    ("Previous namespace tab", "<Ctrl-H> / <Left-Arrow>"),
    ("Select command", "<Enter>"),
    ("Delete command", "<Ctrl-X>"),
    ("Toggle search/edit mode", "<Tab> / <Ctrl-E>"),
    ("Toggle Command to edit in edit mode", "<Tab>"),
    ("Exit edit mode", "<Esc>"),
    ("Quit", "<Ctrl-D> / <Ctrl-C> / <Ctrl-G>"),
    ("Show help", HELP_KEY),
    ("Close help", "<Any key>"),
];

pub fn draw(
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

pub fn key_handler(_input: Key, app: &mut State) -> Option<HoardCommand> {
    app.draw_state = DrawState::Search;
    None
}
