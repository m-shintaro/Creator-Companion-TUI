use crate::app::state::{AppConfig, AvailablePackage, ManifestSummary};
use crossterm::event::KeyEvent;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub enum Action {
    Init,
    Tick,
    Key(KeyEvent),
    ConfigLoaded(Result<AppConfig, String>),
    ConfigSaved(Result<(), String>),
    FolderScanned(Result<Vec<PathBuf>, String>),
    AvailablePackagesLoaded(Result<Vec<AvailablePackage>, String>),
    ManifestLoaded(Result<ManifestSummary, String>),
    TaskOutput {
        task_id: u64,
        stream: OutputStream,
        line: String,
    },
    TaskDone {
        task_id: u64,
        success: bool,
        cancelled: bool,
        exit_code: Option<i32>,
        error: Option<String>,
    },
}
