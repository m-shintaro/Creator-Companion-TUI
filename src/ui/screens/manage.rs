use crate::app::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Frame, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use std::collections::HashSet;

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(area);

    let mode = if state.available_package_search_mode {
        format!("Search available: {}", state.available_package_search)
    } else if state.add_package_mode {
        format!(
            "Install input: {} (Enter run, Esc cancel)",
            state.add_package_input
        )
    } else {
        "[h/l] Focus Installed/Available  [j/k] Move  [+/-] Add/Remove  [u] Update selected installed  [U] Update VRChat SDK  [/] Search  [r] Reload manifest  [R] Reload available  [v] Resolve".to_string()
    };

    frame.render_widget(
        Paragraph::new(mode)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Manage Project")
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .wrap(Wrap { trim: true }),
        chunks[0],
    );

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(54), Constraint::Percentage(46)])
        .split(chunks[1]);

    render_available(frame, state, panes[0]);
    render_installed(frame, state, panes[1]);
}

fn render_available(frame: &mut Frame, state: &AppState, area: Rect) {
    let installed_ids = installed_package_ids(state);
    let available = state.filtered_available_packages();
    let items = available
        .iter()
        .map(|p| {
            let installed = installed_ids.contains(&p.id);
            let base = if installed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };
            let text = format!(
                "{} [{}] {} - {} ({})",
                if installed { "-" } else { "+" },
                p.latest_version,
                p.id,
                p.display_name,
                p.repo_id
            );
            ListItem::new(text).style(base)
        })
        .collect::<Vec<_>>();

    let mut list_state = ListState::default().with_selected(Some(state.selected_available_package));
    let title = format!(
        "Available Packages {}  filter='{}'",
        if state.manage_focus_available {
            "(focus)"
        } else {
            ""
        },
        state.available_package_search
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if state.manage_focus_available {
                    Style::default().fg(Color::LightCyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_installed(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut items = Vec::new();
    if let Some(m) = &state.selected_project_manifest {
        if !m.exists {
            items.push(
                ListItem::new("Manifest missing: Packages/vpm-manifest.json")
                    .style(Style::default().fg(Color::LightRed)),
            );
        }
        if let Some(msg) = &m.message {
            items.push(ListItem::new(msg.clone()).style(Style::default().fg(Color::Red)));
        }
        items.extend(m.packages.iter().map(|p| {
            ListItem::new(format!("- [{}] {}", p.version, p.name))
                .style(Style::default().fg(Color::LightGreen))
        }));
    }

    let mut list_state = ListState::default().with_selected(Some(state.selected_manifest_package));
    let title = if let Some(project) = state.selected_project() {
        format!(
            "Installed ({}) {}",
            if state.manage_focus_available {
                ""
            } else {
                "(focus)"
            },
            project.display_name
        )
    } else {
        "Installed".to_string()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if state.manage_focus_available {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::LightMagenta)
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn installed_package_ids(state: &AppState) -> HashSet<String> {
    state
        .selected_project_manifest
        .as_ref()
        .map(|m| m.packages.iter().map(|p| p.name.clone()).collect())
        .unwrap_or_default()
}
