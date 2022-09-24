use crossbeam_channel::unbounded;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
#[allow(dead_code)]
pub struct Events {
    rx: crossbeam_channel::Receiver<Event<Key>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    #[allow(clippy::manual_flatten)]
    pub fn with_config(config: Config) -> Self {
        let (tx, rx) = unbounded();

        {
            let tx = tx.clone();
            thread::spawn(move || {
                let tty = termion::get_tty().expect("Could not find tty session");
                for key in tty.keys().flatten() {
                    if let Err(err) = tx.send(Event::Input(key)) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            })
        };

        thread::spawn(move || loop {
            if let Err(err) = tx.send(Event::Tick) {
                eprintln!("{}", err);
                break;
            }
            thread::sleep(config.tick_rate);
        });
        Self { rx }
    }

    pub fn next(&self) -> Result<Event<Key>, crossbeam_channel::RecvError> {
        self.rx.recv()
    }
}
