use crate::config::Repository;
use crate::git;
use anyhow::Result;
use colored::*;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug)]
pub struct PrOptions {
    pub title: String,
    pub body: String,
    pub branch_name: Option<String>,
    pub base_branch: Option<String>,
    pub commit_msg: Option<String>,
    pub draft: bool,
    pub token: String,
    pub create_only: bool,
}

pub async fn create_pull_request(repo: &Repository, options: &PrOptions) -> Result<()> {
    let repo_path = repo.get_target_dir();

    // Check if repository has changes
    if !git::has_changes(&repo_path)? {
        println!(
            "{} | {}",
            repo.name.cyan().bold(),
            "No changes detected".yellow()
        );
        return Ok(());
    }

    // Generate branch name if not provided
    let branch_name = options.branch_name.clone().unwrap_or_else(|| {
        format!(
            "automated-changes-{}",
            &Uuid::new_v4().simple().to_string()[..6]
        )
    });

    // Create and checkout new branch
    git::create_and_checkout_branch(&repo_path, &branch_name)?;

    // Add all changes
    git::add_all_changes(&repo_path)?;

    // Commit changes
    let commit_message = options
        .commit_msg
        .clone()
        .unwrap_or_else(|| options.title.clone());
    git::commit_changes(&repo_path, &commit_message)?;

    if !options.create_only {
        // Push branch
        git::push_branch(&repo_path, &branch_name)?;

        // Create PR via GitHub API
        create_github_pr(repo, &branch_name, options).await?;
    }

    Ok(())
}

async fn create_github_pr(repo: &Repository, branch_name: &str, options: &PrOptions) -> Result<()> {
    // Extract owner and repo name from URL
    let (owner, repo_name) = parse_github_url(&repo.url)?;

    // Determine base branch
    let base_branch = options.base_branch.clone().unwrap_or_else(|| {
        // Try to detect default branch - for simplicity, use "main"
        "main".to_string()
    });

    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{owner}/{repo_name}/pulls");

    let pr_data = json!({
        "title": options.title,
        "body": options.body,
        "head": branch_name,
        "base": base_branch,
        "draft": options.draft
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("token {}", options.token))
        .header("User-Agent", "rrepos")
        .json(&pr_data)
        .send()
        .await?;

    if response.status().is_success() {
        let pr_response: serde_json::Value = response.json().await?;
        let pr_url = pr_response["html_url"].as_str().unwrap_or("unknown");
        println!(
            "{} | {} - {}",
            repo.name.cyan().bold(),
            "Pull request created successfully".green(),
            pr_url
        );
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to create PR: {}", error_text);
    }

    Ok(())
}

fn parse_github_url(url: &str) -> Result<(String, String)> {
    // Parse GitHub URLs like git@github.com:owner/repo.git, https://github.com/owner/repo.git,
    // or GitHub Enterprise URLs like git@github-enterprise:owner/repo.git
    let url = url.trim_end_matches(".git");

    if url.starts_with("git@github.com:") {
        let path = url.trim_start_matches("git@github.com:");
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    } else if url.starts_with("https://github.com/") {
        let path = url.trim_start_matches("https://github.com/");
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    } else if url.starts_with("git@") && url.contains(":") {
        // Handle GitHub Enterprise or custom Git hostnames like git@github-enterprise:owner/repo
        if let Some(colon_pos) = url.find(':') {
            let path = &url[colon_pos + 1..];
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() == 2 {
                return Ok((parts[0].to_string(), parts[1].to_string()));
            }
        }
    } else if url.starts_with("https://") {
        // Handle HTTPS URLs for GitHub Enterprise like https://github-enterprise.com/owner/repo
        if let Some(domain_start) = url.find("://") {
            let after_protocol = &url[domain_start + 3..];
            if let Some(slash_pos) = after_protocol.find('/') {
                let path = &after_protocol[slash_pos + 1..];
                let parts: Vec<&str> = path.split('/').collect();
                if parts.len() >= 2 {
                    return Ok((parts[0].to_string(), parts[1].to_string()));
                }
            }
        }
    }

    anyhow::bail!("Invalid GitHub URL format: {}", url);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_standard() {
        // Test standard GitHub SSH URL
        let (owner, repo) = parse_github_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");

        // Test standard GitHub HTTPS URL
        let (owner, repo) = parse_github_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_url_enterprise() {
        // Test GitHub Enterprise SSH URL
        let (owner, repo) =
            parse_github_url("git@github-enterprise:nicos_backbase/journey.git").unwrap();
        assert_eq!(owner, "nicos_backbase");
        assert_eq!(repo, "journey");

        // Test GitHub Enterprise HTTPS URL
        let (owner, repo) =
            parse_github_url("https://github-enterprise.com/nicos_backbase/journey.git").unwrap();
        assert_eq!(owner, "nicos_backbase");
        assert_eq!(repo, "journey");
    }

    #[test]
    fn test_parse_github_url_without_git_suffix() {
        // Test URLs without .git suffix
        let (owner, repo) = parse_github_url("git@custom-host:owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");

        let (owner, repo) = parse_github_url("https://custom-host.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_url_invalid() {
        // Test invalid URLs
        assert!(parse_github_url("invalid-url").is_err());
        assert!(parse_github_url("git@host").is_err());
        assert!(parse_github_url("https://host.com").is_err());
    }
}
