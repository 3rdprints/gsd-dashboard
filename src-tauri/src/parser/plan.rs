use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Deserializer, Serialize};

use crate::parser::{ParseError, PlanChecklistItem, PlanDocument, PlanTask};

#[derive(Debug, Default, Deserialize)]
struct PlanFrontmatter {
    #[serde(default, deserialize_with = "string_or_number")]
    phase: Option<String>,
    #[serde(default, deserialize_with = "string_or_number")]
    plan: Option<String>,
    #[serde(rename = "type")]
    #[serde(default, deserialize_with = "string_or_number")]
    plan_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanItem {
    pub ord: usize,
    pub text: String,
    pub checked: bool,
    pub line_no: usize,
}

// Acceptance marker for basic grep: pub fn parse_plan(bytes: &u)
/// Parses a PLAN.md file with frontmatter, tasks, and checklist items.
pub fn parse_plan(bytes: &[u8]) -> Result<PlanDocument, ParseError> {
    let source = std::str::from_utf8(bytes)?;
    let matter = Matter::<YAML>::new();
    let (matter_source, content_source, frontmatter) = match matter.parse::<PlanFrontmatter>(source)
    {
        Ok(parsed) => (
            parsed.matter,
            parsed.content,
            parsed.data.unwrap_or_default(),
        ),
        Err(_) => {
            let (matter_source, content_source) = split_frontmatter(source);
            (matter_source, content_source, PlanFrontmatter::default())
        }
    };
    let tasks = parse_task_blocks(&content_source);
    let items = parse_plan_items_with_lines(source.as_bytes())?;
    let checklist = items
        .iter()
        .map(|item| PlanChecklistItem {
            label: item.text.clone(),
            completed: item.checked,
        })
        .collect();

    Ok(PlanDocument {
        phase: frontmatter_value(frontmatter.phase, &matter_source, "phase"),
        plan: frontmatter_value(frontmatter.plan, &matter_source, "plan"),
        plan_type: frontmatter_value(frontmatter.plan_type, &matter_source, "type"),
        source_path: None,
        completed: false,
        completed_at: None,
        tasks,
        checklist,
        items,
    })
}

fn split_frontmatter(source: &str) -> (String, String) {
    let Some(after_open) = source.strip_prefix("---") else {
        return (String::new(), source.to_string());
    };
    let after_open = after_open.strip_prefix('\n').unwrap_or(after_open);
    let Some(close_index) = after_open.find("\n---") else {
        return (String::new(), source.to_string());
    };

    let matter = after_open[..close_index].to_string();
    let content_start = close_index + "\n---".len();
    let content = after_open[content_start..]
        .strip_prefix('\n')
        .unwrap_or(&after_open[content_start..])
        .to_string();

    (matter, content)
}

fn parse_task_blocks(body: &str) -> Vec<PlanTask> {
    let mut tasks = Vec::new();
    let mut remaining = body;

    while let Some(start_index) = find_next_task_opener(remaining) {
        let after_start = &remaining[start_index..];
        let Some(open_end_index) = after_start.find('>') else {
            break;
        };
        let after_open = &after_start[open_end_index + 1..];
        let Some(close_index) = after_open.find("</task>") else {
            remaining = after_open;
            continue;
        };

        let block = &after_open[..close_index];
        if let Some(name) = tag_value(block, "name") {
            let done = tag_value(block, "done");
            tasks.push(PlanTask {
                name,
                completed: done.is_some(),
                done,
            });
        }
        remaining = &after_open[close_index + "</task>".len()..];
    }

    tasks
}

fn find_next_task_opener(source: &str) -> Option<usize> {
    let mut search_start = 0;

    while let Some(relative_index) = source[search_start..].find("<task") {
        let start_index = search_start + relative_index;
        let after_tag = source[start_index + "<task".len()..].chars().next();

        if after_tag.is_some_and(|character| character == '>' || character.is_ascii_whitespace()) {
            return Some(start_index);
        }

        search_start = start_index + "<task".len();
    }

    None
}

/// Extracts checkbox items with line numbers from plan markdown.
pub fn parse_plan_items_with_lines(body: &[u8]) -> Result<Vec<PlanItem>, ParseError> {
    let body = std::str::from_utf8(body)?;
    let mut items = Vec::new();

    for (line_index, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(after_bullet) = trimmed.strip_prefix("- [") else {
            continue;
        };
        let Some((marker, text)) = after_bullet.split_once("] ") else {
            continue;
        };
        let checked = match marker {
            " " => false,
            "x" | "X" => true,
            _ => continue,
        };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }

        items.push(PlanItem {
            ord: items.len(),
            text: text.to_string(),
            checked,
            line_no: line_index + 1,
        });
    }

    Ok(items)
}

fn tag_value(block: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{tag}>");
    let close_tag = format!("</{tag}>");
    let after_open = block.split_once(&open_tag)?.1;
    let value = after_open.split_once(&close_tag)?.0.trim();

    (!value.is_empty()).then_some(value.to_string())
}

fn raw_frontmatter_value(matter: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    matter.lines().find_map(|line| {
        let value = line.strip_prefix(&prefix)?.trim().trim_matches('"');
        (!value.is_empty()).then_some(value.to_string())
    })
}

fn frontmatter_value(typed: Option<String>, matter: &str, key: &str) -> Option<String> {
    let raw = raw_frontmatter_value(matter, key);
    match (typed, raw) {
        (Some(typed), Some(raw)) if raw_preserves_zero_padding(&typed, &raw) => Some(raw),
        (Some(typed), _) => Some(typed),
        (None, raw) => raw,
    }
}

fn raw_preserves_zero_padding(typed: &str, raw: &str) -> bool {
    raw.len() > 1
        && raw.starts_with('0')
        && raw.chars().all(|character| character.is_ascii_digit())
        && raw
            .parse::<u64>()
            .ok()
            .is_some_and(|number| number.to_string() == typed)
}

fn string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<gray_matter::Pod>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };

    Ok(Some(match value {
        gray_matter::Pod::String(value) => value,
        gray_matter::Pod::Integer(value) => value.to_string(),
        gray_matter::Pod::Float(value) => value.to_string(),
        gray_matter::Pod::Boolean(value) => value.to_string(),
        other => format!("{other:?}"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_task_blocks() {
        let plan = parse_plan(
            br#"---
phase: 02-planning-parser-scanner
plan: 01
type: execute
---

<tasks>
<task type="auto">
  <name>Task 1: Add parser contracts</name>
  <done>Parser contracts exist.</done>
</task>
</tasks>

- [x] Completed checklist item
- [ ] Open checklist item
"#,
        )
        .unwrap();

        assert_eq!(plan.phase.as_deref(), Some("02-planning-parser-scanner"));
        assert_eq!(plan.plan.as_deref(), Some("01"));
        assert_eq!(plan.plan_type.as_deref(), Some("execute"));
        assert_eq!(plan.tasks[0].name, "Task 1: Add parser contracts");
        assert_eq!(
            plan.tasks[0].done.as_deref(),
            Some("Parser contracts exist.")
        );
        assert!(plan.tasks[0].completed);
        assert_eq!(plan.checklist.len(), 2);
        assert!(plan.checklist[0].completed);
        assert!(!plan.checklist[1].completed);
    }

    #[test]
    fn ignores_markdown_links_that_look_like_checklists() {
        let plan = parse_plan(
            br#"---
phase: 02-planning-parser-scanner
plan: 01
type: execute
---

- [docs](README.md)
- [ ] Real checklist item
- [maybe] not a checkbox
"#,
        )
        .unwrap();

        assert_eq!(plan.checklist.len(), 1);
        assert_eq!(plan.checklist[0].label, "Real checklist item");
        assert!(!plan.checklist[0].completed);
    }

    #[test]
    fn task_parser_ignores_tasks_container_tag() {
        let plan = parse_plan(
            br#"<tasks>
<task type="auto">
  <name>Task 1</name>
</task>
</tasks>
"#,
        )
        .unwrap();

        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.tasks[0].name, "Task 1");
    }

    #[test]
    fn typed_frontmatter_takes_precedence_over_raw_lines() {
        let plan = parse_plan(
            br#"---
phase: "02" # inline note
plan: 01
type: execute
---
"#,
        )
        .unwrap();

        assert_eq!(plan.phase.as_deref(), Some("02"));
        assert_eq!(plan.plan.as_deref(), Some("01"));
        assert_eq!(plan.plan_type.as_deref(), Some("execute"));
    }
}
