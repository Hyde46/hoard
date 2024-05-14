mod event;
mod search;

use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use std::io::stdout;
use eyre::Result;
use ratatui::{prelude::*, widgets::*};
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use termion::event::Key;

use crate::config::HoardConfig;
use crate::core::trove::Trove;
use crate::core::HoardCmd;
use crate::ui::event::{Config, Event, Events};



#[derive(Debug, Eq, PartialEq)]
pub enum DrawState {
    Search,
    Explore,
    About,
}

pub struct App {
    /// If set to true, the UI will exit on the next loop iteration
    pub should_exit: bool,
    // ratatui list of commands to display
    pub commands: ListState,
    // ratatui list of collections to display
    pub collections: ListState,
    // current screen to draw
    pub screen: DrawState, 
 }

impl Default for App {
    fn default() -> Self {
        let mut state = Self {
            should_exit: false,
            commands: ListState::default(),
            collections: ListState::default(),
            screen: DrawState::Search,
        };
        state.commands.select(Some(0));
        state.collections.select(Some(0));
        state
    }
}


/// The main entry point for the UI
/// Handles setting up the UI, running the main loop
/// and switching between different screens based on events it recieves
pub fn run(trove: &mut Trove, config: &HoardConfig) -> Result<()> {
    // Setup terminal
    let stdout = stdout().into_raw_mode()?;
    let stdout = stdout.into_alternate_screen().unwrap();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // create app and run it
    let tick_rate = Duration::from_millis(200);
    let app = App::default();
    let res = run_app(&mut terminal, app, tick_rate);

    // Cleanup
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err}");
        return Err(err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) ->  Result<Option<HoardCmd>> {
    let mut last_tick = Instant::now();
    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(tick_rate.as_millis() as u64),
    });

    // Main loop of the UI
    // first match the current DrawState, and draw correct screen
    // then check for any events that might have happened and handle them
    loop {
        let screen = match app.screen {
            DrawState::Search => {
                not_implemented_ui
            }
            DrawState::Explore => {
                not_implemented_ui
            }
            DrawState::About => {
                not_implemented_ui
            }
        };
        terminal.draw(|f| screen(f, &mut app))?;
        
        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('q') => {
                    return Ok(None);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn not_implemented_ui(frame: &mut Frame, _app: &mut App) {
    frame.render_widget(
        Paragraph::new("Not implemented :(")
            .dark_gray()
            .alignment(Alignment::Center),
        frame.size(),
    );
}
