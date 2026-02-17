pub mod screens;

use crate::app::state::{AppState, Screen};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Frame, Line};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(10),
        ])
        .split(frame.size());

    render_header(frame, state, root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(10)])
        .split(root[1]);

    render_nav(frame, state, body[0]);
    match state.screen {
        Screen::New => screens::new::render(frame, state, body[1]),
        Screen::Add => screens::add::render(frame, state, body[1]),
        Screen::Projects => screens::projects::render(frame, state, body[1]),
        Screen::Manage => screens::manage::render(frame, state, body[1]),
        Screen::Settings => screens::settings::render(frame, state, body[1]),
    }

    render_logs(frame, state, root[2]);
}

fn render_header(frame: &mut Frame, state: &AppState, area: Rect) {
    let text = format!(
        "[Tab/←/→] Navigate  [q] Quit  [c] Cancel task(Settings)  Screen={:?}  Status={} ",
        state.screen, state.status_line
    );
    frame.render_widget(
        Paragraph::new(text).style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        area,
    );
}

fn render_nav(frame: &mut Frame, state: &AppState, area: Rect) {
    let screens = [
        (Screen::New, "New"),
        (Screen::Add, "Add"),
        (Screen::Projects, "Projects"),
        (Screen::Manage, "Manage"),
        (Screen::Settings, "Settings"),
    ];

    let items = screens
        .iter()
        .map(|(_, name)| ListItem::new(*name))
        .collect::<Vec<_>>();

    let selected = screens
        .iter()
        .position(|(screen, _)| *screen == state.screen)
        .unwrap_or(0);
    let mut list_state = ListState::default().with_selected(Some(selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("VCC")
                .border_style(Style::default().fg(Color::LightBlue)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightYellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_logs(frame: &mut Frame, state: &AppState, area: Rect) {
    let lines = state
        .logs
        .iter()
        .map(|l| {
            let style = if l.text.contains(":err]") {
                Style::default().fg(Color::LightRed)
            } else if l.text.contains(":out]") {
                Style::default().fg(Color::White)
            } else if l.text.contains("Task") && l.text.contains("failed") {
                Style::default().fg(Color::LightRed)
            } else if l.text.contains("Task") && l.text.contains("done") {
                Style::default().fg(Color::LightGreen)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::styled(l.text.clone(), style)
        })
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Logs (Up/Down scroll)")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .scroll((state.log_scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
