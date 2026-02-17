use crate::app::state::{AppState, TaskState};
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Frame, Line, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut lines = vec![
        Line::styled("Packages/Repos:", Style::default().fg(Color::LightCyan)),
        Line::styled(
            "[1] add repo nadena  [2] add repo liltoon",
            Style::default().fg(Color::LightGreen),
        ),
        Line::styled(
            "[a] add repo(custom url)  [r] vpm list repos",
            Style::default().fg(Color::LightGreen),
        ),
        Line::from(""),
        Line::styled("Environment checks:", Style::default().fg(Color::LightCyan)),
        Line::styled(
            "[t] vpm install templates",
            Style::default().fg(Color::White),
        ),
        Line::styled("[h] vpm check hub", Style::default().fg(Color::White)),
        Line::styled("[u] vpm check unity", Style::default().fg(Color::White)),
        Line::styled("[l] vpm list unity", Style::default().fg(Color::White)),
        Line::styled(
            "[s] vpm open settingsFolder",
            Style::default().fg(Color::White),
        ),
        Line::styled(
            "[c] cancel latest running task",
            Style::default().fg(Color::LightRed),
        ),
        Line::from(""),
        Line::styled(
            format!(
                "vpm --version: {}",
                state
                    .system_checks
                    .vpm_version
                    .clone()
                    .unwrap_or_else(|| "(not checked yet)".to_string())
            ),
            Style::default().fg(Color::Yellow),
        ),
        Line::styled(
            format!(
                "check hub: {}",
                state
                    .system_checks
                    .hub_check
                    .clone()
                    .unwrap_or_else(|| "(not run)".to_string())
            ),
            Style::default().fg(Color::Yellow),
        ),
        Line::styled(
            format!(
                "check unity: {}",
                state
                    .system_checks
                    .unity_check
                    .clone()
                    .unwrap_or_else(|| "(not run)".to_string())
            ),
            Style::default().fg(Color::Yellow),
        ),
        Line::from(""),
        Line::styled("Recent tasks:", Style::default().fg(Color::LightCyan)),
    ];

    for task in state.tasks.iter().rev().take(8) {
        let (status, color) = match task.state {
            TaskState::Running => ("running", Color::LightBlue),
            TaskState::Success => ("success", Color::LightGreen),
            TaskState::Failed => ("failed", Color::LightRed),
            TaskState::Cancelled => ("cancelled", Color::LightMagenta),
        };
        lines.push(Line::styled(
            format!("- #{} {} ({})", task.id, task.label, status),
            Style::default().fg(color),
        ));
    }
    if state.add_repo_mode {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!(
                "Repo URL input: {} (Enter=add, Esc=cancel)",
                state.add_repo_input
            ),
            Style::default().fg(Color::Black).bg(Color::LightYellow),
        ));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Settings")
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
