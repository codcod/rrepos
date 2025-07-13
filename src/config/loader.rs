//! Configuration file loading and saving

use super::{ConfigValidator, Repository};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repositories: Vec<Repository>,
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        let mut config: Config = serde_yaml::from_str(&content)?;

        // Set the config directory for each repository
        let config_path = Path::new(path);
        let config_dir = config_path.parent().map(|p| p.to_path_buf());

        for repo in &mut config.repositories {
            repo.set_config_dir(config_dir.clone());
        }

        // Validate the loaded configuration
        ConfigValidator::validate_repositories(&config.repositories)?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &str) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;

        std::fs::write(path, yaml)?;

        Ok(())
    }

    /// Filter repositories by tag
    pub fn filter_by_tag(&self, tag: Option<&str>) -> Vec<Repository> {
        match tag {
            Some(tag) => self
                .repositories
                .iter()
                .filter(|repo| repo.has_tag(tag))
                .cloned()
                .collect(),
            None => self.repositories.clone(),
        }
    }

    /// Filter repositories by multiple tags (OR logic)
    pub fn filter_by_any_tag(&self, tags: &[String]) -> Vec<Repository> {
        if tags.is_empty() {
            return self.repositories.clone();
        }

        self.repositories
            .iter()
            .filter(|repo| repo.has_any_tag(tags))
            .cloned()
            .collect()
    }

    /// Filter repositories by multiple tags (AND logic)
    pub fn filter_by_all_tags(&self, tags: &[String]) -> Vec<Repository> {
        if tags.is_empty() {
            return self.repositories.clone();
        }

        self.repositories
            .iter()
            .filter(|repo| tags.iter().all(|tag| repo.has_tag(tag)))
            .cloned()
            .collect()
    }

    /// Get repository by name
    pub fn get_repository(&self, name: &str) -> Option<&Repository> {
        self.repositories.iter().find(|repo| repo.name == name)
    }

    /// Get mutable repository by name
    pub fn get_repository_mut(&mut self, name: &str) -> Option<&mut Repository> {
        self.repositories.iter_mut().find(|repo| repo.name == name)
    }

    /// Add a repository to the configuration
    pub fn add_repository(&mut self, repo: Repository) -> Result<()> {
        // Check for duplicate names
        if self.get_repository(&repo.name).is_some() {
            return Err(anyhow::anyhow!("Repository '{}' already exists", repo.name));
        }

        // Validate the repository
        repo.validate()?;

        self.repositories.push(repo);
        Ok(())
    }

    /// Remove a repository from the configuration
    pub fn remove_repository(&mut self, name: &str) -> bool {
        let initial_len = self.repositories.len();
        self.repositories.retain(|repo| repo.name != name);
        self.repositories.len() != initial_len
    }

    /// Get all unique tags across all repositories
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .repositories
            .iter()
            .flat_map(|repo| repo.tags.iter())
            .cloned()
            .collect();

        tags.sort();
        tags.dedup();
        tags
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<()> {
        ConfigValidator::validate_repositories(&self.repositories)?;
        Ok(())
    }

    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
        }
    }

    /// Alias for load method for backwards compatibility
    pub fn load_config(path: &str) -> Result<Self> {
        Self::load(path)
    }

    /// Filter repositories by tag (alias for backwards compatibility)
    pub fn filter_repositories_by_tag(&self, tag: Option<&str>) -> Vec<Repository> {
        self.filter_by_tag(tag)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        let mut repo1 = Repository::new(
            "repo1".to_string(),
            "git@github.com:owner/repo1.git".to_string(),
        );
        repo1.add_tag("frontend".to_string());
        repo1.add_tag("web".to_string());

        let mut repo2 = Repository::new(
            "repo2".to_string(),
            "git@github.com:owner/repo2.git".to_string(),
        );
        repo2.add_tag("backend".to_string());
        repo2.add_tag("api".to_string());

        Config {
            repositories: vec![repo1, repo2],
        }
    }

    #[test]
    fn test_filter_by_tag() {
        let config = create_test_config();

        let frontend_repos = config.filter_by_tag(Some("frontend"));
        assert_eq!(frontend_repos.len(), 1);
        assert_eq!(frontend_repos[0].name, "repo1");

        let all_repos = config.filter_by_tag(None);
        assert_eq!(all_repos.len(), 2);
    }

    #[test]
    fn test_filter_by_any_tag() {
        let config = create_test_config();

        let web_repos = config.filter_by_any_tag(&["frontend".to_string(), "api".to_string()]);
        assert_eq!(web_repos.len(), 2); // Both repos match

        let no_match = config.filter_by_any_tag(&["mobile".to_string()]);
        assert_eq!(no_match.len(), 0);
    }

    #[test]
    fn test_get_all_tags() {
        let config = create_test_config();
        let tags = config.get_all_tags();

        assert_eq!(tags, vec!["api", "backend", "frontend", "web"]);
    }

    #[test]
    fn test_add_remove_repository() {
        let mut config = Config::new();

        let repo = Repository::new(
            "test".to_string(),
            "git@github.com:owner/test.git".to_string(),
        );

        config.add_repository(repo).unwrap();
        assert_eq!(config.repositories.len(), 1);

        let removed = config.remove_repository("test");
        assert!(removed);
        assert_eq!(config.repositories.len(), 0);

        let not_removed = config.remove_repository("nonexistent");
        assert!(!not_removed);
    }
}
