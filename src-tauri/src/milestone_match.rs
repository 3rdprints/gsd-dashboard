pub fn milestone_names_match(left: &str, right: &str) -> bool {
    let left = normalize_milestone_name(left);
    let right = normalize_milestone_name(right);

    left == right
        || contains_milestone_token(&left, &right)
        || contains_milestone_token(&right, &left)
}

fn contains_milestone_token(haystack: &str, needle: &str) -> bool {
    let needle_tokens = needle.split_whitespace().collect::<Vec<_>>();
    !needle_tokens.is_empty()
        && needle.len() >= 3
        && haystack
            .split_whitespace()
            .collect::<Vec<_>>()
            .windows(needle_tokens.len())
            .any(|window| window == needle_tokens.as_slice())
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
