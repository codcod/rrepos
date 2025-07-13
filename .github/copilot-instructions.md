---
applyTo: "**/*.rs"
---

# Copilot Instructions for Rust Project

## Project Overview

RRepos is a CLI tool to manage multiple GitHub repositories - clone them, run
commands across all repositories, create pull requests, and more—all with
colored output and comprehensive logging.

## Code Style & Standards

### **CORE PRINCIPLE: Absolute Minimal Changes**

- **ALWAYS prefer the smallest possible change that fixes the issue**
- **Never refactor existing working code unless specifically requested**
- **Only modify the exact lines/methods that need fixing**
- **Preserve existing code style, naming, and patterns**
- **Add code only when absolutely necessary to fix the specific issue**
- **When fixing bugs, change only what's broken, not what could be improved**

## Documentation Standards

### Minimal Documentation Changes

- **Add documentation only when fixing documentation-related issues**
- **Update only incorrect/outdated information**
- **Don't add documentation to working undocumented code unless requested**

This project emphasizes making the smallest possible changes to achieve the
desired fix, preserving existing working code and patterns wherever possible.
