//! ruget CLI entry point.

mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use ruget::{client::GitHubClient, models};

fn main() {
    let cli = Cli::parse();

    let client = match &cli.token {
        Some(t) => GitHubClient::with_token(t.clone()),
        None => GitHubClient::new(),
    };

    match &cli.command {
        Commands::Search { kind } => match kind {
            cli::SearchKind::Repos { q, page, per_page, sort, order } => {
                match client.search_repositories(q, *page, *per_page, sort.as_deref(), Some(order)) {
                    Ok(result) => {
                        println!("Found {} repositories (page {})", result.total_count, page);
                        for repo in &result.items {
                            println!(
                                "  {:>6}⭐  {}/{}  {}",
                                repo.stargazers_count,
                                repo.full_name,
                                repo.language.as_deref().unwrap_or("-"),
                                repo.description.as_deref().unwrap_or("")
                            );
                        }
                    }
                    Err(e) => eprintln!("Error: {}", client::format_error(&e)),
                }
            }
            cli::SearchKind::Code { q, page, per_page } => {
                match client.search_code(q, *page, *per_page) {
                    Ok(result) => {
                        println!("Found {} code matches", result.total_count);
                        for m in &result.items {
                            let repo = m.repository.as_ref().map(|r| r.full_name.as_str()).unwrap_or("?");
                            println!("  {}  →  {}  ({})", repo, m.path, m.html_url);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", client::format_error(&e)),
                }
            }
            cli::SearchKind::Issues { q, page, per_page, sort } => {
                match client.search_issues(q, *page, *per_page, sort.as_deref(), None) {
                    Ok(result) => {
                        println!("Found {} issues/PRs", result.total_count);
                        for item in &result.items {
                            println!(
                                "  #{:<6} [{}] {}  ({})",
                                item.number,
                                item.state,
                                item.title,
                                item.html_url
                            );
                        }
                    }
                    Err(e) => eprintln!("Error: {}", client::format_error(&e)),
                }
            }
        },
        Commands::Issue { action } => match action {
            cli::IssueAction::Create { owner, repo, title, body, labels, assignees } => {
                let req = models::CreateIssueRequest {
                    title: title.clone(),
                    body: body.clone(),
                    labels: labels.as_ref().map(|l| l.split(',').map(|s| s.trim().to_string()).collect()),
                    assignees: assignees.as_ref().map(|a| a.split(',').map(|s| s.trim().to_string()).collect()),
                };
                match client.create_issue(owner, repo, &req) {
                    Ok(issue) => println!("Created issue #{}: {}", issue.number, issue.html_url),
                    Err(e) => eprintln!("Error: {}", client::format_error(&e)),
                }
            }
            cli::IssueAction::List { owner, repo, state, page, labels } => {
                match client.list_issues(owner, repo, state, *page, 30, labels.as_deref()) {
                    Ok(issues) => {
                        for issue in &issues {
                            println!("  #{:<6} [{}] {}", issue.number, issue.state, issue.title);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", client::format_error(&e)),
                }
            }
        },
        Commands::Pr { action } => match action {
            cli::PrAction::Create { owner, repo, title, head, base, body, draft } => {
                let req = models::CreatePRRequest {
                    title: title.clone(),
                    head: head.clone(),
                    base: base.clone(),
                    body: body.clone(),
                    draft: if *draft { Some(true) } else { None },
                };
                match client.create_pr(owner, repo, &req) {
                    Ok(pr) => {
                        let url = pr["html_url"].as_str().unwrap_or("?");
                        println!("Created PR: {url}");
                    }
                    Err(e) => eprintln!("Error: {}", client::format_error(&e)),
                }
            }
        },
        Commands::File { action } => match action {
            cli::FileAction::Get { owner, repo, path, branch } => {
                match client.get_file(owner, repo, path, branch.as_deref()) {
                    Ok(file) => {
                        if let Some(ref content) = file.content {
                            // Base64 decode
                            use base64::Engine as _;
                            let decoded = base64::engine::general_purpose::STANDARD
                                .decode(content.replace('\n', ""))
                                .map(|b| String::from_utf8_lossy(&b).to_string())
                                .unwrap_or_else(|_| "[binary]".to_string());
                            println!("{}", decoded);
                        } else {
                            println!("[binary file, {} bytes]", file.size);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", client::format_error(&e)),
                }
            }
        },
        Commands::Api { method, path, data } => {
            let body: Option<serde_json::Value> = data.as_ref().and_then(|d| serde_json::from_str(d).ok());
            match client.api(method, path, body.as_ref()) {
                Ok(resp) => println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default()),
                Err(e) => eprintln!("Error: {}", client::format_error(&e)),
            }
        },
        Commands::Whoami => match client.get_user() {
            Ok(user) => {
                let login = user["login"].as_str().unwrap_or("?");
                let name = user["name"].as_str().unwrap_or("-");
                println!("Logged in as: {login} ({name})");
            }
            Err(e) => eprintln!("Error: {}", client::format_error(&e)),
        },
    }
}
