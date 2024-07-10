extern crate chrono;

use crate::config::HoardConfig;
use crate::core::HoardCmd;
use crate::ui::{partial_highlighted_line, App};
use chrono::offset::Utc;
use chrono::DateTime;
use ratatui::{prelude::*, widgets::*};
use std::time::SystemTime;

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

    app.frame_size = frame.size();

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
    let search_string = format!("[ {} ] > {}", app.current_collection, app.search_string);

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
    let selected_command = app
        .get_selected_hoard_command()
        .unwrap_or(HoardCmd::default());
    frame.render_widget(
        Paragraph::new(selected_command.command.clone())
            .block(Block::default().borders(Borders::ALL).title(" Command "))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn render_command_description_widget(frame: &mut Frame, rect: Rect, app: &mut App) {
    let selected_command = app
        .get_selected_hoard_command()
        .unwrap_or(HoardCmd::default());
    frame.render_widget(
        Paragraph::new(selected_command.description.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Description "),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        rect,
    );
}

fn render_command_subdetails_widget(frame: &mut Frame, rect: Rect, app: &mut App) {
    if let Some(selected_command) = app.get_selected_hoard_command() {
        let usage_count = format!(
            "Usage count: ({}){} {}",
            selected_command.usage_count,
            usage_count_to_emoji(selected_command.usage_count),
            usage_count_to_ticks(selected_command.usage_count)
        );
        let usage_count_span = Span::styled(usage_count, Style::default());

        let tags = format!("Tags: {}", selected_command.tags.join(", "));
        let tags_span = Span::styled(tags, Style::default());

        let last_used_dt: DateTime<Utc> = selected_command.last_used.into();
        let last_used = format!("Last used: {}", last_used_dt);
        let last_used_span = Span::styled(last_used, Style::default().fg(Color::DarkGray));

        let created_dt: DateTime<Utc> = selected_command.created.into();
        let created = format!("Created  : {}", created_dt);
        let created_span = Span::styled(created, Style::default().fg(Color::DarkGray));

        let updated_dt: DateTime<Utc> = selected_command.modified.into();
        let updated = format!("Updated  : {}", updated_dt);
        let updated_span = Span::styled(updated, Style::default().fg(Color::DarkGray));

        let text = vec![Line::from(usage_count_span), Line::from(tags_span), Line::from(last_used_span), Line::from(created_span), Line::from(updated_span)];

        frame.render_widget(
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Details ")), 
            rect
            );
    } else {
        frame.render_widget(
            Paragraph::new("No command selected")
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            rect,
        );
    }
}

/// Build the list of commands to display based on the trove in the current
/// app state. The list of commands is highlighted based on the search string
fn build_command_list_items(app: &App) -> Vec<Line> {
    let selected_index = app.commands.selected().unwrap_or(0);
    app.search_trove
        .commands
        .iter()
        .enumerate()
        .map(|(index, command)| {
            partial_highlighted_line(&command.name, &app.search_string, selected_index == index)
        })
        .collect()
}

fn usage_count_to_ticks(usage_count: usize) -> String {
    let usage_count_max_repeat = 30;
    let usage_count = usage_count.min(usage_count_max_repeat);
    let ticks = "▌";
    let ticks = ticks.repeat(usage_count);
    // if usage_count max is reached, add "..." to the end
    if usage_count == usage_count_max_repeat {
        return format!("{}...", ticks);
    } else {
        return format!("{}", ticks);
    }
}

fn usage_count_to_emoji(usage_count: usize) -> String {
    match usage_count {
        0 => "",
        1..=2 => "🌱",
        3..=4 => "🌿",
        5..=30 => "🔥",
        _ => "💀",
    }
    .to_string()
}
