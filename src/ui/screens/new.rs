use crate::app::state::AppState;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Frame, Line, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let templates = ["Avatar", "World", "UdonSharp"];

    let mut lines = vec![
        Line::styled(
            "[j/k] Select template  [n] Create new project",
            Style::default().fg(Color::LightGreen),
        ),
        Line::styled(
            "If creation fails with template not found, run [t] vpm install templates in Settings.",
            Style::default().fg(Color::Gray),
        ),
        Line::from(""),
    ];

    for (idx, t) in templates.iter().enumerate() {
        let marker = if idx == state.new_project_template_idx {
            "▶"
        } else {
            " "
        };
        let style = if idx == state.new_project_template_idx {
            Style::default().fg(Color::Black).bg(Color::LightYellow)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::styled(format!("{marker} {t}"), style));
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!("Selected: {}", state.current_template()),
        Style::default().fg(Color::LightCyan),
    ));

    if state.new_project_mode {
        let editing = if state.new_project_edit_path {
            "path"
        } else {
            "name"
        };
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!(
                "New Project input (Tab switch field, Enter run, Esc cancel): editing={editing}"
            ),
            Style::default().fg(Color::LightMagenta),
        ));
        lines.push(Line::styled(
            format!("name: {}", state.new_project_name_input),
            if state.new_project_edit_path {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::Black).bg(Color::LightGreen)
            },
        ));
        lines.push(Line::styled(
            format!("path: {}", state.new_project_path_input),
            if state.new_project_edit_path {
                Style::default().fg(Color::Black).bg(Color::LightGreen)
            } else {
                Style::default().fg(Color::Gray)
            },
        ));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("New")
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
