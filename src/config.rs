//! Configuration management for repositories and application settings

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip)]
    pub config_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub repositories: Vec<Repository>,
}

impl Repository {
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Check if the repository URL has a valid format
    #[allow(dead_code)]
    pub fn is_url_valid(&self) -> bool {
        self.url.starts_with("git@") || self.url.starts_with("https://")
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Repository name is required");
        }
        if self.url.is_empty() {
            anyhow::bail!("Repository URL is required");
        }
        Ok(())
    }

    pub fn get_target_dir(&self) -> String {
        match &self.path {
            Some(path) => {
                let path_buf = PathBuf::from(path);
                if path_buf.is_absolute() {
                    // If it's already an absolute path, use it as-is
                    path.clone()
                } else {
                    // If it's a relative path, resolve it relative to the config file directory
                    if let Some(config_dir) = &self.config_dir {
                        config_dir.join(path).to_string_lossy().to_string()
                    } else {
                        // Fallback to current directory if config_dir is not available
                        std::env::current_dir()
                            .unwrap_or_else(|_| PathBuf::from("."))
                            .join(path)
                            .to_string_lossy()
                            .to_string()
                    }
                }
            }
            None => {
                // Default path relative to config directory or current directory
                let default_path = format!("cloned_repos/{}", self.name);
                if let Some(config_dir) = &self.config_dir {
                    config_dir.join(&default_path).to_string_lossy().to_string()
                } else {
                    std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(&default_path)
                        .to_string_lossy()
                        .to_string()
                }
            }
        }
    }
}

impl Config {
    pub fn load_config(path: &str) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

        let mut config: Config = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

        // Set the config directory for each repository
        let config_path = Path::new(path);
        let config_dir = config_path.parent().map(|p| p.to_path_buf());

        for repo in &mut config.repositories {
            repo.config_dir = config_dir.clone();
        }

        Ok(config)
    }

    pub fn filter_repositories_by_tag(&self, tag: Option<&str>) -> Vec<&Repository> {
        match tag {
            Some(tag) => self
                .repositories
                .iter()
                .filter(|repo| repo.has_tag(tag))
                .collect(),
            None => self.repositories.iter().collect(),
        }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_relative_path_resolution() {
        let mut repo = Repository {
            name: "test-repo".to_string(),
            url: "git@github.com:owner/repo.git".to_string(),
            tags: vec![],
            path: Some("journey".to_string()),
            branch: None,
            config_dir: Some(PathBuf::from("/some/config/dir")),
        };

        let target_dir = repo.get_target_dir();
        assert_eq!(target_dir, "/some/config/dir/journey");

        // Test absolute path (should use as-is)
        repo.path = Some("/absolute/path/to/repo".to_string());
        let target_dir = repo.get_target_dir();
        assert_eq!(target_dir, "/absolute/path/to/repo");

        // Test default path when no path is specified
        repo.path = None;
        let target_dir = repo.get_target_dir();
        assert_eq!(target_dir, "/some/config/dir/cloned_repos/test-repo");
    }

    #[test]
    fn test_no_config_dir_fallback() {
        let current_dir = env::current_dir().unwrap();

        let repo = Repository {
            name: "test-repo".to_string(),
            url: "git@github.com:owner/repo.git".to_string(),
            tags: vec![],
            path: Some("journey".to_string()),
            branch: None,
            config_dir: None,
        };

        let target_dir = repo.get_target_dir();
        let expected = current_dir.join("journey").to_string_lossy().to_string();
        assert_eq!(target_dir, expected);
    }

    #[test]
    fn test_url_validation() {
        let repo_ssh = Repository {
            name: "test".to_string(),
            url: "git@github.com:owner/repo.git".to_string(),
            tags: vec![],
            path: None,
            branch: None,
            config_dir: None,
        };
        assert!(repo_ssh.is_url_valid());

        let repo_https = Repository {
            name: "test".to_string(),
            url: "https://github.com/owner/repo.git".to_string(),
            tags: vec![],
            path: None,
            branch: None,
            config_dir: None,
        };
        assert!(repo_https.is_url_valid());

        let repo_invalid = Repository {
            name: "test".to_string(),
            url: "invalid-url".to_string(),
            tags: vec![],
            path: None,
            branch: None,
            config_dir: None,
        };
        assert!(!repo_invalid.is_url_valid());
    }
}
