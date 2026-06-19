//! CLI argument definitions using clap derive.

use clap::{Parser, Subcommand};

/// ruget — Rust GitHub API client (CLI + MCP dual-mode)
#[derive(Parser, Debug)]
#[command(name = "ruget", version, about = "Rust GitHub API client")]
pub struct Cli {
    /// GitHub personal access token
    #[arg(short, long, env = "GITHUB_TOKEN")]
    pub token: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search GitHub repositories
    Search {
        #[command(subcommand)]
        kind: SearchKind,
    },
    /// Create a GitHub issue
    Issue {
        #[command(subcommand)]
        action: IssueAction,
    },
    /// Create a pull request
    Pr {
        #[command(subcommand)]
        action: PrAction,
    },
    /// File operations
    File {
        #[command(subcommand)]
        action: FileAction,
    },
    /// Generic GitHub API call
    Api {
        /// HTTP method
        #[arg(short = 'X', long, default_value = "GET")]
        method: String,
        /// API path (e.g. /search/repositories?q=rust)
        path: String,
        /// JSON body string
        #[arg(short, long)]
        data: Option<String>,
    },
    /// Show authenticated user info
    Whoami,
}

#[derive(Subcommand, Debug)]
pub enum SearchKind {
    /// Search repositories
    Repos {
        /// Search query
        q: String,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Results per page (max 100)
        #[arg(long, default_value = "30")]
        per_page: u32,
        /// Sort: stars, forks, updated
        #[arg(long)]
        sort: Option<String>,
        /// Order: asc, desc
        #[arg(long, default_value = "desc")]
        order: String,
    },
    /// Search code
    Code {
        /// Search query
        q: String,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Results per page (max 100)
        #[arg(long, default_value = "30")]
        per_page: u32,
    },
    /// Search issues and PRs
    Issues {
        /// Search query
        q: String,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Results per page (max 100)
        #[arg(long, default_value = "30")]
        per_page: u32,
        /// Sort: comments, reactions, created, updated
        #[arg(long)]
        sort: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum IssueAction {
    /// Create a new issue
    Create {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// Issue title
        #[arg(short)]
        title: String,
        /// Issue body (Markdown)
        #[arg(short)]
        body: Option<String>,
        /// Labels (comma-separated)
        #[arg(short)]
        labels: Option<String>,
        /// Assignees (comma-separated)
        #[arg(short = 'a')]
        assignees: Option<String>,
    },
    /// List issues
    List {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// State: open, closed, all
        #[arg(long, default_value = "open")]
        state: String,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Labels filter
        #[arg(long)]
        labels: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum PrAction {
    /// Create a pull request
    Create {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// PR title
        #[arg(short)]
        title: String,
        /// Source branch
        #[arg(long)]
        head: String,
        /// Target branch
        #[arg(long)]
        base: String,
        /// PR body
        #[arg(short)]
        body: Option<String>,
        /// Create as draft
        #[arg(long)]
        draft: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum FileAction {
    /// Get file contents
    Get {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// File path
        path: String,
        /// Branch name
        #[arg(short, long)]
        branch: Option<String>,
    },
}
