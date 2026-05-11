use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
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
    #[serde(default)]
    pub completed_plan_count: Option<usize>,
    #[serde(default)]
    pub total_plan_count: Option<usize>,
}

// Acceptance marker for basic grep: pub fn parse_roadmap(bytes: &u)
/// Parses a ROADMAP.md file into phases, milestones, and progress.
pub fn parse_roadmap(bytes: &[u8]) -> Result<RoadmapDocument, ParseError> {
    let source = std::str::from_utf8(bytes)?;
    let milestones = parse_milestones(bytes)?;
    let phase_lines = raw_markdown_lines(source);
    let checkbox_phases = parse_phase_checkboxes(&phase_lines);
    let completion_by_number = checkbox_phases
        .iter()
        .map(|phase| (phase_key(&phase.number), phase.completed))
        .collect::<BTreeMap<_, _>>();
    let plan_counts_by_number = parse_phase_plan_counts(&phase_lines);
    let heading_phases = phase_lines
        .iter()
        .filter_map(|line| parse_phase_heading(line))
        .map(|mut phase| {
            let key = phase_key(&phase.number);
            if let Some(completed) = completion_by_number.get(&key) {
                phase.completed = *completed;
            }
            apply_plan_counts(&mut phase, plan_counts_by_number.get(&key));
            phase
        })
        .collect::<Vec<_>>();
    let phases = if heading_phases.is_empty() {
        checkbox_phases
            .clone()
            .into_iter()
            .map(|mut phase| {
                let key = phase_key(&phase.number);
                apply_plan_counts(&mut phase, plan_counts_by_number.get(&key));
                phase
            })
            .collect()
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

/// Extracts milestone identities from a markdown document.
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
    let matter = Matter::<YAML>::new();
    let content = matter
        .parse::<gray_matter::Pod>(source)
        .map(|parsed| parsed.content)
        .unwrap_or_else(|_| source.to_string());
    let parser = Parser::new_ext(&content, Options::all());
    let mut rendered = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                push_line_prefix(&mut rendered, "# ");
            }
            Event::Start(Tag::Item) => {
                push_line_prefix(&mut rendered, "- ");
            }
            Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::HtmlBlock) => rendered.push('\n'),
            Event::Text(value)
            | Event::Code(value)
            | Event::Html(value)
            | Event::InlineHtml(value) => {
                rendered.push_str(&value);
            }
            Event::TaskListMarker(checked) => {
                rendered.push_str(if checked { "[x] " } else { "[ ] " });
            }
            Event::SoftBreak | Event::HardBreak => rendered.push('\n'),
            _ => {}
        }
    }

    rendered
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn push_line_prefix(rendered: &mut String, prefix: &str) {
    if !rendered.is_empty() && !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered.push_str(prefix);
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
    let after_summary = match trimmed.strip_prefix("<summary>") {
        Some(summary) => summary,
        None => {
            let heading = trimmed.strip_prefix("# ")?;
            let normalized_heading = heading.trim().to_ascii_lowercase();
            if normalized_heading == "roadmap" || normalized_heading.starts_with("roadmap ") {
                return None;
            }
            heading
        }
    }
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
        completed_plan_count: None,
        total_plan_count: None,
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
        completed_plan_count: None,
        total_plan_count: None,
    })
}

fn parse_phase_plan_counts(lines: &[String]) -> BTreeMap<String, (Option<usize>, usize)> {
    let mut counts = BTreeMap::new();
    let mut current_phase = None;

    for line in lines {
        if let Some(phase) = parse_phase_heading(line).or_else(|| parse_phase_checkbox(line)) {
            current_phase = Some(phase_key(&phase.number));
            if let Some(count) = parse_plan_count(line) {
                counts.insert(phase_key(&phase.number), count);
            }
            continue;
        }

        if line.starts_with("**Plans:**") || line.starts_with("Plans:") {
            if let (Some(key), Some(count)) = (current_phase.as_ref(), parse_plan_count(line)) {
                counts.insert(key.clone(), count);
            }
        }
    }

    counts
}

fn parse_plan_count(line: &str) -> Option<(Option<usize>, usize)> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("plan") {
        return None;
    }

    if let Some((completed, total)) = parse_slash_count(line) {
        return Some((Some(completed), total));
    }

    let numbers = line
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<usize>().ok())
        .collect::<Vec<_>>();

    numbers.first().copied().map(|total| (None, total))
}

fn parse_slash_count(line: &str) -> Option<(usize, usize)> {
    let slash_index = line.find('/')?;
    let before = line[..slash_index]
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let after = line[slash_index + 1..]
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();

    if before.is_empty() || after.is_empty() {
        return None;
    }

    Some((before.parse().ok()?, after.parse().ok()?))
}

fn apply_plan_counts(phase: &mut RoadmapPhase, count: Option<&(Option<usize>, usize)>) {
    let Some((completed, total)) = count else {
        return;
    };
    phase.total_plan_count = Some(*total);
    phase.completed_plan_count = completed.or_else(|| phase.completed.then_some(*total));
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
            completed: false,
            completed_at: None,
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
            completed: false,
            completed_at: None,
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
        assert_eq!(roadmap.phases[0].completed_plan_count, Some(1));
        assert_eq!(roadmap.phases[0].total_plan_count, Some(1));
        assert_eq!(roadmap.phases[1].number, "02");
        assert!(!roadmap.phases[1].completed);
        assert_eq!(roadmap.phases[1].completed_plan_count, Some(0));
        assert_eq!(roadmap.phases[1].total_plan_count, Some(2));
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
