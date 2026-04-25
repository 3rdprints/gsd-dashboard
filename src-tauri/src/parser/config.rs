#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_config_shapes() {
        let config = parse_config(
            br#"{
  "commit_docs": false,
  "parallelization": true,
  "research_enabled": true,
  "workflow": {
    "auto_advance": true,
    "use_worktrees": true,
    "unknown_workflow_key": "ignored"
  },
  "git": {
    "branching_strategy": "phase",
    "phase_branch_template": "gsd/phase-{phase}-{slug}"
  },
  "hooks": {
    "context_warnings": true,
    "workflow_guard": true
  },
  "unknown_top_level": true
}"#,
        )
        .unwrap();

        assert_eq!(config.commit_docs, Some(false));
        assert_eq!(config.parallelization, Some(true));
        assert_eq!(
            config.workflow.unwrap().auto_advance,
            Some(true)
        );
        assert_eq!(
            config.git.unwrap().branching_strategy.as_deref(),
            Some("phase")
        );
        assert_eq!(config.hooks.unwrap().workflow_guard, Some(true));
    }
}
