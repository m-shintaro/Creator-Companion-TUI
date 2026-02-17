use crate::app::state::{AppConfig, AvailablePackage, ManifestSummary, PackageInfo};
use anyhow::{Context, Result};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const APP_NAME: &str = "vcc-tui";

pub fn config_file_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join(".config")
        .join(APP_NAME)
        .join("config.json"))
}

pub fn cache_dir_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".cache").join(APP_NAME))
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;
    let config: AppConfig = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse config file: {}", path.display()))?;
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory: {}", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(&path, raw).with_context(|| format!("failed to write config: {}", path.display()))?;
    Ok(())
}

pub fn read_manifest(project_path: &Path) -> Result<ManifestSummary> {
    let manifest_path = project_path.join("Packages").join("vpm-manifest.json");
    if !manifest_path.exists() {
        return Ok(ManifestSummary {
            exists: false,
            packages: Vec::new(),
            message: Some(format!(
                "{} is missing",
                manifest_path
                    .strip_prefix(project_path)
                    .unwrap_or(&manifest_path)
                    .display()
            )),
        });
    }

    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let mut packages = Vec::new();
    if let Some(dependencies) = value.get("dependencies").and_then(|v| v.as_object()) {
        for (name, version_value) in dependencies {
            packages.push(PackageInfo {
                name: name.clone(),
                version: version_value
                    .as_str()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| version_value.to_string()),
            });
        }
        packages.sort_by(|a, b| a.name.cmp(&b.name));
    }

    Ok(ManifestSummary {
        exists: true,
        packages,
        message: None,
    })
}

pub fn scan_projects_one_level(root: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry.with_context(|| format!("failed to read entry in {}", root.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest = path.join("Packages").join("vpm-manifest.json");
        if manifest.exists() {
            found.push(path);
        }
    }
    found.sort();
    Ok(found)
}

pub fn load_available_packages_from_vcc_cache() -> Result<Vec<AvailablePackage>> {
    let repos_dir = home_dir()?
        .join(".local")
        .join("share")
        .join("VRChatCreatorCompanion")
        .join("Repos");
    if !repos_dir.exists() {
        return Ok(Vec::new());
    }

    let mut newest_by_repo_id: std::collections::HashMap<String, (std::time::SystemTime, PathBuf)> =
        std::collections::HashMap::new();
    for entry in fs::read_dir(&repos_dir)
        .with_context(|| format!("failed to read {}", repos_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(v) => v,
            None => continue,
        };
        if file_name == "package-cache.json" {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(v) => v,
            None => continue,
        };
        let repo_id = match stem.rsplit_once('-') {
            Some((id, _)) => id.to_string(),
            None => continue,
        };
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        match newest_by_repo_id.get(&repo_id) {
            Some((existing, _)) if *existing >= modified => {}
            _ => {
                newest_by_repo_id.insert(repo_id, (modified, path));
            }
        }
    }

    let mut packages_by_id: std::collections::HashMap<String, AvailablePackage> =
        std::collections::HashMap::new();
    for (_repo_id_key, (_mtime, path)) in newest_by_repo_id {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let value: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        let repo_obj = match value.get("repo").and_then(|v| v.as_object()) {
            Some(v) => v,
            None => continue,
        };
        let repo_id = repo_obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown.repo");

        let packages = match repo_obj.get("packages").and_then(|v| v.as_object()) {
            Some(v) => v,
            None => continue,
        };
        for (pkg_id, pkg_value) in packages {
            let pkg_obj = match pkg_value.as_object() {
                Some(v) => v,
                None => continue,
            };
            let versions = match pkg_obj.get("versions").and_then(|v| v.as_object()) {
                Some(v) => v,
                None => continue,
            };
            if versions.is_empty() {
                continue;
            }
            let latest_version = versions
                .keys()
                .max()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let display_name = versions
                .values()
                .find_map(|v| v.get("displayName").and_then(|n| n.as_str()))
                .unwrap_or(pkg_id)
                .to_string();

            packages_by_id
                .entry(pkg_id.clone())
                .or_insert(AvailablePackage {
                    id: pkg_id.clone(),
                    display_name,
                    latest_version,
                    repo_id: repo_id.to_string(),
                });
        }
    }

    let mut packages = packages_by_id.into_values().collect::<Vec<_>>();
    packages.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(packages)
}

fn home_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_serialization() {
        let input = AppConfig {
            projects: vec![crate::app::state::ProjectMeta {
                path: PathBuf::from("/tmp/sample"),
                display_name: "sample".to_string(),
                tags: vec!["vrchat".to_string(), "test".to_string()],
                last_opened: Some("2026-02-16T00:00:00Z".to_string()),
            }],
        };

        let raw = serde_json::to_string(&input).expect("serialize config");
        let output: AppConfig = serde_json::from_str(&raw).expect("deserialize config");

        assert_eq!(output.projects.len(), 1);
        assert_eq!(output.projects[0].display_name, "sample");
        assert_eq!(output.projects[0].tags.len(), 2);
    }
}
