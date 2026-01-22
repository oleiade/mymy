# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mymy is a multi-platform Rust CLI tool (binary name: `my`) for gathering system information. It provides 14 subcommands for querying network, system, datetime, and storage information with text or JSON output.

## Build & Development Commands

```bash
# Build
cargo build                        # Debug build
cargo build --release              # Release build

# Cross-platform builds (via Makefile)
make linux                         # Linux targets + .deb packages
make macos                         # macOS targets
make release                       # All platforms

# Run
cargo run -- <COMMAND>             # Run with cargo
my <COMMAND>                       # Run installed binary
my --format json <COMMAND>         # JSON output
```

## Code Quality

The project enforces very strict lints in Cargo.toml:
- **Unsafe code is forbidden** at compile time
- Clippy pedantic and nursery lints are denied
- `unwrap_used` is denied - use `?` operator or explicit error handling

## Architecture

**Entry point**: `src/main.rs` - Clap-based CLI with subcommand enum routing

**Module responsibilities**:
- `network.rs` - Public IP lookup (via OpenDNS), DNS servers, network interfaces
- `system.rs` - Hostname, username, device-name, OS, architecture, CPU, RAM
- `datetime.rs` - Date/time with NTP sync for clock offset calculation
- `storage.rs` - Disk information with total/free space
- `output.rs` - `Named` enum wrapper for semantic JSON keys
- `format.rs` - Human-readable byte size formatting (KiB, MiB, GiB, etc.)

**Key patterns**:
- `CommandResult` enum wraps all command outputs for unified Display/Serialize
- All I/O is async via tokio; blocking operations use `spawn_blocking`
- Error handling uses anyhow with `.context()` chains
- Output formatting separates Display (colored text) from Serialize (JSON)

## Version Control

This repository uses **Jujutsu** (`jj`) - use `jj` commands instead of `git`.

## Conventions

- Follow conventional commits (feat:, fix:, chore:, ci:, etc.)
- Never use `git add .` - add files explicitly by name
