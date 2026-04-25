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
    fn next_command_defaults_to_gsd_next() {
        let state = parse_state(
            br#"---
milestone: v1.0
---

## Current Position

**Phase:** 06.1
"#,
        )
        .unwrap();

        assert_eq!(state.next_command, "/gsd-next");
    }
}
