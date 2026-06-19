//! GitHub REST API client using ureq + native-tls (schannel).

use crate::auth::resolve_token;
use crate::models::*;
use serde::de::DeserializeOwned;
use std::time::Duration;

const API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = "ruget/0.1";
const TIMEOUT_SECS: u64 = 30;

/// Main GitHub API client.
pub struct GitHubClient {
    agent: ureq::Agent,
    token: String,
}

impl GitHubClient {
    /// Create a new client.
    /// Token is read from `GITHUB_TOKEN` or `GITHUB_PERSONAL_ACCESS_TOKEN` env.
    /// Panics if no token is found.
    pub fn new() -> Self {
        let token = resolve_token()
            .expect("GITHUB_TOKEN or GITHUB_PERSONAL_ACCESS_TOKEN env not set");
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .user_agent(USER_AGENT)
            .build();

        Self { agent, token }
    }

    /// Create with explicit token.
    pub fn with_token(token: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .user_agent(USER_AGENT)
            .build();

        Self { agent, token }
    }

    // ── HTTP helpers ──

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ureq::Error> {
        self.agent
            .get(&format!("{API_BASE}{path}"))
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .call()?
            .into_json()
    }

    fn post<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, ureq::Error> {
        let json = serde_json::to_string(body).unwrap_or_default();
        self.agent
            .post(&format!("{API_BASE}{path}"))
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set("Content-Type", "application/json")
            .send_string(&json)?
            .into_json()
    }

    fn put<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, ureq::Error> {
        let json = serde_json::to_string(body).unwrap_or_default();
        self.agent
            .put(&format!("{API_BASE}{path}"))
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28")
            .set("Content-Type", "application/json")
            .send_string(&json)?
            .into_json()
    }

    fn get_raw(&self, path: &str) -> Result<serde_json::Value, ureq::Error> {
        self.get::<serde_json::Value>(path)
    }

    // ── Search APIs ──

    /// Search repositories.
    pub fn search_repositories(
        &self,
        q: &str,
        page: u32,
        per_page: u32,
        sort: Option<&str>,
        order: Option<&str>,
    ) -> Result<SearchResult<MinimalRepo>, ureq::Error> {
        let mut path = format!("/search/repositories?q={q}&page={page}&per_page={}", per_page.min(100));
        if let Some(s) = sort {
            path.push_str(&format!("&sort={s}"));
        }
        if let Some(o) = order {
            path.push_str(&format!("&order={o}"));
        }
        self.get(&path)
    }

    /// Search code.
    pub fn search_code(
        &self,
        q: &str,
        page: u32,
        per_page: u32,
    ) -> Result<SearchResult<CodeMatch>, ureq::Error> {
        let path = format!("/search/code?q={q}&page={page}&per_page={}", per_page.min(100));
        self.get(&path)
    }

    /// Search issues and PRs.
    pub fn search_issues(
        &self,
        q: &str,
        page: u32,
        per_page: u32,
        sort: Option<&str>,
        order: Option<&str>,
    ) -> Result<SearchResult<IssueItem>, ureq::Error> {
        let mut path = format!("/search/issues?q={q}&page={page}&per_page={}", per_page.min(100));
        if let Some(s) = sort {
            path.push_str(&format!("&sort={s}"));
        }
        if let Some(o) = order {
            path.push_str(&format!("&order={o}"));
        }
        self.get(&path)
    }

    // ── Issue APIs ──

    /// Create an issue.
    pub fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        req: &CreateIssueRequest,
    ) -> Result<IssueItem, ureq::Error> {
        let path = format!("/repos/{owner}/{repo}/issues");
        self.post(&path, req)
    }

    /// List issues for a repository.
    pub fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
        page: u32,
        per_page: u32,
        labels: Option<&str>,
    ) -> Result<Vec<IssueItem>, ureq::Error> {
        let mut path = format!(
            "/repos/{owner}/{repo}/issues?state={state}&page={page}&per_page={}",
            per_page.min(100)
        );
        if let Some(l) = labels {
            path.push_str(&format!("&labels={l}"));
        }
        self.get(&path)
    }

    /// Get a single issue.
    pub fn get_issue(&self, owner: &str, repo: &str, number: u64) -> Result<IssueItem, ureq::Error> {
        let path = format!("/repos/{owner}/{repo}/issues/{number}");
        self.get(&path)
    }

    // ── Pull Request APIs ──

    /// Create a pull request.
    pub fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        req: &CreatePRRequest,
    ) -> Result<serde_json::Value, ureq::Error> {
        let path = format!("/repos/{owner}/{repo}/pulls");
        self.post(&path, req)
    }

    /// List pull requests.
    pub fn list_prs(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<serde_json::Value>, ureq::Error> {
        let path = format!(
            "/repos/{owner}/{repo}/pulls?state={state}&page={page}&per_page={}",
            per_page.min(100)
        );
        self.get(&path)
    }

    // ── File APIs ──

    /// Get file contents (decoded).
    pub fn get_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        branch: Option<&str>,
    ) -> Result<FileContent, ureq::Error> {
        let mut api_path = format!("/repos/{owner}/{repo}/contents/{path}");
        if let Some(b) = branch {
            api_path.push_str(&format!("?ref={b}"));
        }
        self.get(&api_path)
    }

    /// Create or update a file.
    pub fn create_or_update_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        req: &CreateFileRequest,
    ) -> Result<FileOpResponse, ureq::Error> {
        let api_path = format!("/repos/{owner}/{repo}/contents/{path}");
        self.put(&api_path, req)
    }

    // ── User API ──

    /// Get authenticated user info.
    pub fn get_user(&self) -> Result<Serde_json::Value, ureq::Error> {
        self.get_raw("/user")
    }

    // ── Generic API ──

    /// Generic GitHub API call.
    pub fn api(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, ureq::Error> {
        let url = format!("{API_BASE}{path}");
        let req = match method.to_uppercase().as_str() {
            "GET" => self.agent.get(&url),
            "POST" => self.agent.post(&url),
            "PUT" => self.agent.put(&url),
            "PATCH" => self.agent.patch(&url),
            "DELETE" => self.agent.delete(&url),
            _ => return Err(ureq::Error::Status(405, ureq::Response::new(405, "Method Not Allowed", "")?)),
        };

        let req = req
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28");

        if let Some(b) = body {
            let json = serde_json::to_string(b).unwrap_or_default();
            req.set("Content-Type", "application/json")
                .send_string(&json)?
                .into_json()
        } else {
            req.call()?.into_json()
        }
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ──

/// Format a ureq error for display.
pub fn format_error(e: &ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.to_string();
            if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
                format!("HTTP {code}: {}", err.message)
            } else {
                format!("HTTP {code}: {body}")
            }
        }
        ureq::Error::Transport(t) => {
            let msg = t.to_string();
            // Redact potential token leakage
            if let Some(token) = resolve_token() {
                msg.replace(&token, "***")
            } else {
                msg
            }
        }
    }
}
