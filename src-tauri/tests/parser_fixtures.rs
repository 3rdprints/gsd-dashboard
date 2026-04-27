use std::{
    fs,
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
};

use gsd_dashboard::parser::{config, plan, roadmap, state};

const FIXTURE_NAMES: [&str; 5] = [
    "deckpilot-web",
    "listingguru",
    "locdirectory",
    "getrovix",
    "youtubeauto",
];

#[derive(Debug)]
struct FixtureParseResult {
    project_name: &'static str,
    has_current_milestone: bool,
    roadmap_phase_numbers: Vec<String>,
    next_command: String,
    decimal_phase_preserved: bool,
}

#[test]
#[ignore]
fn real_fixture_planning_docs_parse() {
    let results = FIXTURE_NAMES
        .into_iter()
        .map(parse_fixture_without_panicking)
        .collect::<Vec<_>>();

    for result in &results {
        assert!(
            result.has_current_milestone || !result.roadmap_phase_numbers.is_empty(),
            "{} should expose a milestone or roadmap phases",
            result.project_name
        );
        assert!(
            !result.next_command.is_empty(),
            "{} should expose a next command or /gsd-next fallback",
            result.project_name
        );
        assert!(
            result
                .roadmap_phase_numbers
                .iter()
                .any(|number| number.chars().any(|character| character.is_ascii_digit())),
            "{} should preserve phase numbers as String values",
            result.project_name
        );
    }

    assert!(
        results.iter().any(|result| {
            matches!(result.project_name, "deckpilot-web" | "youtubeauto")
                && result.decimal_phase_preserved
        }),
        "deckpilot-web or youtubeauto should preserve at least one decimal phase"
    );
}

fn parse_fixture_without_panicking(project_name: &'static str) -> FixtureParseResult {
    let outcome = catch_unwind(AssertUnwindSafe(|| parse_fixture(project_name)));
    match outcome {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => panic!("{project_name} fixture failed to parse: {error}"),
        Err(payload) => panic!("{project_name} fixture panicked: {payload:?}"),
    }
}

fn parse_fixture(project_name: &'static str) -> Result<FixtureParseResult, String> {
    let planning_path = fixture_base_path().join(project_name).join(".planning");
    let roadmap_bytes = fs::read(planning_path.join("ROADMAP.md"))
        .map_err(|error| format!("ROADMAP.md missing: {error}"))?;
    let roadmap = roadmap::parse_roadmap(&roadmap_bytes)
        .map_err(|error| format!("ROADMAP.md parse failed: {error}"))?;

    if let Some(milestones_bytes) = read_optional(planning_path.join("MILESTONES.md"))? {
        let _ = roadmap::parse_milestones(&milestones_bytes)
            .map_err(|error| format!("MILESTONES.md parse failed: {error}"))?;
    }

    let state_doc = if let Some(state_bytes) = read_optional(planning_path.join("STATE.md"))? {
        Some(
            state::parse_state(&state_bytes)
                .map_err(|error| format!("STATE.md parse failed: {error}"))?,
        )
    } else {
        None
    };

    if let Some(config_bytes) = read_optional(planning_path.join("config.json"))? {
        let _ = config::parse_config(&config_bytes)
            .map_err(|error| format!("config.json parse failed: {error}"))?;
    }

    for plan_path in collect_plan_paths(&planning_path)? {
        let bytes = fs::read(&plan_path)
            .map_err(|error| format!("{} read failed: {error}", plan_path.display()))?;
        let _ = plan::parse_plan(&bytes)
            .map_err(|error| format!("{} parse failed: {error}", plan_path.display()))?;
    }

    let roadmap_phase_numbers = roadmap
        .phases
        .iter()
        .map(|phase| phase.number.clone())
        .collect::<Vec<_>>();

    Ok(FixtureParseResult {
        project_name,
        has_current_milestone: state_doc
            .as_ref()
            .and_then(|state| state.current_milestone.as_ref())
            .is_some()
            || !roadmap.milestones.is_empty(),
        roadmap_phase_numbers,
        next_command: state_doc
            .map(|state| state.next_command)
            .unwrap_or_else(|| "/gsd-next".to_string()),
        decimal_phase_preserved: roadmap
            .phases
            .iter()
            .any(|phase| phase.number.contains('.')),
    })
}

fn fixture_base_path() -> PathBuf {
    std::env::var_os("GSD_DASHBOARD_FIXTURE_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        })
}

fn read_optional(path: PathBuf) -> Result<Option<Vec<u8>>, String> {
    match fs::read(&path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("{} read failed: {error}", path.display())),
    }
}

fn collect_plan_paths(planning_path: &Path) -> Result<Vec<PathBuf>, String> {
    let phases_path = planning_path.join("phases");
    let mut plan_paths = Vec::new();

    if !phases_path.exists() {
        return Ok(plan_paths);
    }

    collect_plan_paths_recursive(&phases_path, &mut plan_paths)?;
    plan_paths.sort();
    Ok(plan_paths)
}

fn collect_plan_paths_recursive(path: &Path, plan_paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(path).map_err(|error| format!("{} read failed: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| format!("{} entry failed: {error}", path.display()))?;
        let entry_path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("{} type failed: {error}", entry_path.display()))?;

        if file_type.is_dir() {
            collect_plan_paths_recursive(&entry_path, plan_paths)?;
        } else if entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("-PLAN.md"))
        {
            plan_paths.push(entry_path);
        }
    }

    Ok(())
}
