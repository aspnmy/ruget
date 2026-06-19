use std::env;

const ENV_TOKEN: &str = "GITHUB_TOKEN";
const ENV_PAT: &str = "GITHUB_PERSONAL_ACCESS_TOKEN";

/// Resolve GitHub PAT from environment.
/// Returns `None` if no token is set — caller must handle.
pub fn resolve_token() -> Option<String> {
    for var in [ENV_TOKEN, ENV_PAT] {
        if let Ok(t) = env::var(var) {
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    None
}

/// Mask a token for safe display (show first 4 + last 4 chars).
pub fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "***".to_string();
    }
    format!("{}...{}", &token[..4], &token[token.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_token_short() {
        assert_eq!(mask_token("abc"), "***");
    }

    #[test]
    fn test_mask_token_long() {
        let masked = mask_token("ghp_abcdefgh12345678");
        assert!(masked.starts_with("ghp_"));
        assert!(masked.ends_with("5678"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn test_resolve_token_from_env() {
        env::set_var("GITHUB_TOKEN", "test_token_123");
        assert_eq!(resolve_token(), Some("test_token_123".to_string()));
        env::remove_var("GITHUB_TOKEN");
    }

    #[test]
    fn test_resolve_token_not_set() {
        env::remove_var("GITHUB_TOKEN");
        env::remove_var("GITHUB_PERSONAL_ACCESS_TOKEN");
        assert_eq!(resolve_token(), None);
    }
}
