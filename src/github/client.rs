//! GitHub API client implementation

use super::auth::GitHubAuth;
use super::types::{PullRequestParams, constants::*};
use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};

/// GitHub API client
pub struct GitHubClient {
    client: Client,
    auth: Option<GitHubAuth>,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new(token: Option<String>) -> Self {
        let auth = token.map(GitHubAuth::new);
        Self {
            client: Client::new(),
            auth,
        }
    }

    /// Parse GitHub URL to extract owner and repository name
    pub fn parse_github_url(&self, url: &str) -> Result<(String, String)> {
        let url = url.trim_end_matches('/').trim_end_matches(".git");

        if let Some(captures) = regex::Regex::new(r"github\.com[:/]([^/]+)/([^/]+)")?.captures(url)
        {
            let owner = captures.get(1).unwrap().as_str().to_string();
            let repo = captures.get(2).unwrap().as_str().to_string();
            Ok((owner, repo))
        } else {
            Err(anyhow::anyhow!("Invalid GitHub URL: {}", url))
        }
    }

    /// Create a pull request
    pub async fn create_pull_request(&self, params: PullRequestParams<'_>) -> Result<Value> {
        let auth = self
            .auth
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("GitHub token is required"))?;

        let url = format!(
            "{}/repos/{}/{}/pulls",
            GITHUB_API_BASE, params.owner, params.repo
        );

        let payload = json!({
            "title": params.title,
            "body": params.body,
            "head": params.head,
            "base": params.base,
            "draft": params.draft
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("token {}", auth.token()))
            .header("User-Agent", DEFAULT_USER_AGENT)
            .header("Accept", "application/vnd.github.v3+json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: Value = response.json().await?;
            Ok(result)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("GitHub API error: {}", error_text))
        }
    }
}
