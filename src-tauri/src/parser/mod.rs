pub mod config;
pub mod plan;
pub mod roadmap;
pub mod state;

use serde::{Deserialize, Serialize};

pub use plan::PlanItem;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub project_id: String,
    pub project_name: String,
    pub root_path: String,
    pub planning_path: String,
    pub current_milestone: Option<MilestoneIdentity>,
    pub current_phase: Option<PhaseIdentity>,
    pub milestone_progress_pct: u8,
    #[serde(default)]
    pub roadmap_phases: Vec<roadmap::RoadmapPhase>,
    pub phase_plans: Vec<PhasePlan>,
    pub state_excerpt: Option<String>,
    pub next_command: String,
    pub config: Option<ProjectConfig>,
    pub parse_issues: Vec<ParseIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneIdentity {
    pub index: usize,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseIdentity {
    pub number: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhasePlan {
    pub phase: PhaseIdentity,
    pub plan: String,
    pub plan_type: String,
    #[serde(default)]
    pub plan_path: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub completed_at: Option<i64>,
    pub checklist: Vec<PlanChecklistItem>,
    #[serde(default)]
    pub items: Vec<PlanItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanChecklistItem {
    pub label: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDocument {
    pub phase: Option<String>,
    pub plan: Option<String>,
    pub plan_type: Option<String>,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub completed_at: Option<i64>,
    pub tasks: Vec<PlanTask>,
    pub checklist: Vec<PlanChecklistItem>,
    #[serde(default)]
    pub items: Vec<PlanItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanTask {
    pub name: String,
    pub done: Option<String>,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSummary {
    pub percent: u8,
    pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    #[serde(default)]
    pub workflow: Option<WorkflowConfig>,
    #[serde(default)]
    pub git: Option<GitConfig>,
    #[serde(default)]
    pub hooks: Option<HooksConfig>,
    #[serde(default)]
    pub research_enabled: Option<bool>,
    #[serde(default)]
    pub commit_docs: Option<bool>,
    #[serde(default)]
    pub parallelization: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowConfig {
    #[serde(default)]
    pub research: Option<bool>,
    #[serde(default)]
    pub plan_check: Option<bool>,
    #[serde(default)]
    pub verifier: Option<bool>,
    #[serde(default, alias = "auto_advance")]
    pub auto_advance: Option<bool>,
    #[serde(default, alias = "use_worktrees")]
    pub use_worktrees: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitConfig {
    #[serde(default, alias = "branching_strategy")]
    pub branching_strategy: Option<String>,
    #[serde(default, alias = "phase_branch_template")]
    pub phase_branch_template: Option<String>,
    #[serde(default, alias = "milestone_branch_template")]
    pub milestone_branch_template: Option<String>,
    #[serde(default, alias = "quick_branch_template")]
    pub quick_branch_template: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HooksConfig {
    #[serde(default, alias = "context_warnings")]
    pub context_warnings: Option<bool>,
    #[serde(default, alias = "workflow_guard")]
    pub workflow_guard: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseIssue {
    pub file_path: String,
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("input is not valid UTF-8: {message}")]
    InvalidUtf8 { message: String },
    #[error("frontmatter could not be parsed: {message}")]
    Frontmatter { message: String },
    #[error("JSON could not be parsed: {message}")]
    Json { message: String },
}

impl ParseError {
    pub fn issue(&self, file_path: impl Into<String>) -> ParseIssue {
        ParseIssue {
            file_path: file_path.into(),
            kind: self.kind().to_string(),
            message: self.to_string(),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::InvalidUtf8 { .. } => "invalidUtf8",
            Self::Frontmatter { .. } => "frontmatter",
            Self::Json { .. } => "json",
        }
    }
}

impl From<std::str::Utf8Error> for ParseError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::InvalidUtf8 {
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json {
            message: error.to_string(),
        }
    }
}

pub fn derive_progress(
    roadmap: &roadmap::RoadmapDocument,
    plans: &[PlanDocument],
) -> ProgressSummary {
    if roadmap.phase_checkbox_total > 0 {
        return ProgressSummary {
            percent: roadmap.milestone_progress_pct,
            source: "roadmapPhaseCheckboxes".to_string(),
        };
    }

    let total_items = plans
        .iter()
        .map(|plan| plan.checklist.len() + plan.tasks.len())
        .sum::<usize>();
    let completed_items = plans
        .iter()
        .map(|plan| {
            plan.checklist.iter().filter(|item| item.completed).count()
                + plan.tasks.iter().filter(|task| task.completed).count()
        })
        .sum::<usize>();

    if total_items > 0 {
        return ProgressSummary {
            percent: percent(completed_items, total_items),
            source: "planChecklistCompletion".to_string(),
        };
    }

    ProgressSummary {
        percent: roadmap.milestone_progress_pct,
        source: roadmap.progress_source.clone(),
    }
}

fn percent(completed: usize, total: usize) -> u8 {
    if total == 0 {
        return 0;
    }

    ((completed * 100) / total).min(100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_snapshot_contract_contains_planning_fields() {
        let snapshot = ProjectSnapshot {
            project_id: "gsd-dashboard".to_string(),
            project_name: "GSD Dashboard".to_string(),
            root_path: "/Users/smacdonald/homegit/gsd-dashboard".to_string(),
            planning_path: "/Users/smacdonald/homegit/gsd-dashboard/.planning".to_string(),
            current_milestone: Some(MilestoneIdentity {
                index: 1,
                name: "v1.0 MVP".to_string(),
            }),
            current_phase: Some(PhaseIdentity {
                number: "06.1".to_string(),
                name: "Inserted follow-up".to_string(),
            }),
            milestone_progress_pct: 42,
            roadmap_phases: Vec::new(),
            phase_plans: vec![PhasePlan {
                phase: PhaseIdentity {
                    number: "06.1".to_string(),
                    name: "Inserted follow-up".to_string(),
                },
                plan: "01".to_string(),
                plan_type: "execute".to_string(),
                plan_path: ".planning/phases/06.1/06.1-01-PLAN.md".to_string(),
                completed: false,
                completed_at: None,
                checklist: vec![PlanChecklistItem {
                    label: "Parser contracts compile".to_string(),
                    completed: true,
                }],
                items: vec![PlanItem {
                    ord: 0,
                    text: "Parser contracts compile".to_string(),
                    checked: true,
                    line_no: 1,
                }],
            }],
            state_excerpt: Some("Phase: 06.1".to_string()),
            next_command: "/gsd-next".to_string(),
            config: Some(ProjectConfig::default()),
            parse_issues: Vec::new(),
        };

        assert_eq!(snapshot.current_phase.unwrap().number, "06.1");
        assert_eq!(snapshot.next_command, "/gsd-next");
    }

    #[test]
    fn parse_error_converts_to_issue_without_panicking() {
        let issue = ParseError::InvalidUtf8 {
            message: "bad utf-8".to_string(),
        }
        .issue(".planning/STATE.md");

        assert_eq!(issue.file_path, ".planning/STATE.md");
        assert_eq!(issue.kind, "invalidUtf8");
        assert!(issue.message.contains("bad utf-8"));
    }
}
