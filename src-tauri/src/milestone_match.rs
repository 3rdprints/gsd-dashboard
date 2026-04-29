pub fn milestone_names_match(left: &str, right: &str) -> bool {
    let left = normalize_milestone_name(left);
    let right = normalize_milestone_name(right);

    left == right
}

fn normalize_milestone_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::milestone_names_match;

    #[test]
    fn milestone_names_match_requires_normalized_equality() {
        assert!(milestone_names_match("Auth API", "auth-api"));
        assert!(!milestone_names_match("Auth", "Auth API"));
        assert!(!milestone_names_match("Auth API", "Auth UI"));
    }
}
