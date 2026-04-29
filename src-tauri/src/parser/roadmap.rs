use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    #[serde(default)]
    pub milestone_name: Option<String>,
}

// Acceptance marker for basic grep: pub fn parse_roadmap(bytes: &u)
pub fn parse_roadmap(bytes: &[u8]) -> Result<RoadmapDocument, ParseError> {
    let source = std::str::from_utf8(bytes)?;
    let milestones = parse_milestones(bytes)?;
    let phase_lines = raw_markdown_lines(source);
    let checkbox_phases = parse_phase_checkboxes(&phase_lines);
    let completion_by_number = checkbox_phases
        .iter()
        .map(|phase| (phase_key(&phase.number), phase.completed))
        .collect::<BTreeMap<_, _>>();
    let heading_phases = phase_lines
        .iter()
        .filter_map(|line| parse_phase_heading(line))
        .map(|mut phase| {
            if let Some(completed) = completion_by_number.get(&phase_key(&phase.number)) {
                phase.completed = *completed;
            }
            phase
        })
        .collect::<Vec<_>>();
    let phases = if heading_phases.is_empty() {
        checkbox_phases.clone()
    } else {
        heading_phases
    };
    let phase_checkbox_total = checkbox_phases.len();
    let phase_checkbox_completed = checkbox_phases
        .iter()
        .filter(|phase| phase.completed)
        .count();

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
    let phase_lines = raw_markdown_lines(source);
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

fn raw_markdown_lines(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_milestone_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if let Some(value) = trimmed.strip_prefix("- ") {
        let value = value.trim_start_matches(['❌', '✅', '🚫', '⛔']).trim();
        let value = value.strip_prefix("**").unwrap_or(value);
        let name = value
            .split("**")
            .next()
            .unwrap_or(value)
            .split('—')
            .next()
            .unwrap_or(value)
            .trim()
            .to_string();

        return (!name.is_empty()).then_some(name);
    }

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

fn parse_phase_checkboxes(lines: &[String]) -> Vec<RoadmapPhase> {
    let mut phases = Vec::new();
    let mut current_milestone = None;

    for line in lines {
        if let Some(milestone_name) = parse_summary_milestone(line) {
            current_milestone = Some(milestone_name);
            continue;
        }

        if line.trim() == "</details>" {
            current_milestone = None;
            continue;
        }

        if let Some(mut phase) = parse_phase_checkbox(line) {
            phase.milestone_name = current_milestone.clone();
            phases.push(phase);
        }
    }

    phases
}

fn parse_summary_milestone(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let after_summary = trimmed
        .strip_prefix("<summary>")
        .or_else(|| trimmed.strip_prefix("# "))?
        .trim_start_matches(['❌', '✅', '🚫', '⛔'])
        .trim();
    let name = after_summary
        .split(" (Phases")
        .next()
        .unwrap_or(after_summary)
        .split('—')
        .next()
        .unwrap_or(after_summary)
        .trim()
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
        milestone_name: None,
    })
}

fn parse_phase_heading(line: &str) -> Option<RoadmapPhase> {
    let label = line.trim_start().trim_start_matches('#').trim();
    let identity = parse_phase_identity(label)?;

    Some(RoadmapPhase {
        number: identity.number,
        name: identity.name,
        completed: false,
        milestone_name: None,
    })
}

fn parse_phase_identity(label: &str) -> Option<PhaseIdentity> {
    let after_phase = label.strip_prefix("Phase ")?;
    let number_len = after_phase
        .chars()
        .take_while(|character| {
            character.is_ascii_alphanumeric() || *character == '.' || *character == '-'
        })
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
    let without_inserted = name_source.replace("(INSERTED)", "");
    let name = without_inserted
        .split("**")
        .next()
        .unwrap_or(without_inserted.as_str())
        .split(" - ")
        .next()
        .unwrap_or(without_inserted.as_str())
        .trim()
        .to_string();

    Some(PhaseIdentity { number, name })
}

fn phase_key(number: &str) -> String {
    let without_project_code = number
        .split_once('-')
        .and_then(|(prefix, rest)| {
            (prefix
                .chars()
                .all(|character| character.is_ascii_alphabetic())
                && rest
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_digit()))
            .then_some(rest)
        })
        .unwrap_or(number);
    let stripped = without_project_code
        .trim_start_matches('0')
        .trim()
        .to_ascii_lowercase();

    if stripped.is_empty() {
        "0".to_string()
    } else {
        stripped
    }
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
            source_path: None,
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
            items: Vec::new(),
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
            source_path: None,
            tasks: Vec::new(),
            checklist: vec![PlanChecklistItem {
                label: "Done".to_string(),
                completed: true,
            }],
            items: Vec::new(),
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

    #[test]
    fn uses_phase_detail_headings_and_checkboxes_as_completion_markers() {
        let roadmap = parse_roadmap(
            br#"## Roadmap v1.0: MVP

- [x] **Phase 01: Foundation**
- [ ] **Phase 02: Parser**

### Phase 01: Foundation

**Plans:** 1/1 plans complete

### Phase 02: Parser

**Plans:** 0/2 plans executed
"#,
        )
        .unwrap();

        assert_eq!(roadmap.phases.len(), 2);
        assert_eq!(roadmap.phases[0].number, "01");
        assert!(roadmap.phases[0].completed);
        assert_eq!(roadmap.phases[1].number, "02");
        assert!(!roadmap.phases[1].completed);
    }

    #[test]
    fn parses_letter_decimal_and_project_code_phase_tokens() {
        let roadmap = parse_roadmap(
            br#"### Phase CK-12A.1: Inserted Follow-up (INSERTED)
"#,
        )
        .unwrap();

        assert_eq!(roadmap.phases[0].number, "CK-12A.1");
        assert_eq!(roadmap.phases[0].name, "Inserted Follow-up");
    }
}
