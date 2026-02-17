use crate::app::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Frame, Line};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    let mode = if state.search_mode {
        format!("Search: {}", state.search_query)
    } else {
        format!(
            "[/] Search  [Enter] Manage Project  [j/k] Select  query='{}'",
            state.search_query
        )
    };

    frame.render_widget(
        Paragraph::new(mode)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Projects")
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .wrap(Wrap { trim: true }),
        chunks[0],
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    let filtered = state.filtered_projects();
    let items = filtered
        .iter()
        .map(|p| {
            ListItem::new(format!("{} ({})", p.display_name, p.path.display()))
                .style(Style::default().fg(Color::White))
        })
        .collect::<Vec<_>>();

    let selected_idx = state
        .selected_project()
        .and_then(|selected| {
            filtered
                .iter()
                .position(|candidate| candidate.path == selected.path)
        })
        .unwrap_or(0);
    let mut list_state = ListState::default().with_selected(Some(selected_idx));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Project List")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightCyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, body[0], &mut list_state);

    let detail_lines = if let Some(project) = state.selected_project() {
        vec![
            Line::styled(
                format!("Name: {}", project.display_name),
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::styled(
                format!("Path: {}", project.path.display()),
                Style::default().fg(Color::Gray),
            ),
            Line::styled(
                format!("Tags: {}", project.tags.join(", ")),
                Style::default().fg(Color::LightMagenta),
            ),
            Line::styled(
                format!(
                    "Last opened: {}",
                    project
                        .last_opened
                        .clone()
                        .unwrap_or_else(|| "(none)".to_string())
                ),
                Style::default().fg(Color::White),
            ),
            Line::styled(
                "Action: Enter -> Manage Project",
                Style::default().fg(Color::LightGreen),
            ),
        ]
    } else {
        vec![Line::styled(
            "No projects registered",
            Style::default().fg(Color::DarkGray),
        )]
    };

    frame.render_widget(
        Paragraph::new(detail_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Summary")
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .wrap(Wrap { trim: false }),
        body[1],
    );
}
