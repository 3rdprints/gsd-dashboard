#[test]
fn state_excerpt_extracts_current_position_and_next_command() {
    let excerpt = gsd_dashboard::parser::state::extract_state_excerpt(
        r#"# State: Demo

Intro line

## Current Position

Phase: 05 (project-detail)
Plan: 4 of 12

## Next Command

```
/gsd-next
```
"#,
        20,
        2048,
    )
    .expect("excerpt should parse");

    assert!(excerpt.contains("Phase: 05 (project-detail)"));
    assert!(excerpt.contains("Plan: 4 of 12"));
    assert!(!excerpt.contains("## Next Command"));
}

#[test]
fn state_excerpt_caps_lines_and_falls_back_without_heading() {
    let body = (1..=30)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let excerpt = gsd_dashboard::parser::state::extract_state_excerpt(&body, 20, 2048)
        .expect("fallback excerpt should parse");

    assert_eq!(excerpt.lines().count(), 20);
    assert!(excerpt.contains("line 20"));
    assert!(!excerpt.contains("line 21"));
}
