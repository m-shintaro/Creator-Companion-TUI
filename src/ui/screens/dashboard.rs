use crate::app::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Frame, Line};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    let mode = if state.add_project_mode {
        format!("Add project path: {}", state.add_project_input)
    } else if state.search_mode {
        format!("Search: {}", state.search_query)
    } else {
        format!(
            "[/] Search  [a] Add project  [Enter] Open project  [j/k] Select  query='{}'",
            state.search_query
        )
    };

    frame.render_widget(
        Paragraph::new(mode)
            .block(Block::default().borders(Borders::ALL).title("Dashboard"))
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
        .map(|p| ListItem::new(format!("{} ({})", p.display_name, p.path.display())))
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
        .block(Block::default().borders(Borders::ALL).title("Projects"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, body[0], &mut list_state);

    let detail_lines = if let Some(project) = state.selected_project() {
        vec![
            Line::from(format!("Name: {}", project.display_name)),
            Line::from(format!("Path: {}", project.path.display())),
            Line::from(format!("Tags: {}", project.tags.join(", "))),
            Line::from(format!(
                "Last opened: {}",
                project
                    .last_opened
                    .clone()
                    .unwrap_or_else(|| "(none)".to_string())
            )),
        ]
    } else {
        vec![Line::from("No projects registered")]
    };

    frame.render_widget(
        Paragraph::new(detail_lines)
            .block(Block::default().borders(Borders::ALL).title("Summary"))
            .wrap(Wrap { trim: false }),
        body[1],
    );
}
