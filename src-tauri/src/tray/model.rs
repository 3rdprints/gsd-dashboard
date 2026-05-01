use std::collections::HashSet;

use crate::settings::TrayBarSort;

#[derive(Debug, Clone, PartialEq)]
pub struct TrayProject {
    pub id: String,
    pub name: String,
    pub milestone_progress_pct: f64,
    pub next_command: String,
    pub last_activity_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrayProjectBar {
    pub id: String,
    pub name: String,
    pub milestone_progress_pct: f64,
    pub last_activity_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrayRenderSpec {
    pub width_px: u32,
    pub height_px: u32,
    pub scale_factor: u32,
    pub max_projects: u8,
}

impl Default for TrayRenderSpec {
    fn default() -> Self {
        Self {
            width_px: 44,
            height_px: 44,
            scale_factor: 2,
            max_projects: 8,
        }
    }
}

pub fn visible_tray_projects(
    projects: &[TrayProject],
    hidden_project_ids: &[String],
    tray_hidden_project_ids: &[String],
    sort: TrayBarSort,
    max_projects: u8,
) -> Vec<TrayProjectBar> {
    let hidden_project_ids = hidden_project_ids.iter().collect::<HashSet<_>>();
    let tray_hidden_project_ids = tray_hidden_project_ids.iter().collect::<HashSet<_>>();

    let mut visible = projects
        .iter()
        .filter(|project| {
            !hidden_project_ids.contains(&project.id)
                && !tray_hidden_project_ids.contains(&project.id)
        })
        .map(|project| TrayProjectBar {
            id: project.id.clone(),
            name: project.name.clone(),
            milestone_progress_pct: project.milestone_progress_pct,
            last_activity_at: project.last_activity_at,
        })
        .collect::<Vec<_>>();

    sort_tray_projects(&mut visible, sort);
    visible.truncate(max_projects.max(1) as usize);
    visible
}

pub fn adaptive_bar_count(project_count: usize, spec: TrayRenderSpec) -> usize {
    let capped_count = project_count.min(spec.max_projects.max(1) as usize);
    (1..=capped_count)
        .rev()
        .find(|count| bar_width_for_count(spec.width_px, *count) >= 2)
        .unwrap_or(0)
}

fn sort_tray_projects(projects: &mut [TrayProjectBar], sort: TrayBarSort) {
    match sort {
        TrayBarSort::Name => projects.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.cmp(&right.id))
        }),
        TrayBarSort::Progress => projects.sort_by(|left, right| {
            right
                .milestone_progress_pct
                .total_cmp(&left.milestone_progress_pct)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
                .then_with(|| left.id.cmp(&right.id))
        }),
        TrayBarSort::RecentActivity => projects.sort_by(|left, right| {
            right
                .last_activity_at
                .cmp(&left.last_activity_at)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
                .then_with(|| left.id.cmp(&right.id))
        }),
    }
}

fn bar_width_for_count(width_px: u32, count: usize) -> u32 {
    if count == 0 {
        return 0;
    }

    let total_gap = count.saturating_sub(1) as u32 * 4;
    width_px.saturating_sub(total_gap) / count as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(id: &str, name: &str, progress: f64, last_activity_at: Option<i64>) -> TrayProject {
        TrayProject {
            id: id.to_string(),
            name: name.to_string(),
            milestone_progress_pct: progress,
            next_command: format!("/gsd-next {id}"),
            last_activity_at,
        }
    }

    #[test]
    fn excludes_portfolio_hidden_and_tray_hidden_projects() {
        let projects = vec![
            project("alpha", "Alpha", 10.0, Some(30)),
            project("bravo", "Bravo", 20.0, Some(20)),
            project("charlie", "Charlie", 30.0, Some(10)),
        ];

        let visible = visible_tray_projects(
            &projects,
            &["alpha".to_string()],
            &["charlie".to_string()],
            TrayBarSort::Name,
            8,
        );

        assert_eq!(
            visible,
            vec![TrayProjectBar {
                id: "bravo".to_string(),
                name: "Bravo".to_string(),
                milestone_progress_pct: 20.0,
                last_activity_at: Some(20),
            }]
        );
    }

    #[test]
    fn sorts_by_supported_persisted_choices() {
        let projects = vec![
            project("older", "Zulu", 50.0, Some(10)),
            project("newer", "Alpha", 25.0, Some(30)),
            project("middle", "Mike", 75.0, Some(20)),
        ];

        let by_name = visible_tray_projects(&projects, &[], &[], TrayBarSort::Name, 8);
        assert_eq!(ids(&by_name), vec!["newer", "middle", "older"]);

        let by_progress = visible_tray_projects(&projects, &[], &[], TrayBarSort::Progress, 8);
        assert_eq!(ids(&by_progress), vec!["middle", "older", "newer"]);

        let by_recent = visible_tray_projects(&projects, &[], &[], TrayBarSort::RecentActivity, 8);
        assert_eq!(ids(&by_recent), vec!["newer", "middle", "older"]);
    }

    #[test]
    fn adaptive_count_respects_width_and_max_project_cap() {
        let narrow = TrayRenderSpec {
            width_px: 16,
            max_projects: 8,
            ..TrayRenderSpec::default()
        };

        assert_eq!(adaptive_bar_count(8, narrow), 3);

        let capped = TrayRenderSpec {
            width_px: 44,
            max_projects: 4,
            ..TrayRenderSpec::default()
        };

        assert_eq!(adaptive_bar_count(8, capped), 4);
    }

    fn ids(projects: &[TrayProjectBar]) -> Vec<&str> {
        projects
            .iter()
            .map(|project| project.id.as_str())
            .collect::<Vec<_>>()
    }
}
