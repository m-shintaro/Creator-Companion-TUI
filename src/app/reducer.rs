use crate::app::action::{Action, OutputStream};
use crate::app::effect::Effect;
use crate::app::state::{AppConfig, AppState, ProjectMeta, Screen, TaskRecord, TaskState};
use crossterm::event::{KeyCode, KeyModifiers};
use std::path::PathBuf;

const NADENA_REPO_URL: &str = "https://vpm.nadena.dev/vpm.json";
const LILTOON_REPO_URL: &str = "https://lilxyzw.github.io/vpm-repos/vpm.json";

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::Init => vec![
            Effect::LoadConfig,
            Effect::LoadAvailablePackages,
            enqueue_system_task(state, "vpm --version", ["--version"]),
            enqueue_dotnet_task(state, "dotnet --version", ["--version"]),
        ],
        Action::Tick => {
            state.tick_count = state.tick_count.saturating_add(1);
            vec![]
        }
        Action::Key(key) => on_key(state, key),
        Action::ConfigLoaded(result) => {
            match result {
                Ok(config) => {
                    let original_count = config.projects.len();
                    state.projects = config
                        .projects
                        .into_iter()
                        .filter(|p| p.path.exists())
                        .collect::<Vec<_>>();
                    state.selected_project_clamped();
                    let removed = original_count.saturating_sub(state.projects.len());
                    state.status_line = format!("Loaded {} project(s)", state.projects.len());
                    let mut effects = Vec::new();
                    if removed > 0 {
                        state.push_log(
                            None,
                            format!(
                                "[config] removed {removed} missing project(s) from local list"
                            ),
                        );
                        effects.push(Effect::SaveConfig(AppConfig {
                            projects: state.projects.clone(),
                        }));
                    }
                    if let Some(project) = state.selected_project() {
                        effects.push(Effect::ReadManifest {
                            project_path: project.path.clone(),
                        });
                    }
                    return effects;
                }
                Err(err) => {
                    state.push_log(None, format!("[config] load failed: {err}"));
                    state.status_line = "Config load failed; using defaults".to_string();
                }
            }
            vec![]
        }
        Action::ConfigSaved(result) => {
            if let Err(err) = result {
                state.push_log(None, format!("[config] save failed: {err}"));
                state.status_line = "Config save failed".to_string();
            }
            vec![]
        }
        Action::FolderScanned(result) => match result {
            Ok(paths) => {
                let mut added = 0;
                for path in paths {
                    if state.projects.iter().any(|p| p.path == path) {
                        continue;
                    }
                    let display_name = path
                        .file_name()
                        .map(|v| v.to_string_lossy().to_string())
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| path.to_string_lossy().to_string());
                    state.projects.push(ProjectMeta {
                        path,
                        display_name,
                        tags: vec![],
                        last_opened: None,
                    });
                    added += 1;
                }
                state.selected_project_clamped();
                state.status_line = format!("Add folder scan complete: {added} project(s) added");
                vec![Effect::SaveConfig(AppConfig {
                    projects: state.projects.clone(),
                })]
            }
            Err(err) => {
                state.status_line = "Folder scan failed".to_string();
                state.push_log(None, format!("[add] folder scan failed: {err}"));
                vec![]
            }
        },
        Action::AvailablePackagesLoaded(result) => {
            match result {
                Ok(packages) => {
                    state.available_packages = packages;
                    state.selected_available_package_clamped();
                    state.status_line = format!(
                        "Loaded {} available package(s)",
                        state.available_packages.len()
                    );
                }
                Err(err) => {
                    state.push_log(None, format!("[packages] load failed: {err}"));
                    state.status_line = "Available package load failed".to_string();
                }
            }
            vec![]
        }
        Action::ManifestLoaded(result) => {
            match result {
                Ok(summary) => {
                    state.selected_project_manifest = Some(summary);
                    state.selected_manifest_package_clamped();
                }
                Err(err) => {
                    state.selected_project_manifest = None;
                    state.selected_manifest_package = 0;
                    state.push_log(None, format!("[manifest] failed: {err}"));
                }
            }
            vec![]
        }
        Action::TaskOutput {
            task_id,
            stream,
            line,
        } => {
            let prefix = match stream {
                OutputStream::Stdout => "out",
                OutputStream::Stderr => "err",
            };
            state.push_log(Some(task_id), format!("[{task_id}:{prefix}] {line}"));

            if matches!(stream, OutputStream::Stdout) {
                if task_is_label(state, task_id, "vpm --version") {
                    state.system_checks.vpm_version = Some(line);
                } else if task_is_label(state, task_id, "dotnet --version") {
                    state.system_checks.dotnet_version = Some(line);
                }
            }
            vec![]
        }
        Action::TaskDone {
            task_id,
            success,
            cancelled,
            exit_code,
            error,
        } => {
            let mut next_effects = Vec::new();
            let mut deferred_log: Option<String> = None;
            let mut recheck_vpm = false;
            if let Some(task) = state.tasks.iter_mut().find(|t| t.id == task_id) {
                task.exit_code = exit_code;
                task.error = error.clone();
                task.state = if cancelled {
                    TaskState::Cancelled
                } else if success {
                    TaskState::Success
                } else {
                    TaskState::Failed
                };

                if task.label == "vpm --version" && !success {
                    state.system_checks.vpm_version = Some("not installed".to_string());
                }
                if task.label == "dotnet --version" && !success {
                    state.system_checks.dotnet_version = Some("not installed".to_string());
                }
                if task.label == "vpm check hub" {
                    state.system_checks.hub_check = Some(task_state_text(task));
                }
                if task.label == "vpm check unity" {
                    state.system_checks.unity_check = Some(task_state_text(task));
                }
                if task.label == "vpm list unity" {
                    let lines = state
                        .logs
                        .iter()
                        .filter(|l| l.task_id == Some(task_id))
                        .map(|l| l.text.clone())
                        .collect::<Vec<_>>();
                    state.system_checks.unity_list = lines;
                }
                if task.label.starts_with("vpm add repo ") {
                    next_effects.push(Effect::LoadAvailablePackages);
                }
                if (task.label == "dotnet tool install vpm"
                    || task.label == "dotnet tool update vpm")
                    && success
                {
                    recheck_vpm = true;
                }
                if success {
                    if let Some(project) = task.pending_add_project.clone() {
                        if project.path.exists() {
                            if !state.projects.iter().any(|p| p.path == project.path) {
                                state.projects.push(project.clone());
                                state.selected_project = state.projects.len() - 1;
                                next_effects.push(Effect::SaveConfig(AppConfig {
                                    projects: state.projects.clone(),
                                }));
                            }
                        } else {
                            deferred_log = Some(format!(
                                "Project path was not created: {}",
                                project.path.display()
                            ));
                        }
                    }
                }

                if let Some(path) = task.refresh_manifest_path.clone() {
                    next_effects.push(Effect::ReadManifest { project_path: path });
                }
            }
            if let Some(line) = deferred_log {
                state.push_log(Some(task_id), line);
            }
            if recheck_vpm {
                next_effects.push(enqueue_system_task(
                    state,
                    "vpm --version",
                    ["--version"],
                ));
            }

            let message = if cancelled {
                format!("Task {task_id} cancelled")
            } else if success {
                format!("Task {task_id} done")
            } else {
                format!("Task {task_id} failed")
            };
            state.status_line = message.clone();
            state.push_log(Some(task_id), message);
            next_effects
        }
    }
}

fn on_key(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        state.should_quit = true;
        return vec![];
    }

    if state.add_project_mode {
        return on_add_project_input(state, key);
    }
    if state.add_folder_mode {
        return on_add_folder_input(state, key);
    }
    if state.search_mode {
        return on_search_input(state, key);
    }
    if state.add_repo_mode {
        return on_add_repo_input(state, key);
    }
    if state.add_package_mode {
        return on_add_package_input(state, key);
    }
    if state.available_package_search_mode {
        return on_available_package_search_input(state, key);
    }
    if state.new_project_mode {
        return on_new_project_input(state, key);
    }

    match key.code {
        KeyCode::Char('q') => {
            state.should_quit = true;
            vec![]
        }
        KeyCode::Tab => {
            state.screen = state.screen.next();
            vec![]
        }
        KeyCode::Right => {
            state.screen = state.screen.next();
            vec![]
        }
        KeyCode::Left => {
            state.screen = state.screen.prev();
            vec![]
        }
        KeyCode::Up => {
            state.log_scroll = state.log_scroll.saturating_sub(1);
            vec![]
        }
        KeyCode::Down => {
            let max_scroll = (state.logs.len() as u16).saturating_sub(1);
            state.log_scroll = state.log_scroll.saturating_add(1).min(max_scroll);
            vec![]
        }
        _ => match state.screen {
            Screen::New => on_new_key(state, key),
            Screen::Add => on_add_key(state, key),
            Screen::Projects => on_projects_key(state, key),
            Screen::Manage => on_manage_key(state, key),
            Screen::Settings => on_settings_key(state, key),
        },
    }
}

fn on_search_input(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Esc => state.search_mode = false,
        KeyCode::Enter => state.search_mode = false,
        KeyCode::Backspace => {
            state.search_query.pop();
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                state.search_query.push(c);
            }
        }
        _ => {}
    }
    vec![]
}

fn on_add_project_input(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Esc => {
            state.add_project_mode = false;
            state.add_project_input.clear();
            vec![]
        }
        KeyCode::Backspace => {
            state.add_project_input.pop();
            vec![]
        }
        KeyCode::Enter => {
            let input = state.add_project_input.trim();
            if input.is_empty() {
                state.status_line = "Project path is empty".to_string();
                return vec![];
            }

            let path = PathBuf::from(input);
            if state.projects.iter().any(|p| p.path == path) {
                state.status_line = "Project already exists".to_string();
                state.add_project_mode = false;
                state.add_project_input.clear();
                return vec![];
            }

            let display_name = path
                .file_name()
                .map(|v| v.to_string_lossy().to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| input.to_string());

            state.projects.push(ProjectMeta {
                path: path.clone(),
                display_name,
                tags: vec![],
                last_opened: None,
            });
            state.selected_project = state.projects.len() - 1;
            state.add_project_mode = false;
            state.add_project_input.clear();
            state.status_line = "Project added".to_string();

            vec![
                Effect::SaveConfig(AppConfig {
                    projects: state.projects.clone(),
                }),
                Effect::ReadManifest { project_path: path },
            ]
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                state.add_project_input.push(c);
            }
            vec![]
        }
        _ => vec![],
    }
}

fn on_add_folder_input(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Esc => {
            state.add_folder_mode = false;
            state.add_folder_input.clear();
            vec![]
        }
        KeyCode::Backspace => {
            state.add_folder_input.pop();
            vec![]
        }
        KeyCode::Enter => {
            let root = state.add_folder_input.trim().to_string();
            if root.is_empty() {
                state.status_line = "Folder path is empty".to_string();
                return vec![];
            }
            state.add_folder_mode = false;
            state.add_folder_input.clear();
            vec![Effect::ScanProjectsFolder {
                root: PathBuf::from(root),
            }]
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                state.add_folder_input.push(c);
            }
            vec![]
        }
        _ => vec![],
    }
}

fn on_add_repo_input(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Esc => {
            state.add_repo_mode = false;
            state.add_repo_input.clear();
            vec![]
        }
        KeyCode::Backspace => {
            state.add_repo_input.pop();
            vec![]
        }
        KeyCode::Enter => {
            let repo = state.add_repo_input.trim().to_string();
            if repo.is_empty() {
                state.status_line = "Repo URL is empty".to_string();
                return vec![];
            }
            state.add_repo_mode = false;
            state.add_repo_input.clear();
            vec![enqueue_project_task(
                state,
                format!("vpm add repo {repo}"),
                vec!["add".to_string(), "repo".to_string(), repo],
                None,
                None,
            )]
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                state.add_repo_input.push(c);
            }
            vec![]
        }
        _ => vec![],
    }
}

fn on_add_package_input(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Esc => {
            state.add_package_mode = false;
            state.add_package_input.clear();
            vec![]
        }
        KeyCode::Backspace => {
            state.add_package_input.pop();
            vec![]
        }
        KeyCode::Enter => {
            let package = state.add_package_input.trim().to_string();
            if package.is_empty() {
                state.status_line = "Package name is empty".to_string();
                return vec![];
            }

            if let Some(project) = state.selected_project() {
                let project_path = project.path.clone();
                state.add_package_mode = false;
                state.add_package_input.clear();
                return vec![enqueue_project_task(
                    state,
                    format!("vpm add package {package}"),
                    vec![
                        "add".to_string(),
                        "package".to_string(),
                        package,
                        "-p".to_string(),
                        project_path.to_string_lossy().to_string(),
                    ],
                    Some(project_path),
                    None,
                )];
            }

            state.status_line = "No project selected".to_string();
            vec![]
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                state.add_package_input.push(c);
            }
            vec![]
        }
        _ => vec![],
    }
}

fn on_available_package_search_input(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
) -> Vec<Effect> {
    match key.code {
        KeyCode::Esc => state.available_package_search_mode = false,
        KeyCode::Enter => state.available_package_search_mode = false,
        KeyCode::Backspace => {
            state.available_package_search.pop();
            state.selected_available_package_clamped();
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                state.available_package_search.push(c);
                state.selected_available_package_clamped();
            }
        }
        _ => {}
    }
    vec![]
}

fn on_new_project_input(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Esc => {
            state.new_project_mode = false;
            vec![]
        }
        KeyCode::Tab => {
            state.new_project_edit_path = !state.new_project_edit_path;
            vec![]
        }
        KeyCode::Backspace => {
            if state.new_project_edit_path {
                state.new_project_path_input.pop();
            } else {
                state.new_project_name_input.pop();
            }
            vec![]
        }
        KeyCode::Enter => {
            let name = state.new_project_name_input.trim().to_string();
            let path = state.new_project_path_input.trim().to_string();
            if name.is_empty() || path.is_empty() {
                state.status_line = "Project name/path is required".to_string();
                return vec![];
            }
            let template = state.current_template().to_string();

            let project_root = PathBuf::from(&path).join(&name);
            let pending_project = ProjectMeta {
                path: project_root.clone(),
                display_name: name.clone(),
                tags: vec![],
                last_opened: None,
            };

            state.new_project_mode = false;
            state.new_project_name_input.clear();
            state.new_project_path_input.clear();
            state.screen = Screen::Projects;

            vec![enqueue_project_task(
                state,
                format!("vpm new {name} {template} -p {path}"),
                vec!["new".to_string(), name, template, "-p".to_string(), path],
                Some(project_root),
                Some(pending_project),
            )]
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                if state.new_project_edit_path {
                    state.new_project_path_input.push(c);
                } else {
                    state.new_project_name_input.push(c);
                }
            }
            vec![]
        }
        _ => vec![],
    }
}

fn on_new_key(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Char('j') => {
            state.new_project_template_idx = (state.new_project_template_idx + 1).min(2);
            vec![]
        }
        KeyCode::Char('k') => {
            state.new_project_template_idx = state.new_project_template_idx.saturating_sub(1);
            vec![]
        }
        KeyCode::Char('n') => {
            state.new_project_mode = true;
            state.new_project_edit_path = false;
            vec![]
        }
        _ => vec![],
    }
}

fn on_add_key(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Char('a') => {
            state.add_project_mode = true;
            state.add_project_input.clear();
            vec![]
        }
        KeyCode::Char('f') => {
            state.add_folder_mode = true;
            state.add_folder_input.clear();
            vec![]
        }
        _ => vec![],
    }
}

fn on_projects_key(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Char('/') => {
            state.search_mode = true;
            vec![]
        }
        KeyCode::Char('a') => {
            state.screen = Screen::Add;
            vec![]
        }
        KeyCode::Char('j') => {
            let len = state.filtered_projects().len();
            if len > 0 {
                state.selected_project = (state.selected_project + 1).min(len - 1);
            }
            vec![]
        }
        KeyCode::Char('k') => {
            state.selected_project = state.selected_project.saturating_sub(1);
            vec![]
        }
        KeyCode::Enter => {
            state.screen = Screen::Manage;
            if let Some(project) = state.selected_project() {
                return vec![Effect::ReadManifest {
                    project_path: project.path.clone(),
                }];
            }
            vec![]
        }
        _ => vec![],
    }
}

fn on_manage_key(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Char('r') => {
            if let Some(project) = state.selected_project() {
                return vec![Effect::ReadManifest {
                    project_path: project.path.clone(),
                }];
            }
            vec![]
        }
        KeyCode::Char('R') => vec![Effect::LoadAvailablePackages],
        KeyCode::Char('/') => {
            state.available_package_search_mode = true;
            vec![]
        }
        KeyCode::Char('h') => {
            state.manage_focus_available = true;
            vec![]
        }
        KeyCode::Char('l') => {
            state.manage_focus_available = false;
            vec![]
        }
        KeyCode::Char('i') => {
            state.add_package_mode = true;
            state.add_package_input.clear();
            vec![]
        }
        KeyCode::Char('j') => {
            if state.manage_focus_available {
                let len = state.filtered_available_packages().len();
                if len > 0 {
                    state.selected_available_package =
                        (state.selected_available_package + 1).min(len - 1);
                }
            } else if let Some(summary) = &state.selected_project_manifest {
                if !summary.packages.is_empty() {
                    state.selected_manifest_package =
                        (state.selected_manifest_package + 1).min(summary.packages.len() - 1);
                }
            }
            vec![]
        }
        KeyCode::Char('k') => {
            if state.manage_focus_available {
                state.selected_available_package =
                    state.selected_available_package.saturating_sub(1);
            } else {
                state.selected_manifest_package = state.selected_manifest_package.saturating_sub(1);
            }
            vec![]
        }
        KeyCode::Char('+')
        | KeyCode::Char('=')
        | KeyCode::Char(':')
        | KeyCode::Char('＋')
        | KeyCode::Char('a') => install_selected_available_package(state),
        KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Char('－') | KeyCode::Char('x') => {
            remove_selected_available_package(state, false)
        }
        KeyCode::Char('u') => update_selected_installed_package(state),
        KeyCode::Char('U') => update_vrchat_sdk_package(state),
        KeyCode::Char('d') => remove_selected_package(state, false),
        KeyCode::Char('D') => remove_selected_package(state, true),
        KeyCode::Char('v') => resolve_selected_project(state),
        _ => vec![],
    }
}

fn on_settings_key(state: &mut AppState, key: crossterm::event::KeyEvent) -> Vec<Effect> {
    match key.code {
        KeyCode::Char('t') => vec![enqueue_system_task(
            state,
            "vpm install templates",
            ["install", "templates"],
        )],
        KeyCode::Char('h') => vec![enqueue_system_task(
            state,
            "vpm check hub",
            ["check", "hub"],
        )],
        KeyCode::Char('u') => vec![enqueue_system_task(
            state,
            "vpm check unity",
            ["check", "unity"],
        )],
        KeyCode::Char('l') => vec![enqueue_system_task(
            state,
            "vpm list unity",
            ["list", "unity"],
        )],
        KeyCode::Char('s') => vec![enqueue_system_task(
            state,
            "vpm open settingsFolder",
            ["open", "settingsFolder"],
        )],
        KeyCode::Char('1') => vec![enqueue_project_task(
            state,
            format!("vpm add repo {NADENA_REPO_URL}"),
            vec![
                "add".to_string(),
                "repo".to_string(),
                NADENA_REPO_URL.to_string(),
            ],
            None,
            None,
        )],
        KeyCode::Char('2') => vec![enqueue_project_task(
            state,
            format!("vpm add repo {LILTOON_REPO_URL}"),
            vec![
                "add".to_string(),
                "repo".to_string(),
                LILTOON_REPO_URL.to_string(),
            ],
            None,
            None,
        )],
        KeyCode::Char('a') => {
            state.add_repo_mode = true;
            state.add_repo_input.clear();
            vec![]
        }
        KeyCode::Char('r') => vec![enqueue_system_task(
            state,
            "vpm list repos",
            ["list", "repos"],
        )],
        KeyCode::Char('i') => {
            if state.system_checks.dotnet_version.as_deref() == Some("not installed") {
                state.status_line =
                    "dotnet required. Install .NET 8 SDK: https://dotnet.microsoft.com/download/dotnet/8.0".to_string();
                vec![]
            } else {
                vec![enqueue_dotnet_task(
                    state,
                    "dotnet tool install vpm",
                    ["tool", "install", "--global", "vrchat.vpm.cli"],
                )]
            }
        }
        KeyCode::Char('p') => {
            if state.system_checks.dotnet_version.as_deref() == Some("not installed") {
                state.status_line =
                    "dotnet required. Install .NET 8 SDK: https://dotnet.microsoft.com/download/dotnet/8.0".to_string();
                vec![]
            } else {
                vec![enqueue_dotnet_task(
                    state,
                    "dotnet tool update vpm",
                    ["tool", "update", "--global", "vrchat.vpm.cli"],
                )]
            }
        }
        KeyCode::Char('c') => {
            if let Some(task) = state
                .tasks
                .iter()
                .rev()
                .find(|t| t.state == TaskState::Running)
            {
                return vec![Effect::CancelTask { task_id: task.id }];
            }
            vec![]
        }
        _ => vec![],
    }
}

fn install_selected_available_package(state: &mut AppState) -> Vec<Effect> {
    let Some(project) = state.selected_project() else {
        state.status_line = "No project selected".to_string();
        return vec![];
    };
    let Some(pkg) = state.selected_available_package() else {
        state.status_line = "No available package selected".to_string();
        return vec![];
    };

    let project_path = project.path.clone();
    let package_name = pkg.id.clone();
    let args = vec![
        "add".to_string(),
        "package".to_string(),
        package_name.clone(),
        "-p".to_string(),
        project_path.to_string_lossy().to_string(),
    ];

    vec![enqueue_project_task(
        state,
        format!("vpm add package {package_name}"),
        args,
        Some(project_path),
        None,
    )]
}

fn remove_selected_available_package(state: &mut AppState, force: bool) -> Vec<Effect> {
    let Some(project) = state.selected_project() else {
        state.status_line = "No project selected".to_string();
        return vec![];
    };
    let Some(pkg) = state.selected_available_package() else {
        state.status_line = "No available package selected".to_string();
        return vec![];
    };

    let project_path = project.path.clone();
    let package_name = pkg.id.clone();
    let mut args = vec![
        "remove".to_string(),
        "package".to_string(),
        package_name.clone(),
        "-p".to_string(),
        project_path.to_string_lossy().to_string(),
    ];
    if force {
        args.push("--force".to_string());
    }
    vec![enqueue_project_task(
        state,
        if force {
            format!("vpm remove package {package_name} --force")
        } else {
            format!("vpm remove package {package_name}")
        },
        args,
        Some(project_path),
        None,
    )]
}

fn update_selected_installed_package(state: &mut AppState) -> Vec<Effect> {
    let Some(project) = state.selected_project() else {
        state.status_line = "No project selected".to_string();
        return vec![];
    };
    let Some(pkg) = state.selected_manifest_package() else {
        state.status_line = "No installed package selected".to_string();
        return vec![];
    };
    let project_path = project.path.clone();
    let package_name = pkg.name.clone();
    vec![enqueue_project_task(
        state,
        format!("vpm add package {package_name} (update)"),
        vec![
            "add".to_string(),
            "package".to_string(),
            package_name,
            "-p".to_string(),
            project_path.to_string_lossy().to_string(),
        ],
        Some(project_path),
        None,
    )]
}

fn update_vrchat_sdk_package(state: &mut AppState) -> Vec<Effect> {
    let Some(project) = state.selected_project() else {
        state.status_line = "No project selected".to_string();
        return vec![];
    };
    let manifest = match &state.selected_project_manifest {
        Some(v) => v,
        None => {
            state.status_line = "Manifest is not loaded".to_string();
            return vec![];
        }
    };

    let sdk_pkg = if manifest
        .packages
        .iter()
        .any(|p| p.name == "com.vrchat.avatars")
    {
        "com.vrchat.avatars"
    } else if manifest
        .packages
        .iter()
        .any(|p| p.name == "com.vrchat.worlds")
    {
        "com.vrchat.worlds"
    } else {
        state.status_line = "No VRChat SDK package found in project".to_string();
        return vec![];
    };

    let project_path = project.path.clone();
    vec![enqueue_project_task(
        state,
        format!("vpm add package {sdk_pkg} (sdk update)"),
        vec![
            "add".to_string(),
            "package".to_string(),
            sdk_pkg.to_string(),
            "-p".to_string(),
            project_path.to_string_lossy().to_string(),
        ],
        Some(project_path),
        None,
    )]
}

fn resolve_selected_project(state: &mut AppState) -> Vec<Effect> {
    let Some(project) = state.selected_project() else {
        state.status_line = "No project selected".to_string();
        return vec![];
    };
    let project_path = project.path.clone();
    vec![enqueue_project_task(
        state,
        format!("vpm resolve project {}", project_path.display()),
        vec![
            "resolve".to_string(),
            "project".to_string(),
            project_path.to_string_lossy().to_string(),
        ],
        Some(project_path),
        None,
    )]
}

fn remove_selected_package(state: &mut AppState, force: bool) -> Vec<Effect> {
    let Some(project) = state.selected_project() else {
        state.status_line = "No project selected".to_string();
        return vec![];
    };
    let Some(pkg) = state.selected_manifest_package() else {
        state.status_line = "No installed package selected".to_string();
        return vec![];
    };

    let pkg_name = pkg.name.clone();
    let project_path = project.path.clone();
    let mut args = vec![
        "remove".to_string(),
        "package".to_string(),
        pkg_name.clone(),
        "-p".to_string(),
        project_path.to_string_lossy().to_string(),
    ];
    if force {
        args.push("--force".to_string());
    }

    vec![enqueue_project_task(
        state,
        if force {
            format!("vpm remove package {pkg_name} --force")
        } else {
            format!("vpm remove package {pkg_name}")
        },
        args,
        Some(project_path),
        None,
    )]
}

fn enqueue_command_task(
    state: &mut AppState,
    label: String,
    program: &str,
    args: Vec<String>,
    refresh_manifest_path: Option<PathBuf>,
    pending_add_project: Option<ProjectMeta>,
) -> Effect {
    let task_id = state.next_task_id;
    state.next_task_id = state.next_task_id.saturating_add(1);
    state.tasks.push(TaskRecord {
        id: task_id,
        label: label.clone(),
        state: TaskState::Running,
        exit_code: None,
        error: None,
        refresh_manifest_path,
        pending_add_project,
    });
    state.status_line = format!("Running {label}");
    state.push_log(Some(task_id), format!("[task:{task_id}] start {label}"));

    Effect::RunCommand {
        task_id,
        label,
        program: program.to_string(),
        args,
    }
}

fn enqueue_project_task(
    state: &mut AppState,
    label: String,
    args: Vec<String>,
    refresh_manifest_path: Option<PathBuf>,
    pending_add_project: Option<ProjectMeta>,
) -> Effect {
    enqueue_command_task(state, label, "vpm", args, refresh_manifest_path, pending_add_project)
}

fn enqueue_system_task<const N: usize>(
    state: &mut AppState,
    label: &str,
    args: [&str; N],
) -> Effect {
    enqueue_project_task(
        state,
        label.to_string(),
        args.into_iter().map(|v| v.to_string()).collect(),
        None,
        None,
    )
}

fn enqueue_dotnet_task<const N: usize>(
    state: &mut AppState,
    label: &str,
    args: [&str; N],
) -> Effect {
    enqueue_command_task(
        state,
        label.to_string(),
        "dotnet",
        args.into_iter().map(|v| v.to_string()).collect(),
        None,
        None,
    )
}

fn task_is_label(state: &AppState, task_id: u64, label: &str) -> bool {
    state
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .map(|t| t.label == label)
        .unwrap_or(false)
}

fn task_state_text(task: &TaskRecord) -> String {
    match task.state {
        TaskState::Running => "running".to_string(),
        TaskState::Success => "ok".to_string(),
        TaskState::Cancelled => "cancelled".to_string(),
        TaskState::Failed => {
            if let Some(err) = &task.error {
                format!("failed: {err}")
            } else {
                format!("failed (exit={})", task.exit_code.unwrap_or(-1))
            }
        }
    }
}
