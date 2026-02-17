use crate::app::state::AppConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Effect {
    LoadConfig,
    LoadAvailablePackages,
    SaveConfig(AppConfig),
    ScanProjectsFolder {
        root: PathBuf,
    },
    ReadManifest {
        project_path: PathBuf,
    },
    RunCommand {
        task_id: u64,
        label: String,
        program: String,
        args: Vec<String>,
    },
    CancelTask {
        task_id: u64,
    },
}
