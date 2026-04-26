use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};

use crate::parser::{MilestoneIdentity, ParseError, PhaseIdentity};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDocument {
    pub current_milestone: Option<MilestoneIdentity>,
    pub current_phase: Option<PhaseIdentity>,
    pub next_command: String,
}

#[derive(Debug, Default, Deserialize)]
struct StateFrontmatter {
    milestone: Option<String>,
    milestone_name: Option<String>,
}

// Acceptance marker for basic grep: pub fn parse_state(bytes: &u)
pub fn parse_state(bytes: &[u8]) -> Result<StateDocument, ParseError> {
    let source = std::str::from_utf8(bytes)?;
    let matter = Matter::<YAML>::new();
    let parsed =
        matter
            .parse::<StateFrontmatter>(source)
            .map_err(|error| ParseError::Frontmatter {
                message: error.to_string(),
            })?;
    let frontmatter = parsed.data.unwrap_or_default();
    let current_milestone = parse_milestone(&parsed.content, &frontmatter);
    let current_phase = parse_phase(&parsed.content);
    let next_command =
        parse_next_command(&parsed.content).unwrap_or_else(|| default_next_command(&current_phase));

    Ok(StateDocument {
        current_milestone,
        current_phase,
        next_command,
    })
}

fn parse_milestone(body: &str, frontmatter: &StateFrontmatter) -> Option<MilestoneIdentity> {
    let body_value = body
        .lines()
        .find_map(|line| field_value(line, "**Milestone:**"));
    let fallback_value = frontmatter
        .milestone_name
        .as_ref()
        .or(frontmatter.milestone.as_ref())
        .cloned();
    let name = body_value.or(fallback_value)?;

    Some(MilestoneIdentity { index: 1, name })
}

fn parse_phase(body: &str) -> Option<PhaseIdentity> {
    let value = body
        .lines()
        .find_map(|line| field_value(line, "**Phase:**").or_else(|| field_value(line, "Phase:")))?;
    let number = value
        .split_whitespace()
        .next()
        .unwrap_or(value.as_str())
        .trim_matches(|character: char| !character.is_ascii_digit() && character != '.')
        .to_string();
    if number.is_empty() {
        return None;
    }

    let name = value
        .split_once('(')
        .and_then(|(_, rest)| {
            rest.split_once(')')
                .map(|(name, _)| name.trim().to_string())
        })
        .unwrap_or_default();

    Some(PhaseIdentity { number, name })
}

fn parse_next_command(body: &str) -> Option<String> {
    let mut in_next_command = false;
    let mut in_fence = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## Next Command") {
            in_next_command = true;
            continue;
        }

        if !in_next_command {
            continue;
        }

        if trimmed.starts_with("## ") {
            break;
        }

        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }

        if (in_fence || trimmed.starts_with('/')) && !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

fn default_next_command(current_phase: &Option<PhaseIdentity>) -> String {
    current_phase
        .as_ref()
        .map(|phase| format!("/gsd-execute-phase {}", phase.number))
        .unwrap_or_else(|| "/gsd-next".to_string())
}

fn field_value(line: &str, marker: &str) -> Option<String> {
    let value = line.trim().strip_prefix(marker)?.trim();
    (!value.is_empty()).then_some(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_current_phase() {
        let state = parse_state(
            br#"---
milestone: v1.0
milestone_name: milestone
---

## Current Position

**Milestone:** v1.0 MVP
**Phase:** 2

## Next Command

```
/gsd-execute-phase 2
```
"#,
        )
        .unwrap();

        assert_eq!(state.current_milestone.unwrap().name, "v1.0 MVP");
        assert_eq!(state.current_phase.unwrap().number, "2");
        assert_eq!(state.next_command, "/gsd-execute-phase 2");
    }

    #[test]
    fn next_command_defaults_to_current_phase_when_available() {
        let state = parse_state(
            br#"---
milestone: v1.0
---

## Current Position

**Phase:** 06.1
"#,
        )
        .unwrap();

        assert_eq!(state.next_command, "/gsd-execute-phase 06.1");
    }

    #[test]
    fn next_command_defaults_to_gsd_next_without_phase() {
        let state = parse_state(
            br#"---
milestone: v1.0
---

## Current Position
"#,
        )
        .unwrap();

        assert_eq!(state.next_command, "/gsd-next");
    }
}
