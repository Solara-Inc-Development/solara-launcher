//! Exact protected-resource allowlist. Provider credentials never reach the UI.
pub fn allowed(method: &str, path: &str) -> bool {
    if path.len() > 256 || path.contains(['?', '#', '%', '\\']) || path.contains("..") {
        return false;
    }
    match (method, path) {
        (
            "GET",
            "/v1/launcher/me"
            | "/v1/launcher/servers"
            | "/v1/launcher/entitlements"
            | "/v1/launcher/assignments",
        )
        | ("POST", "/v1/launcher/assignments") => true,
        _ => {
            let parts: Vec<_> = path.split('/').collect();
            if parts.len() == 5 && parts[1..4] == ["v1", "launcher", "assignments"] {
                return matches!(method, "PATCH" | "DELETE")
                    && parts[4].len() == 32
                    && parts[4]
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
            }
            parts.len() == 6
                && parts[1..4] == ["v1", "launcher", "servers"]
                && !parts[4].is_empty()
                && parts[4].len() <= 64
                && parts[4]
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && method == "POST"
                && parts[5] == "power"
        }
    }
}
#[cfg(test)]
mod tests {
    use super::allowed;
    #[test]
    fn owner_assignment_operations_have_exact_routes() {
        let path = "/v1/launcher/assignments/0123456789abcdef0123456789abcdef";
        assert!(allowed("GET", "/v1/launcher/assignments"));
        assert!(allowed("POST", "/v1/launcher/assignments"));
        assert!(allowed("PATCH", path));
        assert!(allowed("DELETE", path));
        for method in ["GET", "POST", "PUT", "HEAD"] {
            assert!(!allowed(method, path));
        }
        for suffix in [
            "",
            "abc",
            "0123456789abcdef0123456789abcdeF",
            "0123456789abcdef0123456789abcdef/",
            "0123456789abcdef0123456789abcdef?user=other",
            "%2e%2e",
        ] {
            assert!(!allowed(
                "PATCH",
                &format!("/v1/launcher/assignments/{suffix}")
            ));
        }
    }
    #[test]
    fn other_resources_remain_restricted() {
        assert!(allowed("POST", "/v1/launcher/servers/abcd-1234/power"));
        for path in [
            "/v1/launcher/servers//power",
            "/v1/launcher/servers/abc/console",
            "/v1/launcher/servers/abc/../power",
            "/v1/launcher/admin",
            "/v1/launcher/me?token=anything",
        ] {
            assert!(!allowed("POST", path));
        }
        assert!(!allowed("DELETE", "/v1/launcher/me"));
    }
}
