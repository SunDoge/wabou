//! Backend-neutral command-line render options and frontend preparation.

use std::path::{Path, PathBuf};

use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use super::config::{BuildProfile, bundle_path};
use super::project::App;
use super::{Result, ensure, frontend};

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub(super) enum HeadlessColorScheme {
    Light,
    Dark,
}

pub(super) struct RenderOptions {
    pub(super) out: PathBuf,
    pub(super) batch: Option<PathBuf>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) window_id: u64,
    pub(super) scale_factor: f64,
    pub(super) color_scheme: HeadlessColorScheme,
    pub(super) mode: Option<String>,
    pub(super) fixture: Option<String>,
    pub(super) skip_build: bool,
    pub(super) with_host: bool,
    pub(super) scenario: Option<PathBuf>,
    pub(super) wait_ms: u64,
    pub(super) metrics: Option<PathBuf>,
    pub(super) snapshot: Option<PathBuf>,
    pub(super) samples: usize,
    pub(super) actions: Vec<RenderAction>,
    pub(super) layout_only: bool,
    pub(super) cargo_features: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct LayoutBatchManifest {
    pub(super) version: u32,
    #[serde(default)]
    pub(super) all: bool,
    #[serde(default)]
    pub(super) cases: Vec<LayoutBatchCase>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct LayoutBatchCase {
    pub(super) id: String,
    #[serde(default)]
    pub(super) width: Option<u32>,
    #[serde(default)]
    pub(super) height: Option<u32>,
    #[serde(default)]
    pub(super) scale_factor: Option<f64>,
    #[serde(default)]
    pub(super) wait_ms: Option<u64>,
}

impl LayoutBatchCase {
    pub(super) fn inherit(&mut self, fixture: &Self) {
        self.width = self.width.or(fixture.width);
        self.height = self.height.or(fixture.height);
        self.scale_factor = self.scale_factor.or(fixture.scale_factor);
        self.wait_ms = self.wait_ms.or(fixture.wait_ms);
    }

    pub(super) fn width(&self) -> u32 {
        self.width.unwrap_or(1_440)
    }

    pub(super) fn height(&self) -> u32 {
        self.height.unwrap_or(900)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LayoutBatchReport {
    pub(super) version: u32,
    pub(super) total_duration_ms: f64,
    pub(super) cases: Vec<LayoutBatchResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LayoutBatchResult {
    pub(super) id: String,
    pub(super) duration_ms: f64,
    pub(super) snapshot: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum RenderAction {
    Click([f64; 2]),
    Wheel([f64; 4]),
    Text(String),
    Key(String),
}

pub(super) fn actions(
    clicks: Vec<f64>,
    wheels: Vec<f64>,
    text: Option<String>,
    keys: Vec<String>,
) -> Vec<RenderAction> {
    let mut actions =
        clicks
            .as_chunks::<2>()
            .0
            .iter()
            .map(|values| RenderAction::Click([values[0], values[1]]))
            .chain(
                wheels.as_chunks::<4>().0.iter().map(|values| {
                    RenderAction::Wheel([values[0], values[1], values[2], values[3]])
                }),
            )
            .collect::<Vec<_>>();
    actions.extend(text.map(RenderAction::Text));
    actions.extend(keys.into_iter().map(RenderAction::Key));
    actions
}

pub(super) fn actions_from_matches(matches: &ArgMatches) -> Option<Vec<RenderAction>> {
    let (name, render) = matches.subcommand()?;
    if name != "render" {
        return None;
    }
    let mut indexed = Vec::<(usize, RenderAction)>::new();
    if let (Some(indices), Some(values)) =
        (render.indices_of("click"), render.get_many::<f64>("click"))
    {
        let positions = indices.collect::<Vec<_>>();
        let values = values.copied().collect::<Vec<_>>();
        for (positions, values) in positions
            .as_chunks::<2>()
            .0
            .iter()
            .zip(values.as_chunks::<2>().0.iter())
        {
            indexed.push((positions[0], RenderAction::Click([values[0], values[1]])));
        }
    }
    if let (Some(indices), Some(values)) =
        (render.indices_of("wheel"), render.get_many::<f64>("wheel"))
    {
        let positions = indices.collect::<Vec<_>>();
        let values = values.copied().collect::<Vec<_>>();
        for (positions, values) in positions
            .as_chunks::<4>()
            .0
            .iter()
            .zip(values.as_chunks::<4>().0.iter())
        {
            indexed.push((
                positions[0],
                RenderAction::Wheel([values[0], values[1], values[2], values[3]]),
            ));
        }
    }
    if let (Some(index), Some(value)) = (
        render
            .indices_of("text")
            .and_then(|mut values| values.next()),
        render.get_one::<String>("text"),
    ) {
        indexed.push((index, RenderAction::Text(value.clone())));
    }
    if let (Some(indices), Some(values)) =
        (render.indices_of("key"), render.get_many::<String>("key"))
    {
        indexed.extend(
            indices
                .zip(values)
                .map(|(index, value)| (index, RenderAction::Key(value.clone()))),
        );
    }
    indexed.sort_by_key(|(index, _)| *index);
    Some(indexed.into_iter().map(|(_, action)| action).collect())
}

pub(super) fn prepare_frontend(
    workspace: &Path,
    app: &App,
    mode: Option<&str>,
    skip_build: bool,
) -> Result<()> {
    let bundle = bundle_path(workspace, app, BuildProfile::Debug)?;
    if skip_build {
        if !bundle.is_file() {
            return Err(format!(
                "--skip-build requires an existing debug bundle at {}; rerun without --skip-build",
                bundle.display()
            )
            .into());
        }
        return Ok(());
    }
    let frontend_lock = frontend::lock(workspace, app)?;
    let mode_args = mode.map(|mode| ["--mode", mode]);
    ensure(
        frontend::build_unlocked(
            workspace,
            app,
            mode_args.as_ref().map_or(&[], |args| args),
            BuildProfile::Debug,
            true,
        )?,
        "Vite build",
    )?;
    drop(frontend_lock);
    Ok(())
}
