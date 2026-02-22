use crate::app::state::{AppState, TaskState};
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Frame, Line, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut lines = Vec::new();

    // --- Prerequisites ---
    lines.push(Line::styled(
        "Prerequisites:",
        Style::default().fg(Color::LightCyan),
    ));

    // dotnet
    let (dotnet_text, dotnet_color) = match state.system_checks.dotnet_version.as_deref() {
        Some("not installed") => (
            "  dotnet: not installed  -> https://dotnet.microsoft.com/download/dotnet/8.0"
                .to_string(),
            Color::LightRed,
        ),
        Some(v) => (format!("  dotnet: {v}"), Color::LightGreen),
        None => ("  dotnet: checking...".to_string(), Color::Yellow),
    };
    lines.push(Line::styled(dotnet_text, Style::default().fg(dotnet_color)));

    // vpm
    let (vpm_text, vpm_color) = match state.system_checks.vpm_version.as_deref() {
        Some("not installed") => (
            "  vpm:    not installed  -> press [i] to install".to_string(),
            Color::LightRed,
        ),
        Some(v) => (format!("  vpm:    {v}"), Color::LightGreen),
        None => ("  vpm:    checking...".to_string(), Color::Yellow),
    };
    lines.push(Line::styled(vpm_text, Style::default().fg(vpm_color)));

    // hub
    let (hub_text, hub_color) = match state.system_checks.hub_check.as_deref() {
        Some(v) if v == "ok" => (format!("  hub:    {v}"), Color::LightGreen),
        Some(v) => (format!("  hub:    {v}"), Color::LightRed),
        None => (
            "  hub:    (press [h] to check)".to_string(),
            Color::DarkGray,
        ),
    };
    lines.push(Line::styled(hub_text, Style::default().fg(hub_color)));

    // unity
    let (unity_text, unity_color) = match state.system_checks.unity_check.as_deref() {
        Some(v) if v == "ok" => (format!("  unity:  {v}"), Color::LightGreen),
        Some(v) => (format!("  unity:  {v}"), Color::LightRed),
        None => (
            "  unity:  (press [u] to check)".to_string(),
            Color::DarkGray,
        ),
    };
    lines.push(Line::styled(unity_text, Style::default().fg(unity_color)));

    lines.push(Line::from(""));

    // --- Install / Update ---
    lines.push(Line::styled(
        "[i] Install vpm CLI  [p] Update vpm CLI  (auto-installs dotnet if missing)",
        Style::default().fg(Color::LightGreen),
    ));

    lines.push(Line::from(""));

    // --- Packages / Repos ---
    lines.push(Line::styled(
        "Packages/Repos:",
        Style::default().fg(Color::LightCyan),
    ));
    lines.push(Line::styled(
        "[1] add repo nadena  [2] add repo liltoon",
        Style::default().fg(Color::LightGreen),
    ));
    lines.push(Line::styled(
        "[a] add repo(custom url)  [r] vpm list repos",
        Style::default().fg(Color::LightGreen),
    ));

    lines.push(Line::from(""));

    // --- Environment ---
    lines.push(Line::styled(
        "Environment:",
        Style::default().fg(Color::LightCyan),
    ));
    lines.push(Line::styled(
        "[t] install templates  [h] check hub  [u] check unity",
        Style::default().fg(Color::White),
    ));
    lines.push(Line::styled(
        "[l] list unity  [s] open settingsFolder",
        Style::default().fg(Color::White),
    ));
    lines.push(Line::styled(
        "[c] cancel latest running task",
        Style::default().fg(Color::LightRed),
    ));

    lines.push(Line::from(""));

    // --- Recent tasks ---
    lines.push(Line::styled(
        "Recent tasks:",
        Style::default().fg(Color::LightCyan),
    ));
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

    // --- Repo input overlay ---
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
