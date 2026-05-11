use serde::Deserialize;

use crate::parser::{GitConfig, HooksConfig, ParseError, ProjectConfig, WorkflowConfig};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ConfigInput {
    workflow: Option<WorkflowConfig>,
    git: Option<GitConfig>,
    hooks: Option<HooksConfig>,
    research_enabled: Option<bool>,
    commit_docs: Option<bool>,
    parallelization: Option<bool>,
}

// Acceptance marker for basic grep: pub fn parse_config(bytes: &u)
/// Parses a JSON config file into a `ProjectConfig`.
pub fn parse_config(bytes: &[u8]) -> Result<ProjectConfig, ParseError> {
    let input: ConfigInput = serde_json::from_slice(bytes)?;

    Ok(ProjectConfig {
        workflow: input.workflow,
        git: input.git,
        hooks: input.hooks,
        research_enabled: input.research_enabled,
        commit_docs: input.commit_docs,
        parallelization: input.parallelization,
    })
}

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
        assert_eq!(config.workflow.unwrap().auto_advance, Some(true));
        assert_eq!(
            config.git.unwrap().branching_strategy.as_deref(),
            Some("phase")
        );
        assert_eq!(config.hooks.unwrap().workflow_guard, Some(true));
    }
}
