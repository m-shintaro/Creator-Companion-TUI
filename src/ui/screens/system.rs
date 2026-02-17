use crate::app::state::{AppState, TaskState};
use ratatui::layout::Rect;
use ratatui::prelude::{Frame, Line};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut lines = vec![
        Line::from("[t] vpm install templates"),
        Line::from("[h] vpm check hub"),
        Line::from("[u] vpm check unity"),
        Line::from("[l] vpm list unity"),
        Line::from("[s] vpm open settingsFolder"),
        Line::from("[1] add repo nadena  [2] add repo liltoon"),
        Line::from("[a] add repo(custom url)  [r] vpm list repos"),
        Line::from("[c] cancel latest running task"),
        Line::from(""),
        Line::from(format!(
            "vpm --version: {}",
            state
                .system_checks
                .vpm_version
                .clone()
                .unwrap_or_else(|| "(not checked yet)".to_string())
        )),
        Line::from(format!(
            "check hub: {}",
            state
                .system_checks
                .hub_check
                .clone()
                .unwrap_or_else(|| "(not run)".to_string())
        )),
        Line::from(format!(
            "check unity: {}",
            state
                .system_checks
                .unity_check
                .clone()
                .unwrap_or_else(|| "(not run)".to_string())
        )),
        Line::from(""),
        Line::from("Recent tasks:"),
    ];

    for task in state.tasks.iter().rev().take(8) {
        let status = match task.state {
            TaskState::Running => "running",
            TaskState::Success => "success",
            TaskState::Failed => "failed",
            TaskState::Cancelled => "cancelled",
        };
        lines.push(Line::from(format!(
            "- #{} {} ({})",
            task.id, task.label, status
        )));
    }
    if state.add_repo_mode {
        lines.push(Line::from(""));
        lines.push(Line::from(format!(
            "Repo URL input: {} (Enter=add, Esc=cancel)",
            state.add_repo_input
        )));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("System"))
            .wrap(Wrap { trim: false }),
        area,
    );
}
