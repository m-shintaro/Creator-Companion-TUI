mod app;
mod events;
mod services;
mod ui;

use anyhow::Result;
use app::action::Action;
use app::effect::Effect;
use app::reducer::reduce;
use app::state::AppState;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use services::vpm::VpmClient;
use std::collections::HashMap;
use std::fs;
use std::io;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(cache_dir) = services::fs::cache_dir_path() {
        let _ = fs::create_dir_all(cache_dir);
    }

    let mut terminal = setup_terminal()?;

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    events::spawn_event_loop(action_tx.clone());

    let vpm = VpmClient;
    let mut running_tokens: HashMap<u64, CancellationToken> = HashMap::new();

    let mut state = AppState::default();
    let _ = action_tx.send(Action::Init);

    loop {
        terminal.draw(|frame| ui::render(frame, &state))?;

        if state.should_quit {
            break;
        }

        let Some(action) = action_rx.recv().await else {
            break;
        };

        if let Action::TaskDone { task_id, .. } = action.clone() {
            running_tokens.remove(&task_id);
        }

        let effects = reduce(&mut state, action);
        for effect in effects {
            handle_effect(effect, &action_tx, &vpm, &mut running_tokens).await;
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

async fn handle_effect(
    effect: Effect,
    action_tx: &mpsc::UnboundedSender<Action>,
    vpm: &VpmClient,
    running_tokens: &mut HashMap<u64, CancellationToken>,
) {
    match effect {
        Effect::LoadConfig => {
            let tx = action_tx.clone();
            tokio::spawn(async move {
                let result = services::fs::load_config().map_err(|e| e.to_string());
                let _ = tx.send(Action::ConfigLoaded(result));
            });
        }
        Effect::SaveConfig(config) => {
            let tx = action_tx.clone();
            tokio::spawn(async move {
                let result = services::fs::save_config(&config).map_err(|e| e.to_string());
                let _ = tx.send(Action::ConfigSaved(result));
            });
        }
        Effect::LoadAvailablePackages => {
            let tx = action_tx.clone();
            tokio::spawn(async move {
                let result = services::fs::load_available_packages_from_vcc_cache()
                    .map_err(|e| e.to_string());
                let _ = tx.send(Action::AvailablePackagesLoaded(result));
            });
        }
        Effect::ScanProjectsFolder { root } => {
            let tx = action_tx.clone();
            tokio::spawn(async move {
                let result =
                    services::fs::scan_projects_one_level(&root).map_err(|e| e.to_string());
                let _ = tx.send(Action::FolderScanned(result));
            });
        }
        Effect::ReadManifest { project_path } => {
            let tx = action_tx.clone();
            tokio::spawn(async move {
                let result = services::fs::read_manifest(&project_path).map_err(|e| e.to_string());
                let _ = tx.send(Action::ManifestLoaded(result));
            });
        }
        Effect::RunVpmCommand {
            task_id,
            label,
            args,
        } => {
            let tx = action_tx.clone();
            let client = vpm.clone();
            let token = CancellationToken::new();
            running_tokens.insert(task_id, token.clone());

            tokio::spawn(async move {
                let command_result = client
                    .run_command(task_id, label.clone(), args, token, tx.clone())
                    .await;

                if let Err(err) = command_result {
                    let _ = tx.send(Action::TaskDone {
                        task_id,
                        success: false,
                        cancelled: false,
                        exit_code: None,
                        error: Some(err.to_string()),
                    });
                }
            });
        }
        Effect::CancelTask { task_id } => {
            if let Some(token) = running_tokens.get(&task_id) {
                token.cancel();
            } else {
                let _ = action_tx.send(Action::TaskOutput {
                    task_id,
                    stream: app::action::OutputStream::Stderr,
                    line: format!("task #{task_id} is not running"),
                });
            }
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
