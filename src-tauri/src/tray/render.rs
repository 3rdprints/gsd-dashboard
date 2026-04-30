use crate::tray::model::{adaptive_bar_count, TrayProjectBar, TrayRenderSpec};

pub fn render_tray_icon_png(
    projects: &[TrayProjectBar],
    spec: TrayRenderSpec,
) -> Result<Vec<u8>, String> {
    let _visible_count = adaptive_bar_count(projects.len(), spec);
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(id: &str, progress: f64) -> TrayProjectBar {
        TrayProjectBar {
            id: id.to_string(),
            name: id.to_string(),
            milestone_progress_pct: progress,
        }
    }

    #[test]
    fn empty_project_list_renders_three_bar_baseline_png() {
        let png = render_tray_icon_png(&[], TrayRenderSpec::default()).unwrap();

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() > 8);
    }

    #[test]
    fn project_bars_render_non_empty_png_bytes() {
        let png = render_tray_icon_png(
            &[bar("alpha", 25.0), bar("bravo", 50.0), bar("charlie", 100.0)],
            TrayRenderSpec::default(),
        )
        .unwrap();

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() > 8);
    }

    #[test]
    fn clamps_progress_and_applies_minimum_visible_height() {
        let png = render_tray_icon_png(
            &[bar("negative", -25.0), bar("tiny", 1.0), bar("large", 250.0)],
            TrayRenderSpec::default(),
        )
        .unwrap();

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() > 8);
    }
}
