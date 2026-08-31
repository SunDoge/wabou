//! GPUI-native implementation of `wabou layout`.

use std::{collections::HashSet, fs, path::Path, sync::Arc, time::Duration};

use serde_json::json;
use wabou_runtime::{
    GpuiHeadlessHarness, GpuiHeadlessOptions, GpuiHeadlessOutput, GpuiLayoutNode,
    ProjectedNodeKind, WindowOptions,
};

use super::{
    HeadlessColorScheme, LayoutBatchCase, LayoutBatchManifest, LayoutBatchReport,
    LayoutBatchResult, RenderOptions, prepare_frontend,
};
use crate::{Result, config::BuildProfile, config::bundle_path, project::App};

pub(super) fn run(workspace: &Path, app: &App, options: &RenderOptions) -> Result<()> {
    prepare_frontend(
        workspace,
        app,
        options
            .fixture
            .as_ref()
            .map(|_| "layout-test")
            .or(options.mode.as_deref()),
        options.skip_build,
    )?;
    let bundle = bundle_path(workspace, app, BuildProfile::Debug)?;
    let source: Arc<str> = fs::read_to_string(&bundle)?.into();
    let source_map = fs::read(bundle.with_extension("js.map"))
        .ok()
        .map(Arc::<[u8]>::from);

    if let Some(manifest) = &options.batch {
        return run_batch(source, source_map, manifest, &options.out, options);
    }

    let mut harness = boot(source, source_map, options.width, options.height)?;
    let fixture = options
        .fixture
        .as_deref()
        .map(|id| mount_fixture(&mut harness, id))
        .transpose()?;
    if let Some(fixture) = fixture {
        if fixture.width.is_some() || fixture.height.is_some() {
            return run_selected_fixture(workspace, app, options, fixture);
        }
    }
    settle_wait(&mut harness, options.wait_ms)?;
    super::replay_actions(&mut harness, &options.actions)?;
    write_snapshot(&options.out, &harness.snapshot()?, options.color_scheme)
}

fn run_selected_fixture(
    workspace: &Path,
    app: &App,
    options: &RenderOptions,
    fixture: LayoutBatchCase,
) -> Result<()> {
    let bundle = bundle_path(workspace, app, BuildProfile::Debug)?;
    let source: Arc<str> = fs::read_to_string(&bundle)?.into();
    let source_map = fs::read(bundle.with_extension("js.map"))
        .ok()
        .map(Arc::<[u8]>::from);
    let mut harness = boot(source, source_map, fixture.width(), fixture.height())?;
    mount_fixture(&mut harness, &fixture.id)?;
    settle_wait(&mut harness, fixture.wait_ms.unwrap_or(options.wait_ms))?;
    super::replay_actions(&mut harness, &options.actions)?;
    write_snapshot(&options.out, &harness.snapshot()?, options.color_scheme)
}

fn run_batch(
    source: Arc<str>,
    source_map: Option<Arc<[u8]>>,
    path: &Path,
    out: &Path,
    options: &RenderOptions,
) -> Result<()> {
    let mut manifest: LayoutBatchManifest = serde_json::from_slice(&fs::read(path)?)?;
    if manifest.version != 1 {
        return Err(format!(
            "unsupported layout batch manifest version {}",
            manifest.version
        )
        .into());
    }
    let mut registry = boot(
        source.clone(),
        source_map.clone(),
        options.width,
        options.height,
    )?;
    let registered = fixture_cases(&mut registry)?;
    if manifest.all {
        manifest.cases = registered.clone();
    } else {
        for case in &mut manifest.cases {
            if let Some(fixture) = registered.iter().find(|fixture| fixture.id == case.id) {
                case.inherit(fixture);
            }
        }
    }
    if manifest.cases.is_empty() {
        return Err("layout batch manifest must contain at least one case".into());
    }
    let mut ids = HashSet::new();
    let started = std::time::Instant::now();
    let mut results = Vec::with_capacity(manifest.cases.len());
    for case in manifest.cases {
        if !ids.insert(case.id.clone()) {
            return Err(format!("duplicate layout batch case id `{}`", case.id).into());
        }
        let case_started = std::time::Instant::now();
        let mut harness = boot(
            source.clone(),
            source_map.clone(),
            case.width(),
            case.height(),
        )?;
        mount_fixture(&mut harness, &case.id)?;
        settle_wait(&mut harness, case.wait_ms.unwrap_or(options.wait_ms))?;
        super::replay_actions(&mut harness, &options.actions)?;
        let output = harness.snapshot()?;
        results.push(LayoutBatchResult {
            id: case.id,
            duration_ms: case_started.elapsed().as_secs_f64() * 1_000.0,
            snapshot: snapshot_value(&output, options.color_scheme)?,
        });
    }
    if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        out,
        serde_json::to_vec_pretty(&LayoutBatchReport {
            version: 1,
            total_duration_ms: started.elapsed().as_secs_f64() * 1_000.0,
            cases: results,
        })?,
    )?;
    println!(
        "[wabou] wrote {} GPUI layout fixtures to {}",
        ids.len(),
        out.display()
    );
    Ok(())
}

pub(super) fn boot(
    source: Arc<str>,
    source_map: Option<Arc<[u8]>>,
    width: u32,
    height: u32,
) -> Result<GpuiHeadlessHarness> {
    Ok(GpuiHeadlessHarness::boot(
        source,
        source_map,
        GpuiHeadlessOptions {
            window: WindowOptions::new().initial_inner_size(width, height),
            settle_frames: 4,
        },
    )?)
}

fn fixture_cases(harness: &mut GpuiHeadlessHarness) -> Result<Vec<LayoutBatchCase>> {
    let encoded = harness.eval_string(
        "typeof globalThis.__wabou_layout_fixture_cases === 'function' \
         ? globalThis.__wabou_layout_fixture_cases() : '[]'",
    )?;
    Ok(serde_json::from_str(&encoded)?)
}

pub(super) fn mount_fixture(
    harness: &mut GpuiHeadlessHarness,
    id: &str,
) -> Result<LayoutBatchCase> {
    let cases = fixture_cases(harness)?;
    let fixture = cases
        .into_iter()
        .find(|fixture| fixture.id == id)
        .ok_or_else(|| format!("unknown layout fixture `{id}`"))?;
    let id = serde_json::to_string(id)?;
    harness.eval_script(&format!("globalThis.__wabou_layout_fixture_mount({id});"))?;
    harness.settle(2)?;
    Ok(fixture)
}

pub(super) fn settle_wait(harness: &mut GpuiHeadlessHarness, wait_ms: u64) -> Result<()> {
    if wait_ms > 0 {
        harness.advance_time(Duration::from_millis(wait_ms))?;
    }
    harness.settle(2)?;
    Ok(())
}

pub(super) fn write_snapshot(
    path: &Path,
    output: &GpuiHeadlessOutput,
    color_scheme: HeadlessColorScheme,
) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(&snapshot_value(output, color_scheme)?)?,
    )?;
    println!("[wabou] wrote GPUI layout snapshot {}", path.display());
    Ok(())
}

fn snapshot_value(
    output: &GpuiHeadlessOutput,
    _color_scheme: HeadlessColorScheme,
) -> Result<serde_json::Value> {
    let nodes = output.layout.iter().map(node_value).collect::<Vec<_>>();
    Ok(json!({
        "status": {
            "viewportWidth": output.viewport_width,
            "viewportHeight": output.viewport_height,
            "deviceScale": output.scale_factor,
            "nodeCount": nodes.len(),
        },
        "nodes": nodes,
    }))
}

fn node_value(node: &GpuiLayoutNode) -> serde_json::Value {
    let tag = match &node.kind {
        ProjectedNodeKind::Root => "root",
        ProjectedNodeKind::Element(tag) => tag.as_ref(),
        ProjectedNodeKind::Text => "text",
    };
    let rect = json!({
        "x": f32::from(node.bounds.origin.x),
        "y": f32::from(node.bounds.origin.y),
        "width": f32::from(node.bounds.size.width),
        "height": f32::from(node.bounds.size.height),
    });
    let content_rect = json!({
        "x": f32::from(node.content_bounds.origin.x),
        "y": f32::from(node.content_bounds.origin.y),
        "width": f32::from(node.content_bounds.size.width),
        "height": f32::from(node.content_bounds.size.height),
    });
    let attrs = node
        .attributes
        .iter()
        .map(|(name, value)| [name.as_ref(), value.as_ref()])
        .collect::<Vec<_>>();
    let semantic = node.attributes.get("role").map(|role| {
        json!({
            "role": role.as_ref(),
            "label": node.attributes.get("aria-label").map(AsRef::<str>::as_ref),
        })
    });
    json!({
        "id": node.key,
        "parentId": node.parent,
        "tag": tag,
        "text": node.text.as_deref(),
        "textMetrics": node.text_metrics.as_ref().map(|metrics| json!({
            "source": metrics.source,
            "lineBox": {
                "x": f32::from(metrics.line_box.origin.x),
                "y": f32::from(metrics.line_box.origin.y),
                "width": f32::from(metrics.line_box.size.width),
                "height": f32::from(metrics.line_box.size.height),
            },
            "baseline": metrics.baseline,
        })),
        "classes": node.classes,
        "attrs": attrs,
        "rect": rect,
        "contentRect": content_rect,
        "styleDiagnostics": node.style_diagnostics,
        "semantic": semantic,
        "computed": {
            "position": node.computed.position,
            "overflowX": node.computed.overflow_x,
            "overflowY": node.computed.overflow_y,
            "overlayPlane": "Content",
            "fontSize": node.computed.font_size,
            "fontWeight": node.computed.font_weight,
            "textColor": node.computed.text_color.map(|color| {
                let rgba = wabou_shell::gpui::hsla_to_rgba(color);
                format!(
                    "rgba({:.0}, {:.0}, {:.0}, {:.3})",
                    rgba.color.red * 255.0,
                    rgba.color.green * 255.0,
                    rgba.color.blue * 255.0,
                    rgba.alpha
                )
            }),
            "background": null,
            "opacity": node.computed.opacity,
        },
    })
}
