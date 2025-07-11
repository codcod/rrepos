//! RRepos library - shared types and utilities for managing multiple repositories

pub mod config;
pub mod git;
pub mod github;
pub mod runner;
pub mod util;

pub type Result<T> = anyhow::Result<T>;

// Re-export commonly used types
pub use config::{Config, Repository};
pub use github::PrOptions;
