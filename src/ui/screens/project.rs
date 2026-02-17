use crate::app::state::AppState;
use ratatui::layout::Rect;
use ratatui::prelude::{Frame, Line};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

const MODULAR_AVATAR_PACKAGE: &str = "nadena.dev.modular-avatar";
const LILTOON_PACKAGE: &str = "jp.lilxyzw.liltoon";

pub fn render(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut lines = vec![Line::from(
        "[r] Reload  [i] Install package  [d/D] Remove selected  [m] Toggle ModularAvatar  [l] Toggle lilToon  [v] Resolve",
    )];
    if state.add_package_mode {
        lines.push(Line::from(format!(
            "Install package input: {} (Enter=run, Esc=cancel)",
            state.add_package_input
        )));
    }

    if let Some(project) = state.selected_project() {
        lines.push(Line::from(format!("Project: {}", project.display_name)));
        lines.push(Line::from(format!("Path: {}", project.path.display())));
        let modular_installed = state
            .selected_project_manifest
            .as_ref()
            .map(|m| m.packages.iter().any(|p| p.name == MODULAR_AVATAR_PACKAGE))
            .unwrap_or(false);
        let liltoon_installed = state
            .selected_project_manifest
            .as_ref()
            .map(|m| m.packages.iter().any(|p| p.name == LILTOON_PACKAGE))
            .unwrap_or(false);
        lines.push(Line::from(format!(
            "Preset: [m] {} = {}",
            MODULAR_AVATAR_PACKAGE,
            if modular_installed {
                "installed"
            } else {
                "not installed"
            }
        )));
        lines.push(Line::from(format!(
            "Preset: [l] {} = {}",
            LILTOON_PACKAGE,
            if liltoon_installed {
                "installed"
            } else {
                "not installed"
            }
        )));

        if let Some(summary) = &state.selected_project_manifest {
            if summary.exists {
                lines.push(Line::from(format!("Packages: {}", summary.packages.len())));
                for (idx, pkg) in summary.packages.iter().enumerate() {
                    let marker = if idx == state.selected_manifest_package {
                        ">"
                    } else {
                        " "
                    };
                    lines.push(Line::from(format!(
                        "{marker} {} @ {}",
                        pkg.name, pkg.version
                    )));
                }
            } else {
                lines.push(Line::from("Manifest: missing"));
            }

            if let Some(message) = &summary.message {
                lines.push(Line::from(format!("Note: {message}")));
            }
        } else {
            lines.push(Line::from("Manifest not loaded yet"));
        }
    } else {
        lines.push(Line::from("No project selected"));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Project Detail"),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
