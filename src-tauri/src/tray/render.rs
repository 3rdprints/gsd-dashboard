use crate::tray::model::{adaptive_bar_count, TrayProjectBar, TrayRenderSpec};

const TOP_PADDING_PX: u32 = 4;
const BOTTOM_PADDING_PX: u32 = 4;
const GAP_PX: u32 = 4;
const MIN_VISIBLE_HEIGHT_PX: u32 = 2;

/// Renders project progress bars as a PNG tray icon.
pub fn render_tray_icon_png(
    projects: &[TrayProjectBar],
    spec: TrayRenderSpec,
) -> Result<Vec<u8>, String> {
    let width = spec.width_px.max(1);
    let height = if spec.is_macos_template {
        if spec.height_px != 44 {
            return Err("macOS template tray icons must be rendered at 44px height".to_string());
        }
        44
    } else {
        spec.height_px.max(1)
    };
    let mut pixmap =
        tiny_skia::Pixmap::new(width, height).ok_or_else(|| "invalid tray size".to_string())?;
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(0, 0, 0, 255);

    let bars = if projects.is_empty() {
        empty_baseline_bars()
    } else {
        projects
            .iter()
            .take(adaptive_bar_count(projects.len(), spec))
            .cloned()
            .collect()
    };

    draw_bars(&mut pixmap, &bars, &paint);
    pixmap.encode_png().map_err(|error| error.to_string())
}

fn empty_baseline_bars() -> Vec<TrayProjectBar> {
    (0..3)
        .map(|index| TrayProjectBar {
            id: format!("empty-{index}"),
            name: "empty".to_string(),
            milestone_progress_pct: 0.0,
            last_activity_at: None,
        })
        .collect()
}

fn draw_bars(pixmap: &mut tiny_skia::Pixmap, bars: &[TrayProjectBar], paint: &tiny_skia::Paint) {
    if bars.is_empty() {
        return;
    }

    let count = bars.len() as u32;
    let total_gap = count.saturating_sub(1).saturating_mul(GAP_PX);
    let bar_width = pixmap.width().saturating_sub(total_gap) / count;
    if bar_width < 2 {
        return;
    }

    let drawable_height = pixmap
        .height()
        .saturating_sub(TOP_PADDING_PX + BOTTOM_PADDING_PX);
    if drawable_height == 0 {
        return;
    }

    for (index, bar) in bars.iter().enumerate() {
        let bar_height = progress_height(bar.milestone_progress_pct, drawable_height);
        if bar_height == 0 {
            continue;
        }

        let x = index as u32 * (bar_width + GAP_PX);
        let y = pixmap
            .height()
            .saturating_sub(BOTTOM_PADDING_PX + bar_height);
        if let Some(rect) =
            tiny_skia::Rect::from_xywh(x as f32, y as f32, bar_width as f32, bar_height as f32)
        {
            pixmap.fill_rect(rect, paint, tiny_skia::Transform::identity(), None);
        }
    }
}

fn progress_height(progress: f64, drawable_height: u32) -> u32 {
    if progress <= 0.0 {
        return 1.min(drawable_height);
    }

    let clamped_progress = progress.clamp(0.0, 100.0);
    let scaled_height = ((clamped_progress / 100.0) * drawable_height as f64).round() as u32;
    scaled_height
        .max(MIN_VISIBLE_HEIGHT_PX.min(drawable_height))
        .min(drawable_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(id: &str, progress: f64) -> TrayProjectBar {
        TrayProjectBar {
            id: id.to_string(),
            name: id.to_string(),
            milestone_progress_pct: progress,
            last_activity_at: None,
        }
    }

    #[test]
    fn empty_project_list_renders_three_bar_baseline_png() {
        let png = render_tray_icon_png(&[], TrayRenderSpec::default()).unwrap();

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() > 8);
        assert_eq!(opaque_column_groups(&png), 3);
        assert_black_alpha_pixels(&png);
    }

    #[test]
    fn project_bars_render_non_empty_png_bytes() {
        let png = render_tray_icon_png(
            &[
                bar("alpha", 25.0),
                bar("bravo", 50.0),
                bar("charlie", 100.0),
            ],
            TrayRenderSpec::default(),
        )
        .unwrap();

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() > 8);
        assert_black_alpha_pixels(&png);
    }

    #[test]
    fn clamps_progress_and_applies_minimum_visible_height() {
        let png = render_tray_icon_png(
            &[
                bar("negative", -25.0),
                bar("tiny", 1.0),
                bar("large", 250.0),
            ],
            TrayRenderSpec::default(),
        )
        .unwrap();

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() > 8);
        let pixmap = tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert!(opaque_pixels(&pixmap) > 0);
        assert_black_alpha_pixels(&png);
    }

    #[test]
    fn rejects_wrong_sized_macos_template_specs() {
        let result = render_tray_icon_png(
            &[bar("alpha", 50.0)],
            TrayRenderSpec {
                height_px: 22,
                is_macos_template: true,
                ..TrayRenderSpec::default()
            },
        );

        assert!(result.is_err());
    }

    fn assert_black_alpha_pixels(png: &[u8]) {
        let pixmap = tiny_skia::Pixmap::decode_png(png).unwrap();
        for pixel in pixmap.pixels() {
            if pixel.alpha() > 0 {
                assert_eq!(pixel.red(), 0);
                assert_eq!(pixel.green(), 0);
                assert_eq!(pixel.blue(), 0);
            }
        }
    }

    fn opaque_pixels(pixmap: &tiny_skia::Pixmap) -> usize {
        pixmap
            .pixels()
            .iter()
            .filter(|pixel| pixel.alpha() > 0)
            .count()
    }

    fn opaque_column_groups(png: &[u8]) -> usize {
        let pixmap = tiny_skia::Pixmap::decode_png(png).unwrap();
        let mut groups = 0;
        let mut was_opaque = false;

        for x in 0..pixmap.width() {
            let is_opaque = (0..pixmap.height()).any(|y| {
                pixmap
                    .pixel(x, y)
                    .map(|pixel| pixel.alpha() > 0)
                    .unwrap_or(false)
            });
            if is_opaque && !was_opaque {
                groups += 1;
            }
            was_opaque = is_opaque;
        }

        groups
    }
}
