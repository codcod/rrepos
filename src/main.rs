mod config;
mod git;
mod github;
mod runner;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use config::{Config, Repository};
use std::env;

#[derive(Debug)]
struct PullRequestOptions {
    title: String,
    body: String,
    branch: Option<String>,
    base: Option<String>,
    message: Option<String>,
    draft: bool,
    token: String,
    create_only: bool,
}

#[derive(Parser)]
#[command(name = "rrepos")]
#[command(about = "A tool to manage multiple GitHub repositories")]
#[command(version = "0.1.0")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    /// Filter repositories by tag
    #[arg(short, long)]
    tag: Option<String>,

    /// Execute operations in parallel
    #[arg(short, long)]
    parallel: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone repositories specified in config
    Clone,

    /// Run a command in each repository
    Run {
        /// Command to execute
        command: String,

        /// Directory to store log files
        #[arg(short, long, default_value = "logs")]
        logs: String,
    },

    /// Create pull requests for repositories with changes
    Pr {
        /// Title for the pull request
        #[arg(long, default_value = "Automated changes")]
        title: String,

        /// Body text for the pull request
        #[arg(long, default_value = "This PR was created automatically")]
        body: String,

        /// Branch name to create
        #[arg(long)]
        branch: Option<String>,

        /// Base branch for the PR
        #[arg(long)]
        base: Option<String>,

        /// Commit message
        #[arg(long)]
        message: Option<String>,

        /// Create PR as draft
        #[arg(long)]
        draft: bool,

        /// GitHub token
        #[arg(long)]
        token: Option<String>,

        /// Only create PR, don't commit changes
        #[arg(long)]
        create_only: bool,
    },

    /// Remove cloned repositories
    Rm,

    /// Create a config.yaml file from discovered Git repositories
    Init {
        /// Output file name
        #[arg(short, long, default_value = "config.yaml")]
        output: String,

        /// Overwrite existing file if it exists
        #[arg(long)]
        overwrite: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Clone => {
            clone_repositories(&cli.config, cli.tag.as_deref(), cli.parallel).await?;
        }
        Commands::Run { command, logs } => {
            run_command(
                &cli.config,
                cli.tag.as_deref(),
                &command,
                &logs,
                cli.parallel,
            )
            .await?;
        }
        Commands::Pr {
            title,
            body,
            branch,
            base,
            message,
            draft,
            token,
            create_only,
        } => {
            let token = token.or_else(|| env::var("GITHUB_TOKEN").ok())
                .ok_or_else(|| anyhow::anyhow!("GitHub token not provided. Use --token flag or set GITHUB_TOKEN environment variable."))?;

            let pr_options = PullRequestOptions {
                title,
                body,
                branch,
                base,
                message,
                draft,
                token,
                create_only,
            };

            create_pull_requests(&cli.config, cli.tag.as_deref(), pr_options, cli.parallel).await?;
        }
        Commands::Rm => {
            remove_repositories(&cli.config, cli.tag.as_deref(), cli.parallel).await?;
        }
        Commands::Init { output, overwrite } => {
            init_config(&output, overwrite).await?;
        }
    }

    Ok(())
}

async fn clone_repositories(config_file: &str, tag: Option<&str>, parallel: bool) -> Result<()> {
    let config = Config::load_config(config_file)?;
    let repositories = config.filter_repositories_by_tag(tag);

    if repositories.is_empty() {
        println!(
            "{}",
            format!("No repositories found with tag: {}", tag.unwrap_or("")).yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Cloning {} repositories...", repositories.len()).green()
    );

    if parallel {
        let tasks: Vec<_> = repositories
            .into_iter()
            .map(|repo| tokio::spawn(clone_single_repo(repo.clone())))
            .collect();

        for task in tasks {
            if let Err(e) = task.await? {
                eprintln!("{}", format!("Error: {e}").red());
            }
        }
    } else {
        for repo in repositories {
            if let Err(e) = clone_single_repo(repo.clone()).await {
                eprintln!("{}", format!("Error: {e}").red());
            }
        }
    }

    println!("{}", "Done cloning repositories".green());
    Ok(())
}

async fn clone_single_repo(repo: Repository) -> Result<()> {
    tokio::task::spawn_blocking(move || git::clone_repository(&repo)).await?
}

async fn run_command(
    config_file: &str,
    tag: Option<&str>,
    command: &str,
    log_dir: &str,
    parallel: bool,
) -> Result<()> {
    let config = Config::load_config(config_file)?;
    let repositories = config.filter_repositories_by_tag(tag);

    if repositories.is_empty() {
        println!(
            "{}",
            format!("No repositories found with tag: {}", tag.unwrap_or("")).yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "Running '{}' in {} repositories...",
            command,
            repositories.len()
        )
        .green()
    );

    let runner = runner::CommandRunner::new();

    if parallel {
        let tasks: Vec<_> = repositories
            .into_iter()
            .map(|repo| {
                let runner = &runner;
                let command = command.to_string();
                let log_dir = log_dir.to_string();
                async move { runner.run_command(repo, &command, Some(&log_dir)).await }
            })
            .collect();

        for task in tasks {
            if let Err(e) = task.await {
                eprintln!("{}", format!("Error: {e}").red());
            }
        }
    } else {
        for repo in repositories {
            if let Err(e) = runner.run_command(repo, command, Some(log_dir)).await {
                eprintln!("{}", format!("Error: {e}").red());
            }
        }
    }

    println!("{}", "Done running commands in all repositories".green());
    Ok(())
}

async fn create_pull_requests(
    config_file: &str,
    tag: Option<&str>,
    options: PullRequestOptions,
    parallel: bool,
) -> Result<()> {
    let config = Config::load_config(config_file)?;
    let repositories = config.filter_repositories_by_tag(tag);

    if repositories.is_empty() {
        println!(
            "{}",
            format!("No repositories found with tag: {}", tag.unwrap_or("")).yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "Checking {} repositories for changes...",
            repositories.len()
        )
        .green()
    );

    let pr_options = github::PrOptions {
        title: options.title,
        body: options.body,
        branch_name: options.branch,
        base_branch: options.base,
        commit_msg: options.message,
        draft: options.draft,
        token: options.token,
        create_only: options.create_only,
    };

    let mut success_count = 0;

    if parallel {
        let tasks: Vec<_> = repositories
            .into_iter()
            .map(|repo| {
                let pr_options = github::PrOptions {
                    title: pr_options.title.clone(),
                    body: pr_options.body.clone(),
                    branch_name: pr_options.branch_name.clone(),
                    base_branch: pr_options.base_branch.clone(),
                    commit_msg: pr_options.commit_msg.clone(),
                    draft: pr_options.draft,
                    token: pr_options.token.clone(),
                    create_only: pr_options.create_only,
                };
                async move { github::create_pull_request(repo, &pr_options).await }
            })
            .collect();

        for task in tasks {
            match task.await {
                Ok(_) => success_count += 1,
                Err(e) => eprintln!("{}", format!("Error: {e}").red()),
            }
        }
    } else {
        for repo in repositories {
            match github::create_pull_request(repo, &pr_options).await {
                Ok(_) => success_count += 1,
                Err(e) => eprintln!("{}", format!("Error: {e}").red()),
            }
        }
    }

    println!(
        "{}",
        format!("Created {success_count} pull requests").green()
    );
    Ok(())
}

async fn remove_repositories(config_file: &str, tag: Option<&str>, parallel: bool) -> Result<()> {
    let config = Config::load_config(config_file)?;
    let repositories = config.filter_repositories_by_tag(tag);

    if repositories.is_empty() {
        println!(
            "{}",
            format!("No repositories found with tag: {}", tag.unwrap_or("")).yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Removing {} repositories...", repositories.len()).green()
    );

    if parallel {
        let tasks: Vec<_> = repositories
            .into_iter()
            .map(|repo| tokio::spawn(remove_single_repo(repo.clone())))
            .collect();

        for task in tasks {
            if let Err(e) = task.await? {
                eprintln!("{}", format!("Error: {e}").red());
            }
        }
    } else {
        for repo in repositories {
            if let Err(e) = remove_single_repo(repo.clone()).await {
                eprintln!("{}", format!("Error: {e}").red());
            } else {
                println!(
                    "{} | {}",
                    repo.name.cyan().bold(),
                    "Successfully removed".green()
                );
            }
        }
    }

    println!("{}", "Done removing repositories".green());
    Ok(())
}

async fn remove_single_repo(repo: Repository) -> Result<()> {
    tokio::task::spawn_blocking(move || git::remove_repository(&repo)).await?
}

async fn init_config(output_file: &str, overwrite: bool) -> Result<()> {
    use std::path::Path;

    // Check if output file already exists
    if Path::new(output_file).exists() && !overwrite {
        anyhow::bail!(
            "File {} already exists. Use --overwrite to replace it.",
            output_file
        );
    }

    // Get current directory
    let current_dir = env::current_dir()?;

    println!(
        "{}",
        format!(
            "Scanning for Git repositories in {}...",
            current_dir.display()
        )
        .green()
    );

    // Find Git repositories
    let repositories = util::find_git_repositories(&current_dir.to_string_lossy())?;

    if repositories.is_empty() {
        println!(
            "{}",
            format!("No Git repositories found in {}", current_dir.display()).yellow()
        );
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} Git repositories", repositories.len()).green()
    );

    // Create config structure
    let config = Config { repositories };

    // Save to file
    config.save(output_file)?;

    println!(
        "{}",
        format!(
            "Successfully created {} with {} repositories",
            output_file,
            config.repositories.len()
        )
        .green()
    );

    // Print preview of the generated file
    println!("\nConfig file preview:");
    println!("{}", "---".cyan());
    let yaml_content = std::fs::read_to_string(output_file)?;
    println!("{yaml_content}");

    Ok(())
}
