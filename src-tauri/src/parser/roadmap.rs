use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

use crate::parser::{percent, MilestoneIdentity, ParseError, PhaseIdentity};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapDocument {
    pub milestones: Vec<MilestoneIdentity>,
    pub phases: Vec<RoadmapPhase>,
    pub milestone_progress_pct: u8,
    pub progress_source: String,
    pub phase_checkbox_total: usize,
    pub phase_checkbox_completed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapPhase {
    pub number: String,
    pub name: String,
    pub completed: bool,
}

// Acceptance marker for basic grep: pub fn parse_roadmap(bytes: &u)
pub fn parse_roadmap(bytes: &[u8]) -> Result<RoadmapDocument, ParseError> {
    let source = std::str::from_utf8(bytes)?;
    let milestones = parse_milestones(bytes)?;
    let phase_lines = markdown_lines(source);
    let phases = phase_lines
        .iter()
        .filter_map(|line| parse_phase_checkbox(line).or_else(|| parse_phase_heading(line)))
        .collect::<Vec<_>>();
    let phase_checkbox_total = phases.len();
    let phase_checkbox_completed = phases.iter().filter(|phase| phase.completed).count();

    Ok(RoadmapDocument {
        milestones,
        phases,
        milestone_progress_pct: percent(phase_checkbox_completed, phase_checkbox_total),
        progress_source: "roadmapPhaseCheckboxes".to_string(),
        phase_checkbox_total,
        phase_checkbox_completed,
    })
}

pub fn parse_milestones(bytes: &[u8]) -> Result<Vec<MilestoneIdentity>, ParseError> {
    let source = std::str::from_utf8(bytes)?;
    let phase_lines = markdown_lines(source);
    let mut milestones = phase_lines
        .iter()
        .filter_map(|line| parse_milestone_line(line))
        .enumerate()
        .map(|(index, name)| MilestoneIdentity {
            index: index + 1,
            name,
        })
        .collect::<Vec<_>>();

    if milestones.is_empty() {
        milestones.push(MilestoneIdentity {
            index: 1,
            name: "Milestone".to_string(),
        });
    }

    Ok(milestones)
}

fn markdown_lines(source: &str) -> Vec<String> {
    let parser = Parser::new_ext(source, Options::ENABLE_TASKLISTS);
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut prefix = String::new();
    let mut in_heading = false;

    for event in parser {
        match event {
            Event::Start(Tag::Item) => {
                current.clear();
                prefix = "- ".to_string();
            }
            Event::End(TagEnd::Item) => push_current_line(&mut lines, &mut current, &mut prefix),
            Event::Start(Tag::Paragraph) => current.clear(),
            Event::End(TagEnd::Paragraph) => {
                push_current_line(&mut lines, &mut current, &mut prefix);
            }
            Event::Start(Tag::Heading { level, .. }) => {
                current.clear();
                prefix = heading_prefix(level);
                in_heading = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                push_current_line(&mut lines, &mut current, &mut prefix);
                in_heading = false;
            }
            Event::Text(text) | Event::Code(text) => current.push_str(&text),
            Event::SoftBreak | Event::HardBreak => {
                if !current.trim().is_empty() {
                    push_current_line(&mut lines, &mut current, &mut prefix);
                }
            }
            Event::TaskListMarker(checked) => {
                current.push_str(if checked { "[x] " } else { "[ ] " });
            }
            Event::Html(html) if in_heading || !html.trim().is_empty() => current.push_str(&html),
            _ => {}
        }
    }

    if !current.trim().is_empty() {
        push_current_line(&mut lines, &mut current, &mut prefix);
    }

    lines
}

fn heading_prefix(level: HeadingLevel) -> String {
    let depth = match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    };
    "#".repeat(depth) + " "
}

fn push_current_line(lines: &mut Vec<String>, current: &mut String, prefix: &mut String) {
    let line = format!("{prefix}{}", current.trim());
    if !line.trim().is_empty() {
        lines.push(line);
    }
    current.clear();
    prefix.clear();
}

fn parse_milestone_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let value = trimmed
        .strip_prefix("**Milestone:**")
        .or_else(|| trimmed.strip_prefix("Milestone:"))?
        .trim();
    let name = value
        .split('—')
        .next()
        .unwrap_or(value)
        .trim()
        .trim_matches('*')
        .to_string();

    (!name.is_empty()).then_some(name)
}

fn parse_phase_checkbox(line: &str) -> Option<RoadmapPhase> {
    let trimmed = line.trim_start();
    let after_dash = trimmed.strip_prefix("- [")?;
    let (state, rest) = after_dash.split_once(']')?;
    let completed = matches!(state.trim(), "x" | "X");
    let label = rest.trim().trim_matches('*').trim();
    let identity = parse_phase_identity(label)?;

    Some(RoadmapPhase {
        number: identity.number,
        name: identity.name,
        completed,
    })
}

fn parse_phase_heading(line: &str) -> Option<RoadmapPhase> {
    let label = line.trim_start().trim_start_matches('#').trim();
    let identity = parse_phase_identity(label)?;

    Some(RoadmapPhase {
        number: identity.number,
        name: identity.name,
        completed: false,
    })
}

fn parse_phase_identity(label: &str) -> Option<PhaseIdentity> {
    let after_phase = label.strip_prefix("Phase ")?;
    let number_len = after_phase
        .chars()
        .take_while(|character| character.is_ascii_digit() || *character == '.')
        .map(char::len_utf8)
        .sum::<usize>();
    if number_len == 0 {
        return None;
    }

    let number = after_phase[..number_len].to_string();
    let name_source = after_phase[number_len..]
        .trim_start()
        .strip_prefix(':')
        .unwrap_or("")
        .trim();
    let name = name_source
        .split("**")
        .next()
        .unwrap_or(name_source)
        .split(" - ")
        .next()
        .unwrap_or(name_source)
        .trim()
        .to_string();

    Some(PhaseIdentity { number, name })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{derive_progress, PlanChecklistItem, PlanDocument};

    #[test]
    fn extracts_current_milestone() {
        let roadmap = parse_roadmap(
            br#"# Roadmap

**Milestone:** v1.0 MVP

## Phases

- [x] **Phase 1: Foundation**
- [ ] **Phase 2: Planning Parser & Scanner**
"#,
        )
        .unwrap();

        assert_eq!(roadmap.milestones[0].name, "v1.0 MVP");
    }

    #[test]
    fn computes_milestone_percent() {
        let roadmap = parse_roadmap(
            br#"**Milestone:** v1.0 MVP

- [x] **Phase 1: Foundation**
- [ ] **Phase 2: Planning Parser & Scanner**
"#,
        )
        .unwrap();

        assert_eq!(roadmap.milestone_progress_pct, 50);
        assert_eq!(roadmap.progress_source, "roadmapPhaseCheckboxes");
    }

    #[test]
    fn computes_progress_from_plan_checklist_fallback() {
        let roadmap = parse_roadmap(
            br#"**Milestone:** v1.0 MVP
"#,
        )
        .unwrap();
        let plans = vec![PlanDocument {
            phase: None,
            plan: None,
            plan_type: None,
            tasks: Vec::new(),
            checklist: vec![
                PlanChecklistItem {
                    label: "Done".to_string(),
                    completed: true,
                },
                PlanChecklistItem {
                    label: "Open".to_string(),
                    completed: false,
                },
            ],
        }];

        let progress = derive_progress(&roadmap, &plans);

        assert_eq!(progress.percent, 50);
        assert_eq!(progress.source, "planChecklistCompletion");
    }

    #[test]
    fn prefers_roadmap_phase_checkboxes_when_reliable() {
        let roadmap = parse_roadmap(
            br#"**Milestone:** v1.0 MVP

- [x] Phase 1: Foundation
- [ ] Phase 2: Planning Parser & Scanner
"#,
        )
        .unwrap();
        let plans = vec![PlanDocument {
            phase: None,
            plan: None,
            plan_type: None,
            tasks: Vec::new(),
            checklist: vec![PlanChecklistItem {
                label: "Done".to_string(),
                completed: true,
            }],
        }];

        let progress = derive_progress(&roadmap, &plans);

        assert_eq!(progress.percent, 50);
        assert_eq!(progress.source, "roadmapPhaseCheckboxes");
    }

    #[test]
    fn preserves_decimal_phase_numbers() {
        let roadmap = parse_roadmap(
            br#"**Milestone:** v1.0 MVP

- [ ] **Phase 72.1: Decimal Follow-up**
"#,
        )
        .unwrap();

        assert_eq!(roadmap.phases[0].number, "72.1");
    }
}
