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

- [ ] **Phase 1: Foundation**
- [ ] **Phase 2: Planning Parser & Scanner**
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
