# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- GitHub Actions CI/CD workflows
- Docker support for containerized deployments
- Support for relative paths in config.yaml
- Enhanced GitHub Enterprise and custom Git host support
- Comprehensive test coverage

### Changed

- Migrated from git2 library to system git commands for better compatibility
- Improved URL parsing to support GitHub Enterprise and custom hosts
- Enhanced error handling and logging

### Fixed

- SSH configuration compatibility issues
- GitHub Enterprise authentication problems

## [0.1.0] - 2025-07-11

### New

- Initial release of rrepos
- Repository cloning functionality
- Command execution across multiple repositories
- Pull request creation support
- Repository management (clone, run, pr, rm, init)
- Configuration-based repository management
- Tag-based repository filtering
- Parallel execution support
