#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_task_blocks() {
        let plan = parse_plan(
            br#"---
phase: 02-planning-parser-scanner
plan: 01
type: execute
---

<tasks>
<task type="auto">
  <name>Task 1: Add parser contracts</name>
  <done>Parser contracts exist.</done>
</task>
</tasks>

- [x] Completed checklist item
- [ ] Open checklist item
"#,
        )
        .unwrap();

        assert_eq!(plan.phase.as_deref(), Some("02-planning-parser-scanner"));
        assert_eq!(plan.plan.as_deref(), Some("01"));
        assert_eq!(plan.plan_type.as_deref(), Some("execute"));
        assert_eq!(plan.tasks[0].name, "Task 1: Add parser contracts");
        assert_eq!(plan.tasks[0].done.as_deref(), Some("Parser contracts exist."));
        assert!(plan.tasks[0].completed);
        assert_eq!(plan.checklist.len(), 2);
        assert!(plan.checklist[0].completed);
        assert!(!plan.checklist[1].completed);
    }
}
