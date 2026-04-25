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
            phase_plans: vec![PhasePlan {
                phase: PhaseIdentity {
                    number: "06.1".to_string(),
                    name: "Inserted follow-up".to_string(),
                },
                plan: "01".to_string(),
                plan_type: "execute".to_string(),
                checklist: vec![PlanChecklistItem {
                    label: "Parser contracts compile".to_string(),
                    completed: true,
                }],
            }],
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
