use std::collections::{HashMap, HashSet};
use std::time::Duration;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::{
    app_state::AppState,
    error::AppError,
    events::AppEvent,
    settings,
    store::project_repo::{self, StoredProjectSnapshot},
    tray::{
        menu::{
            format_tooltip, parse_menu_action, portfolio_summary_label, project_detail_label,
            project_menu_label, TrayMenuAction, COPY_NEXT_ID_PREFIX, PREFERENCES_ID,
            PROJECT_ID_PREFIX, QUIT_ID, SHOW_DASHBOARD_ID,
        },
        model::{
            adaptive_bar_count, tray_render_spec_for_projects, visible_tray_projects,
            TrayPortfolioSummary, TrayProject, TrayProjectBar, TrayRenderSpec,
        },
        render::render_tray_icon_png,
    },
};

pub const TRAY_ID: &str = "main-tray";
const MAIN_WINDOW_LABEL: &str = "main";
pub const TRAY_REFRESH_DEBOUNCE_MS: u64 = 250;

#[derive(Debug, Clone, PartialEq)]
pub struct TrayServiceState {
    pub projects: Vec<TrayProjectBar>,
    pub commands_by_project_id: HashMap<String, String>,
    pub summary: TrayPortfolioSummary,
    pub tooltip: String,
    pub icon_png: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTrayUpdate {
    pub icon_png: Vec<u8>,
    pub tooltip: String,
    pub icon_as_template: bool,
}

pub fn native_tray_update(tray_state: &TrayServiceState) -> NativeTrayUpdate {
    NativeTrayUpdate {
        icon_png: tray_state.icon_png.clone(),
        tooltip: tray_state.tooltip.clone(),
        icon_as_template: macos_template_icon_enabled(),
    }
}

pub fn startup_tray_update() -> Result<NativeTrayUpdate, AppError> {
    Ok(NativeTrayUpdate {
        icon_png: render_tray_icon_png(&[], TrayRenderSpec::default()).map_err(AppError::store)?,
        tooltip: "0 active projects".to_string(),
        icon_as_template: macos_template_icon_enabled(),
    })
}

fn macos_template_icon_enabled() -> bool {
    cfg!(target_os = "macos")
}

pub async fn build_tray_state_for_app(state: &AppState) -> Result<TrayServiceState, AppError> {
    let app_settings = settings::load_or_initialize(&state.pool, &state.home_dir).await?;
    let connection = state.pool.get().await.map_err(AppError::store)?;
    let snapshots = connection
        .interact(project_repo::list_project_snapshots)
        .await
        .map_err(AppError::store)??;

    build_tray_state_from_parts(
        snapshots,
        &app_settings.hidden_project_ids,
        &app_settings.tray_hidden_project_ids,
        app_settings.tray_bar_sort,
        app_settings.tray_bar_max_projects,
    )
}

pub fn build_tray_state_from_parts(
    snapshots: Vec<StoredProjectSnapshot>,
    hidden_project_ids: &[String],
    tray_hidden_project_ids: &[String],
    sort: settings::TrayBarSort,
    max_projects: u8,
) -> Result<TrayServiceState, AppError> {
    let projects = snapshots
        .into_iter()
        .map(TrayProject::from)
        .collect::<Vec<_>>();
    let mut visible_projects = visible_tray_projects(
        &projects,
        hidden_project_ids,
        tray_hidden_project_ids,
        sort,
        max_projects,
    );
    let render_spec = tray_render_spec_for_projects(visible_projects.len(), max_projects);
    visible_projects.truncate(adaptive_bar_count(visible_projects.len(), render_spec));
    let visible_ids = visible_projects
        .iter()
        .map(|project| project.id.clone())
        .collect::<HashSet<_>>();
    let commands_by_project_id = projects
        .into_iter()
        .filter(|project| visible_ids.contains(&project.id))
        .map(|project| (project.id, project.next_command))
        .collect::<HashMap<_, _>>();
    let summary = tray_summary(&visible_projects);
    let tooltip = format_tooltip(&visible_projects);
    let icon_png = render_tray_icon_png(&visible_projects, render_spec).map_err(AppError::store)?;

    Ok(TrayServiceState {
        projects: visible_projects,
        commands_by_project_id,
        summary,
        tooltip,
        icon_png,
    })
}

fn tray_summary(projects: &[TrayProjectBar]) -> TrayPortfolioSummary {
    let average_progress_pct = if projects.is_empty() {
        0.0
    } else {
        projects
            .iter()
            .map(|project| project.milestone_progress_pct.clamp(0.0, 100.0))
            .sum::<f64>()
            / projects.len() as f64
    };

    TrayPortfolioSummary {
        visible_project_count: projects.len(),
        average_progress_pct,
    }
}

pub fn resolve_menu_action(id: &str, tray_state: &TrayServiceState) -> Option<TrayMenuAction> {
    match parse_menu_action(id)? {
        action @ (TrayMenuAction::ShowDashboard
        | TrayMenuAction::Preferences
        | TrayMenuAction::Quit) => Some(action),
        TrayMenuAction::OpenProject { project_id } => tray_state
            .commands_by_project_id
            .contains_key(&project_id)
            .then_some(TrayMenuAction::OpenProject { project_id }),
        TrayMenuAction::CopyNextCommand { project_id } => tray_state
            .commands_by_project_id
            .contains_key(&project_id)
            .then_some(TrayMenuAction::CopyNextCommand { project_id }),
    }
}

pub fn request_tray_refresh<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(TRAY_REFRESH_DEBOUNCE_MS)).await;
        if let Err(error) = refresh_tray(&app).await {
            eprintln!("tray refresh failed: {error}");
        }
    });
}

pub async fn record_tray_refresh_request(state: &AppState) -> Result<(), AppError> {
    state.request_tray_refresh();
    Ok(())
}

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let update = startup_tray_update()?;
    let icon = Image::from_bytes(&update.icon_png).map_err(AppError::from)?;
    let menu = build_native_menu(app, None)?;
    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(update.tooltip)
        .menu(&menu)
        .on_menu_event(|app, event| {
            dispatch_menu_action(app, event.id().as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_dashboard_window(tray.app_handle());
            }
        });
    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);

    let tray = builder.build(app).map_err(AppError::from)?;

    apply_macos_template_icon(&tray)?;

    request_tray_refresh(app);
    Ok(())
}

async fn refresh_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let Some(state) = app.try_state::<AppState>() else {
        return Ok(());
    };
    let tray_state = build_tray_state_for_app(&state).await?;
    let update = native_tray_update(&tray_state);
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| AppError::store("tray icon not initialized"))?;
    let icon = Image::from_bytes(&update.icon_png).map_err(AppError::from)?;
    tray.set_icon(Some(icon)).map_err(AppError::from)?;
    apply_macos_template_icon(&tray)?;
    tray.set_tooltip(Some(&update.tooltip))
        .map_err(AppError::from)?;
    let menu = build_native_menu(app, Some(&tray_state))?;
    tray.set_menu(Some(menu)).map_err(AppError::from)?;
    Ok(())
}

fn apply_macos_template_icon<R: Runtime>(tray: &TrayIcon<R>) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    tray.set_icon_as_template(true).map_err(AppError::from)?;

    #[cfg(not(target_os = "macos"))]
    let _ = tray;

    Ok(())
}

fn dispatch_menu_action<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let app = app.clone();
    let id = id.to_string();
    tauri::async_runtime::spawn(async move {
        let Some(action) = current_menu_action(&app, &id).await else {
            return;
        };

        match action {
            TrayMenuAction::ShowDashboard
            | TrayMenuAction::Preferences
            | TrayMenuAction::OpenProject { .. } => {
                show_dashboard_window(&app);
                if let Some(route) = action.navigation_route() {
                    let _ = app.emit("trayNavigate", AppEvent::TrayNavigate { route });
                }
            }
            TrayMenuAction::CopyNextCommand { project_id } => {
                let state = app.state::<AppState>();
                if let Ok(tray_state) = build_tray_state_for_app(&state).await {
                    if let Some(command) = tray_state.commands_by_project_id.get(&project_id) {
                        let _ = app.clipboard().write_text(command);
                    }
                }
            }
            TrayMenuAction::Quit => app.exit(0),
        }
    });
}

async fn current_menu_action<R: Runtime>(app: &AppHandle<R>, id: &str) -> Option<TrayMenuAction> {
    let state = app.try_state::<AppState>()?;
    let tray_state = build_tray_state_for_app(&state).await.ok()?;
    resolve_menu_action(id, &tray_state)
}

fn build_native_menu<R: Runtime>(
    app: &AppHandle<R>,
    tray_state: Option<&TrayServiceState>,
) -> Result<Menu<R>, AppError> {
    let show_dashboard =
        MenuItem::with_id(app, SHOW_DASHBOARD_ID, "Show Dashboard", true, None::<&str>)?;
    let preferences = MenuItem::with_id(app, PREFERENCES_ID, "Preferences", true, None::<&str>)?;
    let overview = MenuItem::with_id(
        app,
        "overview",
        tray_state
            .map(|state| portfolio_summary_label(state.summary))
            .unwrap_or_else(|| "0 active projects · avg 0%".to_string()),
        false,
        None::<&str>,
    )?;
    let first_separator = PredefinedMenuItem::separator(app)?;
    let second_separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&show_dashboard, &preferences, &overview, &first_separator],
    )?;

    for project in tray_state
        .map(|state| state.projects.as_slice())
        .unwrap_or_default()
    {
        let detail = MenuItem::with_id(
            app,
            format!("project_detail:{}", project.id),
            project_detail_label(project),
            false,
            None::<&str>,
        )?;
        let copy = MenuItem::with_id(
            app,
            format!("{COPY_NEXT_ID_PREFIX}{}", project.id),
            "Copy Next Command",
            true,
            None::<&str>,
        )?;
        let submenu = Submenu::with_id_and_items(
            app,
            format!("{PROJECT_ID_PREFIX}{}", project.id),
            project_menu_label(project),
            true,
            &[&detail, &copy],
        )?;
        menu.append(&submenu)?;
    }

    menu.append(&second_separator)?;
    menu.append(&quit)?;
    Ok(menu)
}

fn toggle_dashboard_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        show_dashboard_window(app);
    }
}

fn show_dashboard_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, MAIN_WINDOW_LABEL, WebviewUrl::default())
        .title("GSD Dashboard")
        .build();
}

impl From<StoredProjectSnapshot> for TrayProject {
    fn from(snapshot: StoredProjectSnapshot) -> Self {
        Self {
            id: snapshot.id,
            name: snapshot.name,
            milestone_progress_pct: snapshot.milestone_progress_pct,
            next_command: snapshot.next_command,
            last_activity_at: snapshot.last_activity_at,
        }
    }
}
