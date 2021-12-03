use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub type_prefix: Option<String>,
    pub type_suffix: Option<String>,
    pub subject_separator: String,
    pub scope_prefix: String,
    pub scope_suffix: String,
}

impl Display for Type {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "{} - {}", self.name, self.description)
    }
}

#[derive(Clone, Deserialize)]
pub struct Type {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct Messages {
    pub commit_type: String,
    pub commit_scope: String,
    pub commit_description: String,
    pub commit_body: String,
    pub commit_footer: String,
}

#[derive(Deserialize)]
pub struct CommitPattern {
    pub config: Config,
    pub commit_types: Vec<Type>,
    pub commit_scopes: Vec<Type>,
    pub msg: Messages,
}

fn get_config_path_content(config_path: impl AsRef<Path>) -> Result<String> {
    let mut file = File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn select_custom_config_path(config: Option<PathBuf>) -> Result<PathBuf> {
    match config {
        Some(config_path) => {
            if config_path.exists() {
                Ok(config_path)
            } else {
                Err(anyhow!(
                    "Config file does not exist: {}",
                    config_path.display()
                ))
            }
        }
        None => get_config_path(),
    }
}

fn get_config_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let current_file = current_dir.join("commit.json");
    if current_file.exists() {
        Ok(current_file)
    } else {
        let config_file = dirs::config_dir()
            .ok_or(anyhow!("Could not find config directory"))?
            .join("commit/commit.json");
        Ok(config_file)
    }
}

pub fn get_pattern(config_path: Option<PathBuf>) -> Result<CommitPattern> {
    let default_pattern_str = include_str!("../commit-default.json");
    let selected_config_path = select_custom_config_path(config_path)?;
    let pattern_str = get_config_path_content(&selected_config_path)
        .unwrap_or_else(|_| default_pattern_str.to_string());
    serde_json::from_str(&pattern_str).context("Failed to parse commit pattern from file")
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn select_custom_config_path_test() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let config_file = temp_dir.child("config.json");
        config_file.touch()?;

        let config_path = Some(config_file.path().to_owned());
        let selected_config_path = select_custom_config_path(config_path.clone())?;

        assert_eq!(config_path.unwrap().to_str(), selected_config_path.to_str());

        let config_path_default = dirs::config_dir().unwrap().join("commit/commit.json");
        let selected_config_path = select_custom_config_path(None)?;

        assert_eq!(selected_config_path.to_str(), config_path_default.to_str());

        let selected_config_path = select_custom_config_path(Some(PathBuf::new()));
        match selected_config_path {
            Err(err) => assert_eq!(err.to_string(), "Config file does not exist: "),
            _ => assert!(false),
        }
        Ok(())
    }
    #[test]
    fn get_config_path_test() -> Result<()> {
        let config_file = dirs::config_dir().unwrap().join("commit/commit.json");
        let config_path = get_config_path();
        assert_eq!(config_file.to_str(), config_path.unwrap().to_str());
        Ok(())
    }
    #[test]
    fn get_config_path_content_test() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let config_file = temp_dir.child("config.json");
        config_file.touch()?;
        let config_path = config_file.path();
        let content = get_config_path_content(config_path)?;
        assert_eq!(content, "");
        let expected = include_str!("../commit-default.json");
        config_file.write_str(expected)?;
        let content = get_config_path_content(config_path)?;
        assert_eq!(content, expected);
        Ok(())
    }
    #[test]
    fn get_pattern_test() -> Result<()> {
        let pattern = get_pattern(None)?;
        assert_eq!(pattern.config.type_prefix, None);
        assert_eq!(pattern.config.type_suffix, None);
        assert_eq!(pattern.config.subject_separator, ":");
        assert_eq!(pattern.config.scope_prefix, "(");
        assert_eq!(pattern.config.scope_suffix, ")");
        Ok(())
    }
}
