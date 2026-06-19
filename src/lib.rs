//! ruget — Rust GitHub API client
//!
//! CLI + MCP dual-mode:
//! - `ruget` binary: CLI interface
//! - `ruget-mcp` binary: MCP Server over stdio
//!
//! Core crate re-exports:
//! - `client::GitHubClient` — the main API client
//! - `auth` — token management
//! - `models` — data types

pub mod auth;
pub mod client;
pub mod models;

// Re-export main types
pub use client::GitHubClient;
pub use models::*;
