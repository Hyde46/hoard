use crate::config::HoardConfig;
use crate::ui::App;
use ratatui::{prelude::*, widgets::*};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Draw the search screen
///
/// # Arguments
/// * `frame` - The frame to draw the UI components on
/// * `app` - The application state
///
/// The header of the screen will display the current version of the application
/// The main screen will display the list of commands and details of the selected command
/// The footer will display the search string and the current collection
pub fn draw_search_screen(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ],
    )
    .split(frame.size());

    render_version_header(frame, main_layout[0]);

    render_main_screen(frame, main_layout[1], app);

    render_search_field(frame, main_layout[2], app);
}

/// Draw the version header
///
/// # Arguments
/// * `frame` - The frame to draw the UI components
/// * `rect` - The area to draw the version header
fn render_version_header(frame: &mut Frame, rect: Rect) {
    let version = format!("Hoard v{}", VERSION);

    frame.render_widget(Paragraph::new(version), rect);
}

/// Draw the search field
///
/// # Arguments
/// * `frame` - The frame to draw the UI components
/// * `rect` - The area to draw the search field
/// * `app` - The application state
fn render_search_field(frame: &mut Frame, rect: Rect, app: &mut App) {
    let search_string = format!("[ {} ] > {}", app.collection, app.search_string);

    frame.render_widget(Paragraph::new(search_string), rect);
}

/// Draw the main screen
///
/// # Arguments
/// * `frame` - The frame to draw the UI components
/// * `rect` - The area to draw the main screen
/// * `app` - The application state
fn render_main_screen(frame: &mut Frame, rect: Rect, app: &mut App) {
    let main_screen_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(30), // Scrollable tray for a list of available commands
            Constraint::Percentage(70), // Detail view for the "hovered" command by the selector
        ],
    )
    .split(rect);

    let vertical_scroll = 0; // from app state

    let items = vec![
        Line::from(vec![
            Span::raw("★ connect_"),
            Span::styled("psql", Style::new().fg(Color::Red)),
        ]),
        Line::from("★ deploy_heroku"),
        Line::from("  hoard_trove_server_psql"),
        Line::from("  connect_mcme_local_db"),
        Line::from("  start_postgres"),
        Line::from("  go_to_home"),
        Line::from("  connect_psql"),
        Line::from("  deploy_heroku"),
        Line::from("  hoard_trove_server_psql"),
        Line::from("  connect_mcme_local_db"),
        Line::from("  start_postgres"),
        Line::from("  go_to_home"),
    ];

    // let paragraph = Paragraph::new(Line::from(vec!["Hello, ".into(), "world!".red()]));
    let paragraph = Paragraph::new(items.clone())
        .scroll((vertical_scroll as u16, 0))
        .block(Block::new().borders(Borders::ALL)); // to show a background for the scrollbar

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state = ScrollbarState::new(items.len()).position(vertical_scroll);

    let area = main_screen_layout[0];

    frame.render_widget(paragraph, area);
    // and the scrollbar, those are separate widgets
    frame.render_stateful_widget(
        scrollbar,
        area.inner(&Margin {
            // using an inner vertical margin of 1 unit makes the scrollbar inside the block
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    let detail_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ],
    )
    .split(main_screen_layout[1]);

    frame.render_widget(
        Paragraph::new("cd /home/monarch/code")
            .block(Block::default().borders(Borders::ALL).title(" Command "))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        detail_layout[0],
    );

    frame.render_widget(
        //Block::new().borders(Borders::RIGHT | Borders::TOP),
        Paragraph::new("This is a longwinded description about the command, Probably left side aligned makes the most sense here").block(Block::default().borders(Borders::ALL).title(" Description ")).alignment(Alignment::Left).wrap(Wrap { trim: false }),
        detail_layout[1]
    );

    frame.render_widget(
        //Block::new().borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP),
        Block::new().borders(Borders::ALL),
        detail_layout[2],
    );
}
