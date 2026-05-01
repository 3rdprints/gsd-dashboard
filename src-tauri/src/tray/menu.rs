use super::model::TrayProjectBar;

pub const SHOW_DASHBOARD_ID: &str = "show_dashboard";
pub const PREFERENCES_ID: &str = "preferences";
pub const QUIT_ID: &str = "quit";
pub const PROJECT_ID_PREFIX: &str = "project:";
pub const COPY_NEXT_ID_PREFIX: &str = "copy_next:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayMenuAction {
    ShowDashboard,
    Preferences,
    Quit,
    OpenProject { project_id: String },
    CopyNextCommand { project_id: String },
}

impl TrayMenuAction {
    pub fn navigation_route(&self) -> Option<String> {
        match self {
            Self::ShowDashboard => Some("/".to_string()),
            Self::Preferences => Some("/settings".to_string()),
            Self::OpenProject { project_id } => Some(format!("/project/{project_id}")),
            Self::Quit | Self::CopyNextCommand { .. } => None,
        }
    }
}

pub fn format_tooltip(projects: &[TrayProjectBar]) -> String {
    let mut parts = vec![format!("{} active projects", projects.len())];
    parts.extend(projects.iter().take(3).map(|project| {
        format!(
            "{} {}%",
            project.name,
            rounded_pct(project.milestone_progress_pct)
        )
    }));
    parts.join(" · ")
}

pub fn project_menu_label(project: &TrayProjectBar) -> String {
    format!(
        "{} · {}%",
        project.name,
        rounded_pct(project.milestone_progress_pct)
    )
}

pub fn parse_menu_action(id: &str) -> Option<TrayMenuAction> {
    match id {
        SHOW_DASHBOARD_ID => Some(TrayMenuAction::ShowDashboard),
        PREFERENCES_ID => Some(TrayMenuAction::Preferences),
        QUIT_ID => Some(TrayMenuAction::Quit),
        _ => {
            if let Some(project_id) = parse_scoped_project_id(id, PROJECT_ID_PREFIX) {
                return Some(TrayMenuAction::OpenProject { project_id });
            }
            if let Some(project_id) = parse_scoped_project_id(id, COPY_NEXT_ID_PREFIX) {
                return Some(TrayMenuAction::CopyNextCommand { project_id });
            }
            None
        }
    }
}

fn parse_scoped_project_id(id: &str, prefix: &str) -> Option<String> {
    let project_id = id.strip_prefix(prefix)?;
    if project_id.is_empty() || project_id.contains(':') {
        return None;
    }
    Some(project_id.to_string())
}

fn rounded_pct(percent: f64) -> i64 {
    percent.clamp(0.0, 100.0).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(id: &str, name: &str, progress: f64) -> TrayProjectBar {
        TrayProjectBar {
            id: id.to_string(),
            name: name.to_string(),
            milestone_progress_pct: progress,
            last_activity_at: None,
        }
    }

    #[test]
    fn tooltip_summarizes_active_count_and_top_three_projects() {
        let projects = vec![
            project("alpha", "Alpha", 10.4),
            project("bravo", "Bravo", 50.5),
            project("charlie", "Charlie", 99.6),
            project("delta", "Delta", 25.0),
        ];

        assert_eq!(
            format_tooltip(&projects),
            "4 active projects · Alpha 10% · Bravo 51% · Charlie 100%"
        );
        assert!(!format_tooltip(&projects).contains("Delta"));
    }

    #[test]
    fn project_menu_label_uses_name_and_whole_percentage() {
        assert_eq!(
            project_menu_label(&project("alpha", "Alpha", 72.6)),
            "Alpha · 73%"
        );
    }

    #[test]
    fn parser_accepts_only_fixed_and_project_scoped_ids() {
        assert_eq!(
            parse_menu_action(SHOW_DASHBOARD_ID),
            Some(TrayMenuAction::ShowDashboard)
        );
        assert_eq!(
            parse_menu_action(PREFERENCES_ID),
            Some(TrayMenuAction::Preferences)
        );
        assert_eq!(parse_menu_action(QUIT_ID), Some(TrayMenuAction::Quit));
        assert_eq!(
            parse_menu_action("project:alpha"),
            Some(TrayMenuAction::OpenProject {
                project_id: "alpha".to_string()
            })
        );
        assert_eq!(
            parse_menu_action("copy_next:alpha"),
            Some(TrayMenuAction::CopyNextCommand {
                project_id: "alpha".to_string()
            })
        );

        assert_eq!(parse_menu_action("project:"), None);
        assert_eq!(parse_menu_action("copy_next:"), None);
        assert_eq!(parse_menu_action("project:alpha:extra"), None);
        assert_eq!(parse_menu_action("copy_next:alpha:extra"), None);
        assert_eq!(parse_menu_action("open_dashboard"), None);
    }

    #[test]
    fn navigation_routes_are_separate_from_copy_actions() {
        assert_eq!(
            TrayMenuAction::ShowDashboard.navigation_route(),
            Some("/".to_string())
        );
        assert_eq!(
            TrayMenuAction::Preferences.navigation_route(),
            Some("/settings".to_string())
        );
        assert_eq!(
            TrayMenuAction::OpenProject {
                project_id: "alpha".to_string()
            }
            .navigation_route(),
            Some("/project/alpha".to_string())
        );
        assert_eq!(
            TrayMenuAction::CopyNextCommand {
                project_id: "alpha".to_string()
            }
            .navigation_route(),
            None
        );
    }
}
