use crate::config::HoardConfig;
use crate::ui::{App, partial_highlighted_line};
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

    render_version_header_widget(frame, main_layout[0]);

    render_main_screen(frame, main_layout[1], app);

    render_search_field_widget(frame, main_layout[2], app);
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

    render_commands_list_widget(frame, main_screen_layout[0], app);

    render_command_detail(frame, main_screen_layout[1], app);
}

fn render_command_detail(frame: &mut Frame, rect: Rect, app: &mut App) {
    let detail_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ],
    )
    .split(rect);

    render_command_string_widget(frame, detail_layout[0], app);

    render_command_description_widget(frame, detail_layout[1], app);

    render_command_subdetails_widget(frame, detail_layout[2], app);
}


/// Draw the version header
///
/// # Arguments
/// * `frame` - The frame to draw the UI components
/// * `rect` - The area to draw the version header
fn render_version_header_widget(frame: &mut Frame, rect: Rect) {
    let version = format!("Hoard v{}", VERSION);

    frame.render_widget(Paragraph::new(version), rect);
}

/// Draw the search field
///
/// # Arguments
/// * `frame` - The frame to draw the UI components
/// * `rect` - The area to draw the search field
/// * `app` - The application state
fn render_search_field_widget(frame: &mut Frame, rect: Rect, app: &mut App) {
    let search_string = format!("[ {} ] > {}", app.collection, app.search_string);

    frame.render_widget(Paragraph::new(search_string), rect);
}

fn render_commands_list_widget(frame: &mut Frame, rect: Rect, app: &mut App) {
    let vertical_scroll = app.vertical_scroll; // from app state
    let items = build_command_list_items(app);
    let paragraph = Paragraph::new(items.clone())
        .scroll((vertical_scroll as u16, 0))
        .block(Block::new().borders(Borders::ALL)); // to show a background for the scrollbar

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state = ScrollbarState::new(items.len()).position(vertical_scroll);

    frame.render_widget(paragraph, rect);

    // and the scrollbar, those are separate widgets
    frame.render_stateful_widget(
        scrollbar,
        rect.inner(&Margin {
            // using an inner vertical margin of 1 unit makes the scrollbar inside the block
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_command_string_widget(frame: &mut Frame, rect: Rect, app: &mut App) {
    frame.render_widget(
        Paragraph::new("cd /home/monarch/code")
            .block(Block::default().borders(Borders::ALL).title(" Command "))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn render_command_description_widget(frame: &mut Frame, rect: Rect, app: &mut App) {
    frame.render_widget(
        Paragraph::new("This is a longwinded description about the command, Probably left side aligned makes the most sense here")
            .block(Block::default().borders(Borders::ALL).title(" Description "))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn render_command_subdetails_widget(frame: &mut Frame, rect: Rect, app: &mut App) {
    frame.render_widget(
        //Block::new().borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP),
        Block::new().borders(Borders::ALL),
        rect,
    );
}

/// Build the list of commands to display based on the trove in the current 
/// app state. The list of commands is highlighted based on the search string
fn build_command_list_items(app: &App) -> Vec<Line> {
    app.trove
        .commands
        .iter()
        .map(|command| partial_highlighted_line(&command.name, &app.search_string))
        .collect()
}
