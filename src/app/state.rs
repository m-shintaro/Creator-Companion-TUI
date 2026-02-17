use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    New,
    Add,
    Projects,
    Manage,
    Settings,
}

impl Screen {
    pub fn next(self) -> Self {
        match self {
            Self::New => Self::Add,
            Self::Add => Self::Projects,
            Self::Projects => Self::Manage,
            Self::Manage => Self::Settings,
            Self::Settings => Self::New,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::New => Self::Settings,
            Self::Add => Self::New,
            Self::Projects => Self::Add,
            Self::Manage => Self::Projects,
            Self::Settings => Self::Manage,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub path: PathBuf,
    pub display_name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub last_opened: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub projects: Vec<ProjectMeta>,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct AvailablePackage {
    pub id: String,
    pub display_name: String,
    pub latest_version: String,
    pub repo_id: String,
}

#[derive(Debug, Clone)]
pub struct ManifestSummary {
    pub exists: bool,
    pub packages: Vec<PackageInfo>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct TaskRecord {
    pub id: u64,
    pub label: String,
    pub state: TaskState,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub refresh_manifest_path: Option<PathBuf>,
    pub pending_add_project: Option<ProjectMeta>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub task_id: Option<u64>,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct SystemChecks {
    pub dotnet_version: Option<String>,
    pub vpm_version: Option<String>,
    pub hub_check: Option<String>,
    pub unity_check: Option<String>,
    pub unity_list: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub screen: Screen,
    pub should_quit: bool,
    pub tick_count: u64,
    pub projects: Vec<ProjectMeta>,
    pub selected_project: usize,
    pub search_mode: bool,
    pub search_query: String,
    pub add_project_mode: bool,
    pub add_project_input: String,
    pub add_folder_mode: bool,
    pub add_folder_input: String,
    pub new_project_mode: bool,
    pub new_project_edit_path: bool,
    pub new_project_name_input: String,
    pub new_project_path_input: String,
    pub new_project_template_idx: usize,
    pub add_repo_mode: bool,
    pub add_repo_input: String,
    pub add_package_mode: bool,
    pub add_package_input: String,
    pub selected_project_manifest: Option<ManifestSummary>,
    pub selected_manifest_package: usize,
    pub available_packages: Vec<AvailablePackage>,
    pub available_package_search: String,
    pub available_package_search_mode: bool,
    pub selected_available_package: usize,
    pub manage_focus_available: bool,
    pub logs: Vec<LogEntry>,
    pub log_scroll: u16,
    pub tasks: Vec<TaskRecord>,
    pub next_task_id: u64,
    pub status_line: String,
    pub system_checks: SystemChecks,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Projects,
            should_quit: false,
            tick_count: 0,
            projects: Vec::new(),
            selected_project: 0,
            search_mode: false,
            search_query: String::new(),
            add_project_mode: false,
            add_project_input: String::new(),
            add_folder_mode: false,
            add_folder_input: String::new(),
            new_project_mode: false,
            new_project_edit_path: false,
            new_project_name_input: String::new(),
            new_project_path_input: String::new(),
            new_project_template_idx: 0,
            add_repo_mode: false,
            add_repo_input: String::new(),
            add_package_mode: false,
            add_package_input: String::new(),
            selected_project_manifest: None,
            selected_manifest_package: 0,
            available_packages: Vec::new(),
            available_package_search: String::new(),
            available_package_search_mode: false,
            selected_available_package: 0,
            manage_focus_available: true,
            logs: Vec::new(),
            log_scroll: 0,
            tasks: Vec::new(),
            next_task_id: 1,
            status_line: "Ready".to_string(),
            system_checks: SystemChecks::default(),
        }
    }
}

impl AppState {
    pub fn filtered_projects(&self) -> Vec<&ProjectMeta> {
        if self.search_query.is_empty() {
            return self.projects.iter().collect();
        }

        let needle = self.search_query.to_lowercase();
        self.projects
            .iter()
            .filter(|p| {
                p.display_name.to_lowercase().contains(&needle)
                    || p.path.to_string_lossy().to_lowercase().contains(&needle)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&needle))
            })
            .collect()
    }

    pub fn push_log<T: Into<String>>(&mut self, task_id: Option<u64>, line: T) {
        const MAX_LOG_LINES: usize = 1500;
        self.logs.push(LogEntry {
            task_id,
            text: line.into(),
        });
        if self.logs.len() > MAX_LOG_LINES {
            let overflow = self.logs.len() - MAX_LOG_LINES;
            self.logs.drain(0..overflow);
        }
    }

    pub fn selected_project(&self) -> Option<&ProjectMeta> {
        self.projects.get(self.selected_project)
    }

    pub fn selected_project_clamped(&mut self) {
        if self.projects.is_empty() {
            self.selected_project = 0;
        } else if self.selected_project >= self.projects.len() {
            self.selected_project = self.projects.len() - 1;
        }
    }

    pub fn selected_manifest_package_clamped(&mut self) {
        let len = self
            .selected_project_manifest
            .as_ref()
            .map(|m| m.packages.len())
            .unwrap_or(0);
        if len == 0 {
            self.selected_manifest_package = 0;
        } else if self.selected_manifest_package >= len {
            self.selected_manifest_package = len - 1;
        }
    }

    pub fn selected_manifest_package(&self) -> Option<&PackageInfo> {
        self.selected_project_manifest
            .as_ref()
            .and_then(|m| m.packages.get(self.selected_manifest_package))
    }

    pub fn filtered_available_packages(&self) -> Vec<&AvailablePackage> {
        if self.available_package_search.is_empty() {
            return self.available_packages.iter().collect();
        }
        let needle = self.available_package_search.to_lowercase();
        self.available_packages
            .iter()
            .filter(|p| {
                p.id.to_lowercase().contains(&needle)
                    || p.display_name.to_lowercase().contains(&needle)
                    || p.repo_id.to_lowercase().contains(&needle)
            })
            .collect()
    }

    pub fn selected_available_package_clamped(&mut self) {
        let len = self.filtered_available_packages().len();
        if len == 0 {
            self.selected_available_package = 0;
        } else if self.selected_available_package >= len {
            self.selected_available_package = len - 1;
        }
    }

    pub fn selected_available_package(&self) -> Option<&AvailablePackage> {
        self.filtered_available_packages()
            .get(self.selected_available_package)
            .copied()
    }

    pub fn current_template(&self) -> &'static str {
        match self.new_project_template_idx {
            0 => "Avatar",
            1 => "World",
            _ => "UdonSharp",
        }
    }
}
