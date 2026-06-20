//! Data models mirroring GitHub REST API responses.
//! Only the fields ruget needs — extensible as needed.

use serde::{Deserialize, Serialize};

// ── Search ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub total_count: u64,
    pub incomplete_results: bool,
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalRepo {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub html_url: String,
    pub description: Option<String>,
    pub fork: bool,
    pub language: Option<String>,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub open_issues_count: u64,
    pub topics: Option<Vec<String>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub pushed_at: Option<String>,
    pub default_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMatch {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub url: String,
    pub html_url: String,
    pub repository: Option<MinimalRepo>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueItem {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub body: Option<String>,
    pub user: Option<SimpleUser>,
    pub labels: Vec<Label>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub closed_at: Option<String>,
}

// ── Users ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: Option<u64>,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

// ── Issues ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
}

// ── Pull Requests ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePRRequest {
    pub title: String,
    pub head: String,
    pub base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
}

// ── File Content ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub url: String,
    pub html_url: Option<String>,
    pub content: Option<String>,
    pub encoding: Option<String>,
    #[serde(rename = "type")]
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileRequest {
    pub message: String,
    pub content: String, // Base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>, // Required for updates
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOpResponse {
    pub content: Option<FileContent>,
    pub commit: Option<CommitRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRef {
    pub sha: String,
    pub html_url: Option<String>,
}

// ── Generic ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub documentation_url: Option<String>,
}

/// Unified result for MCP tool responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
