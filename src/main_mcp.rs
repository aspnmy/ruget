//! MCP Server — JSON-RPC over stdio.
//! Minimal implementation supporting the MCP 2024-11-05 protocol.

use ruget::{client, client::GitHubClient, models, models::ToolResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

const TOOLS: &[(&str, &str)] = &[
    ("github_api", "Generic GitHub REST API call. method (GET/POST/PUT/DELETE), path (/search/...), body (JSON string optional)"),
    ("search_repositories", "Search GitHub repositories. query, page, per_page, sort (optional), order (optional)"),
    ("search_code", "Search GitHub code. query, page, per_page"),
    ("search_issues", "Search GitHub issues and PRs. query, page, per_page, sort (optional)"),
    ("create_issue", "Create a GitHub issue. owner, repo, title, body (optional), labels (comma-sep optional), assignees (comma-sep optional)"),
    ("list_issues", "List repository issues. owner, repo, state, page, per_page, labels (optional)"),
    ("create_pull_request", "Create a pull request. owner, repo, title, head, base, body (optional), draft (optional)"),
    ("get_file_contents", "Get file contents. owner, repo, path, branch (optional)"),
    ("create_or_update_file", "Create or update a file. owner, repo, path, content, message, branch (optional), sha (optional for updates)"),
];

fn main() {
    let client = GitHubClient::new();

    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin.lock());

    loop {
        let mut line = String::new();
        match std::io::BufRead::read_line(&mut reader, &mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                send_error(None, -32700, &format!("Parse error: {e}"));
                continue;
            }
        };

        match req.method.as_str() {
            "initialize" => {
                let result = serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "ruget-mcp",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "tools": {}
                    }
                });
                send_ok(req.id, &result);
            }
            "tools/list" => {
                let tools: Vec<serde_json::Value> = TOOLS
                    .iter()
                    .map(|(name, desc)| {
                        serde_json::json!({
                            "name": name,
                            "description": desc,
                            "inputSchema": {
                                "type": "object",
                                "properties": {}
                            }
                        })
                    })
                    .collect();
                let result = serde_json::json!({ "tools": tools });
                send_ok(req.id, &result);
            }
            "tools/call" => {
                handle_tool_call(&client, req);
            }
            "notifications/initialized" => {
                // No response needed for notifications
            }
            _ => {
                send_error(req.id, -32601, &format!("Method not found: {}", req.method));
            }
        }
    }
}

fn handle_tool_call(client: &GitHubClient, req: JsonRpcRequest) {
    let params = match req.params {
        Some(p) => p,
        None => {
            send_error(req.id, -32602, "Missing params");
            return;
        }
    };

    let tool_name = params["name"].as_str().unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(serde_json::Value::Null);

    let result = match tool_name {
        "github_api" => tool_github_api(client, &args),
        "search_repositories" => tool_search_repositories(client, &args),
        "search_code" => tool_search_code(client, &args),
        "search_issues" => tool_search_issues(client, &args),
        "create_issue" => tool_create_issue(client, &args),
        "list_issues" => tool_list_issues(client, &args),
        "create_pull_request" => tool_create_pr(client, &args),
        "get_file_contents" => tool_get_file(client, &args),
        "create_or_update_file" => tool_create_file(client, &args),
        _ => ToolResult {
            success: false,
            data: None,
            error: Some(format!("Unknown tool: {tool_name}")),
        },
    };

    let content = serde_json::json!([{
        "type": "text",
        "text": serde_json::to_string(&result).unwrap_or_default()
    }]);

    send_ok(req.id, &serde_json::json!({ "content": content }));
}

// ── Tool implementations ──

fn tool_github_api(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let method = args["method"].as_str().unwrap_or("GET");
    let path = args["path"].as_str().unwrap_or("/");
    let body = args.get("body");
    let timeout = args["timeout"].as_u64().unwrap_or(30);

    let _ = timeout; // ureq agent has fixed timeout, could be configurable in future

    match client.api(method, path, body) {
        Ok(data) => ToolResult {
            success: true,
            data: Some(data),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

fn tool_search_repositories(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let query = args["query"].as_str().unwrap_or("");
    let page = args["page"].as_u64().unwrap_or(1) as u32;
    let per_page = args["per_page"].as_u64().unwrap_or(30) as u32;
    let sort = args["sort"].as_str();
    let order = args["order"].as_str().unwrap_or("desc");

    match client.search_repositories(query, page, per_page, sort, Some(order)) {
        Ok(result) => ToolResult {
            success: true,
            data: Some(serde_json::to_value(result).unwrap_or_default()),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

fn tool_search_code(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let query = args["query"].as_str().unwrap_or("");
    let page = args["page"].as_u64().unwrap_or(1) as u32;
    let per_page = args["per_page"].as_u64().unwrap_or(30) as u32;

    match client.search_code(query, page, per_page) {
        Ok(result) => ToolResult {
            success: true,
            data: Some(serde_json::to_value(result).unwrap_or_default()),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

fn tool_search_issues(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let query = args["query"].as_str().unwrap_or("");
    let page = args["page"].as_u64().unwrap_or(1) as u32;
    let per_page = args["per_page"].as_u64().unwrap_or(30) as u32;
    let sort = args["sort"].as_str();
    let order = args["order"].as_str().unwrap_or("desc");

    match client.search_issues(query, page, per_page, sort, Some(order)) {
        Ok(result) => ToolResult {
            success: true,
            data: Some(serde_json::to_value(result).unwrap_or_default()),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

fn tool_create_issue(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let owner = args["owner"].as_str().unwrap_or("");
    let repo = args["repo"].as_str().unwrap_or("");

    let labels: Option<Vec<String>> = args["labels"].as_str()
        .map(|l| l.split(',').map(|s| s.trim().to_string()).collect());

    let assignees: Option<Vec<String>> = args["assignees"].as_str()
        .map(|a| a.split(',').map(|s| s.trim().to_string()).collect());

    let req = models::CreateIssueRequest {
        title: args["title"].as_str().unwrap_or("").to_string(),
        body: args["body"].as_str().map(|s| s.to_string()),
        labels,
        assignees,
    };

    match client.create_issue(owner, repo, &req) {
        Ok(issue) => ToolResult {
            success: true,
            data: Some(serde_json::to_value(issue).unwrap_or_default()),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

fn tool_list_issues(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let owner = args["owner"].as_str().unwrap_or("");
    let repo = args["repo"].as_str().unwrap_or("");
    let state = args["state"].as_str().unwrap_or("open");
    let page = args["page"].as_u64().unwrap_or(1) as u32;
    let per_page = args["per_page"].as_u64().unwrap_or(30) as u32;
    let labels = args["labels"].as_str();

    match client.list_issues(owner, repo, state, page, per_page, labels) {
        Ok(issues) => ToolResult {
            success: true,
            data: Some(serde_json::to_value(issues).unwrap_or_default()),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

fn tool_create_pr(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let owner = args["owner"].as_str().unwrap_or("");
    let repo = args["repo"].as_str().unwrap_or("");

    let req = models::CreatePRRequest {
        title: args["title"].as_str().unwrap_or("").to_string(),
        head: args["head"].as_str().unwrap_or("").to_string(),
        base: args["base"].as_str().unwrap_or("").to_string(),
        body: args["body"].as_str().map(|s| s.to_string()),
        draft: args["draft"].as_bool(),
    };

    match client.create_pr(owner, repo, &req) {
        Ok(pr) => ToolResult {
            success: true,
            data: Some(pr),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

fn tool_get_file(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let owner = args["owner"].as_str().unwrap_or("");
    let repo = args["repo"].as_str().unwrap_or("");
    let path = args["path"].as_str().unwrap_or("");
    let branch = args["branch"].as_str();

    match client.get_file(owner, repo, path, branch) {
        Ok(file) => ToolResult {
            success: true,
            data: Some(serde_json::to_value(file).unwrap_or_default()),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

fn tool_create_file(client: &GitHubClient, args: &serde_json::Value) -> ToolResult {
    let owner = args["owner"].as_str().unwrap_or("");
    let repo = args["repo"].as_str().unwrap_or("");
    let path = args["path"].as_str().unwrap_or("");
    let content = args["content"].as_str().unwrap_or("");
    let message = args["message"].as_str().unwrap_or("Update file via ruget");

    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());

    let req = models::CreateFileRequest {
        message: message.to_string(),
        content: encoded,
        branch: args["branch"].as_str().map(|s| s.to_string()),
        sha: args["sha"].as_str().map(|s| s.to_string()),
    };

    match client.create_or_update_file(owner, repo, path, &req) {
        Ok(resp) => ToolResult {
            success: true,
            data: Some(serde_json::to_value(resp).unwrap_or_default()),
            error: None,
        },
        Err(e) => ToolResult {
            success: false,
            data: None,
            error: Some(client::format_error(&e)),
        },
    }
}

// ── JSON-RPC helpers ──

fn send_ok(id: Option<serde_json::Value>, result: &serde_json::Value) {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result.clone()),
        error: None,
    };
    println!("{}", serde_json::to_string(&resp).unwrap_or_default());
}

fn send_error(id: Option<serde_json::Value>, code: i32, message: &str) {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
        }),
    };
    eprintln!("{}", serde_json::to_string(&resp).unwrap_or_default());
}
