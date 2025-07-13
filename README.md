# Multi-Repository Management Tool

Repos is a CLI tool to manage multiple GitHub repositories - clone them, run
commands across all repositories, create pull requests, and more—all with
colored output and comprehensive logging.

## Features

- **Multi-repository management**: Clone and manage multiple repositories from a single config file
- **Tag-based filtering**: Run commands on specific repository groups using tags
- **Parallel execution**: Execute commands across repositories simultaneously for faster operations
- **Colorized output**: Real-time colored logs with repository identification
- **Comprehensive logging**: Per-repository log files for detailed command history
- **Pull request automation**: Create and manage pull requests across multiple repositories
- **Enterprise support**: Compatible with GitHub Enterprise and custom SSH configurations
- **Built in Rust**: Memory-safe, fast, and reliable implementation

## Installation

### From Source

```bash
git clone https://github.com/yourusername/rrepos.git
cd rrepos
cargo build --release
cp target/release/rrepos /usr/local/bin/
```

### Using Cargo

```bash
cargo install --path .
```

## Configuration

The `config.yaml` file defines which repositories to manage and how to organize
them. RRepos supports various Git URL formats including GitHub Enterprise
instances.

```yaml
repositories:
  - name: loan-pricing
    url: git@github.com:yourorg/loan-pricing.git
    tags: [java, backend]
    branch: develop # Optional: Branch to clone
    path: cloned_repos/loan-pricing # Optional: Directory to place cloned repo

  - name: web-ui
    url: git@github.com:yourorg/web-ui.git
    tags: [frontend, react]
    # When branch is not specified, the default branch will be cloned
    # When path is not specified, the current directory will be used

  - name: enterprise-repo
    url: git@github-enterprise:company/project.git
    tags: [enterprise, backend]
    # GitHub Enterprise and custom SSH configurations are supported
```

### Supported URL Formats

RRepos supports all standard Git URL formats:

- **HTTPS**: `https://github.com/owner/repo.git`
- **SSH**: `git@github.com:owner/repo.git`
- **GitHub Enterprise**: `git@github-enterprise:owner/repo.git`
- **Custom SSH hosts**: `git@custom-host:owner/repo.git`

**Note**: RRepos uses the system `git` command for all operations, ensuring compatibility with SSH configurations, enterprise setups, and custom Git configurations.

**Tip:** You can clone repositories first and use these to generate your `config.yaml`:

```bash
mkdir cloned_repos && cd "$_"
git clone http://github.com/example/project1.git
git clone http://github.com/example/project2.git
rrepos init
```

## Usage

### Repository Management

To configure, clone and remove repositories:

```bash
# Scan current directory for git repositories
rrepos init

# Create a different output file
rrepos init -o my-repos-config.yaml

# Overwrite existing config file
rrepos init --overwrite

# Clone all repositories
rrepos clone

# Clone only repositories with tag "rust"
rrepos clone -t rust

# Clone in parallel
rrepos clone -p

# Use a custom config file
rrepos clone -c custom-config.yaml

# Remove cloned repositories
rrepos rm

# Remove only repositories with tag "rust"
rrepos rm -t rust

# Remove in parallel
rrepos rm -p
```

### Running Commands

To run arbitrary commands in repositories:

```bash
# Run a command in all repositories
rrepos run "cargo check"

# Run a command only in repositories with tag "rust"
rrepos run -t rust "cargo build"

# Run in parallel
rrepos run -p "cargo test"

# Specify a custom log directory
rrepos run -l custom/logs "make build"
```

#### Example Commands

Example commands to run with `rrepos run ""`:

```bash
# Count the number of lines
find . -type f | wc -l

# Build Rust projects (consider using --parallel flag)
cargo build

# Update dependencies
cargo update

# Format code
cargo fmt

# Run tests
cargo test

# Create a report of the changes made in the previous month
git log --all --author='$(id -un)' --since='1 month ago' --pretty=format:'%h %an %ad %s' --date=short
```

### Creating Pull Requests

To submit changes made in the cloned repositories:

```bash
export GITHUB_TOKEN=your_github_token

# Create PRs for repositories with changes
rrepos pr --title "My changes" --body "Description of changes"

# Create PRs with specific branch name
rrepos pr --branch feature/my-changes --title "My changes"

# Create draft pull requests
rrepos pr --draft

# Create PRs for specific repositories
rrepos pr -t backend
```

## Typical Session

Once you have a configuration file in place, an example session can look like the following:

```bash
# Remove existing repositories
rrepos rm

# Clone rust-based repositories in parallel
rrepos clone -t rust -p

# Run command to update dependencies in all repos
rrepos run "cargo update"

# Validate changes to see if updates were applied properly
find . -name "Cargo.lock" -exec ls -la {} \;

# Create pull requests for all changes
rrepos pr --title "Update dependencies" --body "Update Cargo.lock files"
```

## Command Reference

```text
A tool to manage multiple GitHub repositories

Usage: rrepos [OPTIONS] <COMMAND>

Commands:
  clone  Clone repositories specified in config
  run    Run a command in each repository
  pr     Create pull requests for repositories with changes
  rm     Remove cloned repositories
  init   Create a config.yaml file from discovered Git repositories
  help   Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Configuration file path [default: config.yaml]
  -t, --tag <TAG>        Filter repositories by tag
  -p, --parallel         Execute operations in parallel
  -h, --help             Print help
  -V, --version          Print version
```

## Differences from the Original Go Version

This Rust implementation provides the same core functionality as the original Go version with the following differences:

### Included Features

- ✅ Multi-repository cloning and management
- ✅ Tag-based filtering
- ✅ Parallel execution
- ✅ Command execution across repositories
- ✅ Pull request creation
- ✅ Repository discovery and initialization
- ✅ Colored output and logging

### Not Yet Implemented

- ❌ Health checks and repository analysis (complex feature from the original)
- ❌ Cyclomatic complexity analysis
- ❌ Advanced configuration with YAML inheritance

### Rust-Specific Improvements

- 🦀 Memory safety guaranteed by Rust's ownership system
- ⚡ Potentially faster execution due to Rust's performance characteristics
- 🔒 Strong type safety preventing many runtime errors
- 📦 Modern async/await support with Tokio

## Dependencies

- `clap` - Command line argument parsing
- `serde` & `serde_yaml` - Configuration file parsing
- `tokio` - Async runtime
- `reqwest` - HTTP client for GitHub API
- `colored` - Terminal colors
- `anyhow` - Error handling
- `chrono` - Date/time operations
- `walkdir` - Directory traversal
- `uuid` - Unique ID generation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo fmt`
6. Submit a pull request

## License

MIT License - same as the original Go implementation.

## Acknowledgments

This is a Rust translation of the excellent [codcod/repos](https://github.com/codcod/repos) tool originally written in Go. All credit for the original design and functionality goes to the original authors.
