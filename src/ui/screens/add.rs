use crate::app::state::AppState;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Frame, Line, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut lines = vec![
        Line::styled(
            "[a] Add single project folder",
            Style::default().fg(Color::LightGreen),
        ),
        Line::styled(
            "[f] Add projects from folder (one-level scan)",
            Style::default().fg(Color::LightGreen),
        ),
        Line::from(""),
        Line::styled(
            "Single project requires Packages/vpm-manifest.json in target.",
            Style::default().fg(Color::Gray),
        ),
        Line::styled(
            "Folder scan checks only direct child directories.",
            Style::default().fg(Color::Gray),
        ),
    ];

    if state.add_project_mode {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!(
                "Project path input: {} (Enter=add, Esc=cancel)",
                state.add_project_input
            ),
            Style::default().fg(Color::Black).bg(Color::LightYellow),
        ));
    }

    if state.add_folder_mode {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!(
                "Folder path input: {} (Enter=scan, Esc=cancel)",
                state.add_folder_input
            ),
            Style::default().fg(Color::Black).bg(Color::LightYellow),
        ));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Add")
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
