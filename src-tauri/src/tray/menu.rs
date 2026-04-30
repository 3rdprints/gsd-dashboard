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

pub fn format_tooltip(_projects: &[TrayProjectBar]) -> String {
    String::new()
}

pub fn project_menu_label(_project: &TrayProjectBar) -> String {
    String::new()
}

pub fn parse_menu_action(_id: &str) -> Option<TrayMenuAction> {
    None
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
        assert_eq!(project_menu_label(&project("alpha", "Alpha", 72.6)), "Alpha · 73%");
    }

    #[test]
    fn parser_accepts_only_fixed_and_project_scoped_ids() {
        assert_eq!(parse_menu_action(SHOW_DASHBOARD_ID), Some(TrayMenuAction::ShowDashboard));
        assert_eq!(parse_menu_action(PREFERENCES_ID), Some(TrayMenuAction::Preferences));
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
}
