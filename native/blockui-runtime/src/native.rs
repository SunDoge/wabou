//! Thin Rust source over Wabou's existing Winit/Taffy/Vello shell and TextInput.
use crate::native_model::NativeModel;
use crate::{Actor, ActorId, Message};
use blockui_abi::*;
use shell::{
    layout::PlacedNode,
    style::{CornerRadii, Paint},
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use taffy::{Overflow, geometry::Point as TaffyPoint, prelude::*};
use vello_common::peniko::Color;
use wabou_shell_vello::widgets::TextInput;
use wabou_shell_vello::{
    self as shell, EventResponse, FrameSource, PointerPhase, TextContext, UiEvent, Widget,
    WidgetGeometry,
};

#[path = "native_watch.rs"]
mod development_watch;
pub(super) use development_watch::BuildResult;
pub use development_watch::ReloadConfig;
use development_watch::{Watch, watch};
#[cfg(test)]
use development_watch::{source_change_summary, source_snapshot};

struct ActorWatch {
    target: ActorId,
    lifecycle: u64,
    watcher: Option<Watch>,
    builds: crate::build_queue::Receiver,
}
impl Drop for ActorWatch {
    fn drop(&mut self) {
        self.watcher.take();
        for result in self.builds.try_iter() {
            if let Some(path) = result.artifact {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

struct ImplementationWatch {
    name: String,
    watcher: Option<Watch>,
    builds: crate::build_queue::Receiver,
}
impl Drop for ImplementationWatch {
    fn drop(&mut self) {
        self.watcher.take();
        for result in self.builds.try_iter() {
            if let Some(path) = result.artifact {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

struct ScopeWatchProof {
    input: u64,
    scroll: u64,
    node: NodeId,
    widget: usize,
    state: usize,
    slot: usize,
    version: u64,
    failed: bool,
}
struct EditorWindowProof {
    replacement: PathBuf,
    file: PathBuf,
    expected: Option<(u64, usize, usize, usize)>,
    loading: bool,
}
pub struct NativeSource<M: NativeModel = Actor> {
    pub actor: M,
    tree: TaffyTree<Paint>,
    ids: BTreeMap<u64, NodeId>,
    kinds: BTreeMap<u64, u32>,
    applied: BTreeMap<u64, (Node, String)>,
    themes: BTreeMap<u64, (u32, u32)>,
    theme_system_dark: bool,
    #[cfg(test)]
    theme_rebuilds: usize,
    painted_grid_cells: BTreeSet<u64>,
    badge_rows: BTreeSet<u64>,
    applied_order: Vec<u64>,
    #[cfg(test)]
    sync_reconciliations: usize,
    #[cfg(test)]
    badge_row_syncs: usize,
    #[cfg(test)]
    icon_syncs: usize,
    paint_dirty: BTreeSet<u64>,
    motion: blockui_motion::Timeline,
    motion_epoch: Instant,
    motion_manual_time: Option<Duration>,
    hover_paints: BTreeMap<u64, (motion::HoverPaint, motion::HoverPaint)>,
    scroll_targets: BTreeMap<u64, [f64; 2]>,
    painted_scroll: BTreeMap<u64, [f64; 2]>,
    observed_viewports: BTreeMap<u64, (NodeId, u64, [f32; 2])>,
    editors: BTreeMap<u64, Box<TextInput>>,
    sliders: BTreeMap<u64, Box<crate::native_slider::NativeSlider>>,
    selects: BTreeMap<u64, crate::select_model::SelectModel>,
    select_popup: Option<(u64, crate::select_popup::PopupLayout, Vec<i64>)>,
    select_rows: Vec<NodeId>,
    select_pressed: Option<i64>,
    select_semantic_ids: BTreeMap<(u64, Option<i64>), u64>,
    select_semantic_next: u64,
    icons: BTreeMap<u64, icons::CachedIcon>,
    canvases: BTreeMap<u64, blockui_abi::canvas::CanvasScene>,
    scroll_children: BTreeMap<u64, NodeId>,
    placed: Vec<PlacedNode>,
    semantics_enabled: bool,
    semantic_revision: u64,
    semantics: Option<Arc<shell::SemanticSnapshot>>,
    focus: Option<u64>,
    focus_epoch: u64,
    pending_reveal: Option<u64>,
    suspended_grid_focus: Option<(u64, i64)>,
    modal: Option<u64>,
    modal_opener: Option<u64>,
    toast_return_focus: Option<u64>,
    dialog_tops: BTreeMap<u64, f32>,
    paste_target: Option<u64>,
    pressed: Option<u64>,
    pointer_pressed_visual: Option<u64>,
    hovered: Option<u64>,
    tooltip: crate::interaction::tooltip::Controller,
    menus: BTreeMap<u64, crate::interaction::menu::Controller>,
    menubar_stops: BTreeMap<u64, u64>,
    context_anchor: Option<(u64, [f64; 2])>,
    menu_open_last: Option<u64>,
    menu_hover: crate::interaction::menu_hover::Controller,
    menu_pointer_active: bool,
    pointer_position: Option<shell::Point>,
    keyboard_pressed: Option<u64>,
    primary_shortcuts: BTreeMap<String, u64>,
    composing_editor: Option<u64>,
    window_focused: bool,
    scale: f64,
    wake: Arc<Mutex<Option<shell::WakeCallback>>>,
    builds: Option<crate::build_queue::Receiver>,
    watcher: Option<Watch>,
    actor_watches: BTreeMap<ActorId, ActorWatch>,
    actor_diagnostics: BTreeMap<ActorId, String>,
    implementation_watches: BTreeMap<u64, ImplementationWatch>,
    watch_expected: Option<(usize, usize, u64, bool)>,
    smoke: Option<PathBuf>,
    graph_smoke: Option<PathBuf>,
    scope_smoke: Option<bool>,
    scope_watch: bool,
    scope_watch_expected: Option<ScopeWatchProof>,
    io_smoke: Option<PathBuf>,
    save_smoke: Option<PathBuf>,
    editor_window: Option<EditorWindowProof>,
    io_expected: Option<(usize, usize, usize)>,
    frames: u64,
    capture: bool,
    capture_wait_focus: bool,
    clipboard_smoke: Option<(PathBuf, bool)>,
    exit: bool,
    pub metrics: Option<shell::WindowMetrics>,
    pub diagnostic: String,
}
impl<M: NativeModel> NativeSource<M> {
    pub fn new(actor: M, config: ReloadConfig) -> Result<Self, String> {
        if !cfg!(feature = "hotreload") {
            return Err(
                "native hot reload is disabled; enable the runtime hotreload feature".into(),
            );
        }
        let mut source = Self::without_watch(actor)?;
        let (watcher, builds) = watch(config, source.wake.clone());
        source.watcher = Some(watcher);
        source.builds = Some(builds);
        Ok(source)
    }
    pub fn without_watch(actor: M) -> Result<Self, String> {
        let mut source = Self {
            actor,
            tree: TaffyTree::new(),
            ids: BTreeMap::new(),
            kinds: BTreeMap::new(),
            applied: BTreeMap::new(),
            themes: BTreeMap::new(),
            theme_system_dark: false,
            #[cfg(test)]
            theme_rebuilds: 0,
            painted_grid_cells: BTreeSet::new(),
            badge_rows: BTreeSet::new(),
            applied_order: Vec::new(),
            #[cfg(test)]
            sync_reconciliations: 0,
            #[cfg(test)]
            badge_row_syncs: 0,
            #[cfg(test)]
            icon_syncs: 0,
            paint_dirty: BTreeSet::new(),
            motion: blockui_motion::Timeline::default(),
            motion_epoch: Instant::now(),
            motion_manual_time: None,
            hover_paints: BTreeMap::new(),
            scroll_targets: BTreeMap::new(),
            painted_scroll: BTreeMap::new(),
            observed_viewports: BTreeMap::new(),
            editors: BTreeMap::new(),
            sliders: BTreeMap::new(),
            selects: BTreeMap::new(),
            select_popup: None,
            select_rows: Vec::new(),
            select_pressed: None,
            select_semantic_ids: BTreeMap::new(),
            select_semantic_next: 1,
            icons: BTreeMap::new(),
            canvases: BTreeMap::new(),
            scroll_children: BTreeMap::new(),
            placed: Vec::new(),
            semantics_enabled: false,
            semantic_revision: 0,
            semantics: None,
            focus: None,
            focus_epoch: 0,
            pending_reveal: None,
            suspended_grid_focus: None,
            modal: None,
            modal_opener: None,
            toast_return_focus: None,
            dialog_tops: BTreeMap::new(),
            paste_target: None,
            pressed: None,
            pointer_pressed_visual: None,
            hovered: None,
            tooltip: Default::default(),
            menus: Default::default(),
            menubar_stops: Default::default(),
            context_anchor: None,
            menu_open_last: None,
            menu_hover: Default::default(),
            menu_pointer_active: false,
            pointer_position: None,
            keyboard_pressed: None,
            primary_shortcuts: BTreeMap::new(),
            composing_editor: None,
            window_focused: true,
            scale: 1.0,
            wake: Arc::new(Mutex::new(None)),
            builds: None,
            watcher: None,
            actor_watches: BTreeMap::new(),
            actor_diagnostics: BTreeMap::new(),
            implementation_watches: BTreeMap::new(),
            watch_expected: None,
            smoke: None,
            graph_smoke: None,
            scope_smoke: None,
            scope_watch: false,
            scope_watch_expected: None,
            io_smoke: None,
            save_smoke: None,
            editor_window: None,
            io_expected: None,
            frames: 0,
            capture: false,
            capture_wait_focus: false,
            clipboard_smoke: None,
            exit: false,
            metrics: None,
            diagnostic: String::new(),
        };
        source.sync()?;
        Ok(source)
    }
    /// Bind Control+letter (Command+letter on macOS) to a stable button identity.
    /// Only identities are retained; activation resolves the current widget/action.
    pub fn primary_shortcut(mut self, letter: &str, widget: u64) -> Result<Self, String> {
        if letter.len() != 1
            || !letter.is_ascii()
            || !letter.as_bytes()[0].is_ascii_alphabetic()
            || widget == 0
        {
            return Err("shortcut requires an ASCII letter and nonzero widget identity".into());
        }
        let letter = letter.to_ascii_lowercase();
        if self.primary_shortcuts.len() >= 26 || self.primary_shortcuts.contains_key(&letter) {
            return Err("duplicate shortcut or shortcut capacity reached".into());
        }
        self.primary_shortcuts.insert(letter, widget);
        Ok(self)
    }
    /// Registers an independent build target for this exact actor lifecycle.
    pub fn actor_diagnostic(&self, target: ActorId) -> Option<&str> {
        self.actor_diagnostics.get(&target).map(String::as_str)
    }
    pub fn watch_actor(&mut self, target: ActorId, config: ReloadConfig) -> Result<(), String> {
        if !cfg!(feature = "hotreload") {
            return Err(
                "native hot reload is disabled; enable the runtime hotreload feature".into(),
            );
        }
        let lifecycle = self.actor.lifecycle(target).ok_or("unknown watch actor")?;
        if self.actor_watches.len() >= 64
            || self.actor_watches.contains_key(&target)
            || (self.watcher.is_some() && target.0 == self.actor.root_actor().id)
        {
            return Err("duplicate watch target or watch capacity reached".into());
        }
        let (watcher, builds) = watch(config, self.wake.clone());
        self.actor_watches.insert(
            target,
            ActorWatch {
                target,
                lifecycle,
                watcher: Some(watcher),
                builds,
            },
        );
        Ok(())
    }
    /// One build watcher shared by present/future declared children of this implementation.
    pub fn watch_implementation(&mut self, name: &str, config: ReloadConfig) -> Result<(), String> {
        if !cfg!(feature = "hotreload") {
            return Err(
                "native hot reload is disabled; enable the runtime hotreload feature".into(),
            );
        }
        let identity = key(name);
        if name.is_empty()
            || self.implementation_watches.len() >= 64
            || self.implementation_watches.contains_key(&identity)
        {
            return Err("duplicate implementation watcher or capacity reached".into());
        }
        let (watcher, builds) = watch(config, self.wake.clone());
        self.implementation_watches.insert(
            identity,
            ImplementationWatch {
                name: name.to_owned(),
                watcher: Some(watcher),
                builds,
            },
        );
        self.reconcile_implementation_watches()
    }
    fn reconcile_implementation_watches(&mut self) -> Result<(), String> {
        self.actor_watches
            .retain(|id, target| self.actor.lifecycle(*id) == Some(target.lifecycle));
        let children = self.actor.declared_children();
        self.actor_diagnostics.retain(|id, _| {
            self.actor_watches.contains_key(id)
                || children.iter().any(|(child, implementation)| {
                    child == id && self.implementation_watches.contains_key(implementation)
                })
        });
        Ok(())
    }
    pub fn watch_smoke(mut self) -> Result<Self, String> {
        self.actor.root_actor_mut().dispatch(Message {
            action: key("increment"),
            value: 0,
        })?;
        self.actor
            .root_actor_mut()
            .set_text(key("notes"), "Watch keeps unsaved notes")?;
        self.actor.tick()?;
        self.sync()?;
        let input = self
            .actor
            .root_widget(key("counter.notes"))
            .ok_or("missing notes widget")?;
        let scroll = self
            .actor
            .root_widget(key("counter.scroll"))
            .ok_or("missing scroll widget")?;
        self.focus = Some(input);
        self.editors.get_mut(&input).unwrap().focus_changed(true);
        self.actor
            .view_mut()
            .renderer
            .get_mut(&scroll)
            .unwrap()
            .scroll = 137.0;
        self.watch_expected = Some((
            self.actor.root_actor().state_address(),
            self.actor.root_actor().slot_address("count"),
            self.actor.root_actor().implementation_version(),
            false,
        ));
        Ok(self)
    }
    pub fn graph_smoke(mut self, replacement: &Path) -> Result<Self, String> {
        self.actor.root_actor_mut().dispatch(Message {
            action: key("step"),
            value: 1,
        })?;
        self.actor
            .root_actor_mut()
            .set_text(key("notes"), "Native computed state stays")?;
        self.sync()?;
        let input = key("graph.notes");
        self.focus = Some(input);
        self.editors.get_mut(&input).unwrap().focus_changed(true);
        self.graph_smoke = Some(replacement.into());
        Ok(self)
    }
    /// Exercise retained native controls during a local scope update in an isolated window.
    pub fn scope_smoke(mut self) -> Self {
        self.scope_smoke = Some(false);
        self
    }
    pub fn scope_watch_smoke(mut self) -> Self {
        self.scope_watch = true;
        self.scope_smoke()
    }
    pub fn capture(mut self) -> Self {
        self.capture = true;
        self
    }
    pub fn clipboard_smoke(mut self, report: PathBuf) -> Result<Self, String> {
        if let Some(parent) = report.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        self.actor
            .root_actor_mut()
            .set_text(key("notes"), "native clipboard 😀")?;
        self.sync()?;
        self.set_focus(Some(key("counter.notes")));
        self.window_focused = false;
        self.clipboard_smoke = Some((report, false));
        Ok(self)
    }
    pub fn capture_on_focus(mut self) -> Self {
        self.capture = true;
        self.capture_wait_focus = true;
        self.window_focused = false;
        self
    }
    pub fn io_smoke(mut self, replacement: &Path, file: &Path) -> Result<Self, String> {
        self.actor
            .root_actor_mut()
            .set_text(key("notes"), file.to_str().ok_or("invalid UTF8 file path")?)?;
        self.actor.root_actor_mut().dispatch(Message {
            action: key("file_open"),
            value: 1,
        })?;
        self.sync()?;
        self.io_smoke = Some(replacement.into());
        Ok(self)
    }
    pub fn save_smoke(self, replacement: &Path, file: &Path) -> Result<Self, String> {
        let mut source = self.io_smoke(replacement, file)?;
        source.actor.root_actor_mut().dispatch(Message {
            action: key("file_close"),
            value: 0,
        })?;
        source.actor.root_actor_mut().dispatch(Message {
            action: key("file_open"),
            value: 2,
        })?;
        source.save_smoke = Some(file.into());
        Ok(source)
    }
    pub fn editor_window(mut self, replacement: &Path, file: &Path) -> Result<Self, String> {
        self.actor
            .root_actor_mut()
            .set_text(key("editor.path"), file.to_str().ok_or("invalid path")?)?;
        self.actor
            .root_actor_mut()
            .set_text(key("editor.draft"), "Window draft")?;
        self.sync()?;
        self.editor_window = Some(EditorWindowProof {
            replacement: replacement.into(),
            file: file.into(),
            expected: None,
            loading: false,
        });
        Ok(self)
    }
    fn persist_native_renderer(&mut self) {
        for (key, editor) in &self.editors {
            if let Some(selection) = editor.text_selection()
                && let Some(state) = self.actor.view_mut().renderer.get_mut(key)
            {
                state.selection = (
                    selection.anchor.min(i32::MAX as usize) as i32,
                    selection.head.min(i32::MAX as usize) as i32,
                );
                state.selection_saved = true;
            }
        }
        self.actor.persist_renderer();
    }
    fn select_semantic_id(&mut self, owner: u64, value: Option<i64>) -> u64 {
        if let Some(id) = self.select_semantic_ids.get(&(owner, value)).copied()
            && !self.actor.view().bindings.contains_key(&id)
        {
            return id;
        }
        loop {
            let id = self.select_semantic_next;
            self.select_semantic_next = self
                .select_semantic_next
                .checked_add(1)
                .expect("semantic ID exhaustion");
            if !self.actor.view().bindings.contains_key(&id)
                && !self.select_semantic_ids.values().any(|used| *used == id)
            {
                self.select_semantic_ids.insert((owner, value), id);
                return id;
            }
        }
    }
    fn refresh_semantics(&mut self) {
        if !self.semantics_enabled {
            return;
        }
        let view = self.actor.view();
        let bounds: HashMap<_, _> = self
            .placed
            .iter()
            .map(|placed| (placed.node_id, placed.rect))
            .collect();
        let mut snapshot = shell::SemanticSnapshot {
            focus: self.focus,
            ..Default::default()
        };
        let mut children: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        for key in &view.order {
            children
                .entry(view.bindings[key].node.parent)
                .or_default()
                .push(*key);
        }
        for key in &view.order {
            let binding = &view.bindings[key];
            let node = binding.node;
            if (node.kind == ICON && binding.text.is_empty())
                || node.semantics.hidden != 0
                || (node.semantics.role == semantics::TOOLTIP
                    && self.tooltip.visible() != Some(*key))
            {
                continue;
            }
            let role = match node.semantics.role {
                semantics::TOOLTIP => shell::SemanticRole::Tooltip,
                semantics::TABLE => shell::SemanticRole::Table,
                semantics::TABLE_ROW => shell::SemanticRole::Row,
                semantics::COLUMN_HEADER => shell::SemanticRole::ColumnHeader,
                semantics::CELL => shell::SemanticRole::Cell,
                semantics::HEADING => shell::SemanticRole::Heading,
                semantics::HORIZONTAL_SEPARATOR | semantics::VERTICAL_SEPARATOR => {
                    shell::SemanticRole::Separator
                }
                semantics::NAVIGATION => shell::SemanticRole::Region,
                semantics::TAB_LIST => shell::SemanticRole::TabList,
                semantics::TAB => shell::SemanticRole::Tab,
                semantics::TAB_PANEL => shell::SemanticRole::TabPanel,
                semantics::RADIO_GROUP => shell::SemanticRole::RadioGroup,
                semantics::RADIO => shell::SemanticRole::RadioButton,
                semantics::SWITCH => shell::SemanticRole::Switch,
                semantics::ALERT => shell::SemanticRole::Alert,
                semantics::STATUS => shell::SemanticRole::Status,
                semantics::POPOVER_PANEL => shell::SemanticRole::Dialog,
                semantics::MENU_PANEL => shell::SemanticRole::Menu,
                semantics::MENUBAR => shell::SemanticRole::MenuBar,
                semantics::MENU_GROUP | semantics::CONTEXT_SURFACE => shell::SemanticRole::Group,
                semantics::MENU_TRIGGER
                    if view
                        .bindings
                        .get(&node.parent)
                        .and_then(|parent| view.bindings.get(&parent.node.parent))
                        .is_some_and(|bar| bar.node.semantics.role == semantics::MENUBAR) =>
                {
                    shell::SemanticRole::MenuItem
                }

                semantics::MENU_ITEM | semantics::SUBMENU_TRIGGER => shell::SemanticRole::MenuItem,
                semantics::MENU_CHECKBOX => shell::SemanticRole::MenuItemCheckBox,
                semantics::MENU_RADIO => shell::SemanticRole::MenuItemRadio,
                semantics::DISCLOSURE_PANEL | semantics::TOAST_VIEWPORT => {
                    shell::SemanticRole::Region
                }
                _ => match node.kind {
                    DIALOG => shell::SemanticRole::Dialog,
                    SCROLL if node.navigate_action != 0 => shell::SemanticRole::Grid,
                    ROW if view.bindings.get(&node.parent).is_some_and(|parent| {
                        parent.node.kind == SCROLL && parent.node.navigate_action != 0
                    }) =>
                    {
                        shell::SemanticRole::Row
                    }
                    BUTTON if self.grid_owner(*key).is_some() => shell::SemanticRole::GridCell,
                    BUTTON => shell::SemanticRole::Button,
                    CHECKBOX => shell::SemanticRole::CheckBox,
                    SLIDER => shell::SemanticRole::Slider,
                    SELECT => shell::SemanticRole::ComboBox,
                    PROGRESS => shell::SemanticRole::ProgressBar,
                    ENTRY | TEXTAREA => shell::SemanticRole::TextInput,
                    LABEL | ICON => shell::SemanticRole::Label,
                    _ => shell::SemanticRole::Group,
                },
            };
            if node.parent == 0 {
                snapshot.root_children.push(*key);
            }
            snapshot.nodes.push(shell::SemanticNode {
                id: *key,
                role,
                label: if node.semantics.role == semantics::TOAST_VIEWPORT {
                    Some("Notifications".into())
                } else if node.semantics.role == semantics::COLUMN_HEADER {
                    let mut pending: Vec<_> = children
                        .get(key)
                        .into_iter()
                        .flatten()
                        .rev()
                        .copied()
                        .collect();
                    let mut labels = Vec::new();
                    while let Some(child) = pending.pop() {
                        let binding = &view.bindings[&child];
                        if matches!(binding.node.kind, LABEL | BUTTON) && !binding.text.is_empty() {
                            labels.push(binding.text.as_str());
                        } else if let Some(descendants) = children.get(&child) {
                            pending.extend(descendants.iter().rev().copied());
                        }
                    }
                    Some(labels.join(" "))
                } else if (node.kind == CHECKBOX && node.semantics.labelled_by != 0)
                    || (node.semantics.role != 0 && node.semantics.role != semantics::SWITCH)
                {
                    Some(
                        view.bindings
                            .get(&node.semantics.labelled_by)
                            .and_then(|label| {
                                if label.node.semantics.role == semantics::CONTEXT_SURFACE {
                                    view.bindings.get(&label.node.semantics.labelled_by)
                                } else {
                                    Some(label)
                                }
                            })
                            .map_or(binding.text.as_str(), |label| label.text.as_str())
                            .to_owned(),
                    )
                } else if matches!(
                    node.kind,
                    ENTRY | TEXTAREA | SELECT | PROGRESS | CANVAS | DIALOG
                ) {
                    binding
                        .semantic_label
                        .as_ref()
                        .map(|label| label.to_string())
                        .or_else(|| (node.kind == DIALOG).then(|| binding.text.clone()))
                } else {
                    matches!(node.kind, LABEL | ICON | BUTTON | CHECKBOX | SLIDER).then(|| {
                        if node.kind == CHECKBOX {
                            // The binding prepends a visual mark; accessibility exposes
                            // the boolean separately from the authored control name.
                            binding
                                .text
                                .strip_prefix("☑ ")
                                .or_else(|| binding.text.strip_prefix("☐ "))
                                .unwrap_or(&binding.text)
                                .to_owned()
                        } else if role == shell::SemanticRole::GridCell {
                            binding
                                .text
                                .strip_prefix("✓ ")
                                .unwrap_or(&binding.text)
                                .to_owned()
                        } else {
                            binding.text.clone()
                        }
                    })
                },
                value: if node.kind == SELECT {
                    self.selects.get(key).and_then(|control| {
                        control
                            .options()
                            .iter()
                            .find(|option| Some(option.value) == control.selected())
                            .map(|option| option.label.clone())
                    })
                } else {
                    matches!(node.kind, ENTRY | TEXTAREA).then(|| binding.text.clone())
                },
                numeric_value: if node.kind == PROGRESS {
                    binding.text.parse::<f64>().ok()
                } else {
                    (node.kind == SLIDER)
                        .then(|| self.actor.widget_integer(*key).map(|v| v as f64))
                        .flatten()
                },
                min_numeric_value: if node.kind == PROGRESS {
                    Some(0.0)
                } else {
                    (node.kind == SLIDER).then_some(f64::from(node.range.min))
                },
                max_numeric_value: if node.kind == PROGRESS {
                    Some(1.0)
                } else {
                    (node.kind == SLIDER).then_some(f64::from(node.range.max))
                },
                bounds: self
                    .ids
                    .get(key)
                    .and_then(|id| bounds.get(id))
                    .copied()
                    .unwrap_or_default(),
                children: children
                    .remove(key)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|child| {
                        view.bindings[child].node.semantics.hidden == 0
                            && !(view.bindings[child].node.kind == ICON
                                && view.bindings[child].text.is_empty())
                            && (view.bindings[child].node.semantics.role != semantics::TOOLTIP
                                || self.tooltip.visible() == Some(*child))
                    })
                    .collect(),
                controls: (node.semantics.controls != 0)
                    .then_some(node.semantics.controls)
                    .into_iter()
                    .collect(),
                described_by: (node.semantics.described_by != 0
                    && self.tooltip.visible() == Some(node.semantics.described_by))
                .then_some(node.semantics.described_by)
                .into_iter()
                .collect(),
                active_descendant: None,
                disabled: node.disabled != 0 || node.semantics.locked != 0,
                states: shell::SemanticStates {
                    sort_direction: match node.semantics.sort_direction {
                        1 => Some(shell::SemanticSortDirection::Ascending),
                        2 => Some(shell::SemanticSortDirection::Descending),
                        3 => Some(shell::SemanticSortDirection::Other),
                        _ => None,
                    },
                    modal: if node.semantics.role == semantics::POPOVER_PANEL {
                        Some(false)
                    } else {
                        (node.kind == DIALOG).then_some(true)
                    },
                    current: (node.semantics.current == 1).then_some(shell::SemanticCurrent::Page),
                    selected: (node.semantics.role == semantics::TAB)
                        .then_some(node.semantics.selected != 0),
                    expanded: if node.semantics.expanded != 0 {
                        Some(node.semantics.expanded == 2)
                    } else {
                        (node.kind == SELECT).then(|| {
                            self.selects
                                .get(key)
                                .is_some_and(|control| control.is_open())
                        })
                    },
                    popup: if matches!(
                        node.semantics.role,
                        semantics::MENU_TRIGGER
                            | semantics::SUBMENU_TRIGGER
                            | semantics::CONTEXT_TRIGGER
                            | semantics::CONTEXT_SURFACE
                    ) {
                        Some(shell::SemanticPopup::Menu)
                    } else {
                        (node.kind == SELECT).then_some(shell::SemanticPopup::ListBox)
                    },
                    checked: if matches!(
                        node.semantics.role,
                        semantics::RADIO | semantics::MENU_CHECKBOX | semantics::MENU_RADIO
                    ) {
                        Some(if node.semantics.selected != 0 {
                            shell::SemanticToggleState::On
                        } else {
                            shell::SemanticToggleState::Off
                        })
                    } else if node.kind == CHECKBOX {
                        self.actor.widget_boolean(*key).map(|checked| {
                            if checked {
                                shell::SemanticToggleState::On
                            } else {
                                shell::SemanticToggleState::Off
                            }
                        })
                    } else {
                        None
                    },
                    ..Default::default()
                },
            });
        }
        if let Some((owner, layout, values)) = self.select_popup.clone()
            && self
                .selects
                .get(&owner)
                .is_some_and(|control| control.is_open())
        {
            self.select_semantic_ids.retain(|(key, value), _| {
                *key == owner && value.is_none_or(|value| values.contains(&value))
            });
            let list = self.select_semantic_id(owner, None);
            let mut option_ids = Vec::new();
            let mut active = None;
            for (offset, value) in values.iter().copied().enumerate() {
                let id = self.select_semantic_id(owner, Some(value));
                let control = &self.selects[&owner];
                let Some(option) = control
                    .options()
                    .iter()
                    .find(|option| option.value == value)
                else {
                    continue;
                };
                if control.active() == Some(value) {
                    active = Some(id);
                }
                option_ids.push(id);
                snapshot.nodes.push(shell::SemanticNode {
                    id,
                    role: shell::SemanticRole::Option,
                    label: Some(option.label.clone()),
                    value: None,
                    numeric_value: None,
                    min_numeric_value: None,
                    max_numeric_value: None,
                    bounds: layout.row_rect(layout.start + offset).unwrap(),
                    children: Vec::new(),
                    controls: Vec::new(),
                    described_by: Vec::new(),
                    active_descendant: None,
                    disabled: !option.enabled,
                    states: shell::SemanticStates {
                        selected: Some(control.selected() == Some(value)),
                        ..Default::default()
                    },
                });
            }
            snapshot.nodes.push(shell::SemanticNode {
                id: list,
                role: shell::SemanticRole::ListBox,
                label: Some("Options".into()),
                value: None,
                numeric_value: None,
                min_numeric_value: None,
                max_numeric_value: None,
                bounds: layout.rect,
                children: option_ids,
                controls: Vec::new(),
                described_by: Vec::new(),
                active_descendant: active,
                disabled: false,
                states: Default::default(),
            });
            if let Some(control) = snapshot.nodes.iter_mut().find(|node| node.id == owner) {
                control.children.push(list);
                control.controls.push(list);
                control.active_descendant = active;
            }
        } else {
            self.select_semantic_ids.clear();
        }
        let previous = self.semantics.as_deref();
        snapshot.revision = self.semantic_revision;
        if previous == Some(&snapshot) {
            return;
        }
        self.semantic_revision = self.semantic_revision.saturating_add(1);
        snapshot.revision = self.semantic_revision;
        self.semantics = Some(Arc::new(snapshot));
    }
    // Painted nodes can omit descendants of a fully clipped subtree. Reveal
    // geometry therefore comes from the retained layout tree, not the frame.
    fn widget_layout_rect(&self, target: u64) -> Option<[f32; 4]> {
        if matches!(
            self.actor.view().bindings.get(&target)?.node.semantics.role,
            semantics::TOAST_VIEWPORT | semantics::POPOVER_PANEL | semantics::MENU_PANEL
        ) {
            return self
                .placed
                .iter()
                .find(|n| self.ids.get(&target) == Some(&n.node_id))
                .map(|n| n.rect);
        }
        let layout = self.tree.layout(*self.ids.get(&target)?).ok()?;
        let (mut x, mut y) = (layout.location.x, layout.location.y);
        let mut parent = self.actor.view().bindings.get(&target)?.node.parent;
        while parent != 0 {
            let binding = self.actor.view().bindings.get(&parent)?;
            if matches!(
                binding.node.semantics.role,
                semantics::TOAST_VIEWPORT | semantics::POPOVER_PANEL | semantics::MENU_PANEL
            ) {
                let viewport = self
                    .placed
                    .iter()
                    .find(|n| self.ids.get(&parent) == Some(&n.node_id))?;
                x += viewport.rect[0] - self.actor.view().renderer.get(&parent)?.scroll_x as f32;
                y += viewport.rect[1] - self.actor.view().renderer.get(&parent)?.scroll as f32;
                break;
            }
            let ancestor = self.tree.layout(*self.ids.get(&parent)?).ok()?;
            x += ancestor.location.x;
            y += ancestor.location.y;
            if binding.node.kind == SCROLL {
                x -= self.actor.view().renderer.get(&parent)?.scroll_x as f32;
                y -= self.actor.view().renderer.get(&parent)?.scroll as f32;
            }
            parent = binding.node.parent;
        }
        Some([x, y, x + layout.size.width, y + layout.size.height])
    }
    fn reveal_widget(&mut self, target: u64) -> bool {
        let Some(mut rect) = self.widget_layout_rect(target) else {
            return false;
        };
        let mut parent = self.actor.view().bindings[&target].node.parent;
        let mut updates = Vec::new();
        while parent != 0 {
            let node = self.actor.view().bindings[&parent].node;
            if node.kind == SCROLL {
                let Some(bounds) = self.widget_layout_rect(parent) else {
                    return false;
                };
                let Ok(layout) = self.tree.layout(self.ids[&parent]) else {
                    return false;
                };
                let port_top = bounds[1] + layout.border.top + layout.padding.top;
                let port_bottom = bounds[3] - layout.border.bottom - layout.padding.bottom;
                let delta = if rect[1] < port_top || rect[3] - rect[1] > port_bottom - port_top {
                    rect[1] - port_top
                } else if rect[3] > port_bottom {
                    rect[3] - port_bottom
                } else {
                    0.0
                };
                let port_left = bounds[0] + layout.border.left + layout.padding.left;
                let port_right = bounds[2] - layout.border.right - layout.padding.right;
                let delta_x = if rect[0] < port_left || rect[2] - rect[0] > port_right - port_left {
                    rect[0] - port_left
                } else if rect[2] > port_right {
                    rect[2] - port_right
                } else {
                    0.0
                };
                let state = &self.actor.view().renderer[&parent];
                let previous = [state.scroll_x, state.scroll];
                let offset = [
                    (previous[0] + f64::from(delta_x)).clamp(0.0, f64::from(layout.scroll_width())),
                    (previous[1] + f64::from(delta)).clamp(0.0, f64::from(layout.scroll_height())),
                ];
                for axis in 0..2 {
                    let movement = (offset[axis] - previous[axis]) as f32;
                    rect[axis] -= movement;
                    rect[axis + 2] -= movement;
                }
                updates.push((parent, offset));
            }
            if matches!(
                node.semantics.role,
                semantics::TOAST_VIEWPORT | semantics::POPOVER_PANEL | semantics::MENU_PANEL
            ) {
                break;
            }
            parent = node.parent;
        }
        if updates.is_empty() {
            return false;
        }
        for (key, offset) in updates {
            let state = self.actor.view_mut().renderer.get_mut(&key).unwrap();
            state.scroll_x = offset[0];
            state.scroll = offset[1];
            self.notify_scroll(key);
        }
        true
    }
    fn in_modal(&self, mut key: u64) -> bool {
        let Some(modal) = self.modal else {
            return true;
        };
        loop {
            if key == modal {
                return true;
            }
            let Some(binding) = self.actor.view().bindings.get(&key) else {
                return false;
            };
            key = binding.node.parent;
            if key == 0 {
                return false;
            }
        }
    }
    fn notify_scroll(&mut self, key: u64) {
        if self
            .actor
            .view()
            .bindings
            .get(&key)
            .is_none_or(|binding| binding.node.scroll_action == 0)
        {
            return;
        }
        let state = &self.actor.view().renderer[&key];
        let point = blockui_abi::LocalPoint {
            x: state.scroll_x as f32,
            y: state.scroll as f32,
        };
        if let Err(error) = self.actor.observe_scroll(key, point) {
            self.diagnostic = error;
        }
    }
    fn grid_owner(&self, key: u64) -> Option<u64> {
        let node = self.actor.view().bindings.get(&key)?.node;
        if node.kind != BUTTON {
            return None;
        }
        let mut parent = node.parent;
        while parent != 0 {
            let node = self.actor.view().bindings.get(&parent)?.node;
            if node.kind == SCROLL && node.navigate_action != 0 {
                return Some(parent);
            }
            parent = node.parent;
        }
        None
    }
    fn grid_target(&self, grid: u64, index: i64) -> Option<u64> {
        self.actor.view().order.iter().copied().find(|key| {
            let candidate = self.actor.view().bindings[key].node;
            candidate.kind == BUTTON
                && candidate.action_payload == index
                && self.grid_owner(*key) == Some(grid)
        })
    }
    fn grid_navigation_origin(&self) -> Option<(u64, i64, u64)> {
        let (grid, index, cell) = if let Some(focus) = self.focus {
            (
                self.grid_owner(focus)?,
                self.actor.view().bindings[&focus].node.action_payload,
                focus,
            )
        } else {
            let (grid, index) = self.suspended_grid_focus?;
            let cell = self
                .actor
                .view()
                .order
                .iter()
                .copied()
                .find(|key| self.grid_owner(*key) == Some(grid))?;
            (grid, index, cell)
        };
        let parent = self.actor.view().bindings[&cell].node.parent;
        let stride = if self
            .actor
            .view()
            .bindings
            .get(&parent)
            .is_some_and(|binding| binding.node.kind == ROW)
        {
            parent
        } else {
            cell
        };
        Some((grid, index, stride))
    }
    fn focus_branch(&self, key: u64) -> Option<u64> {
        self.actor.view().renderer_branch(key)
    }
    fn set_pointer_pressed_visual(&mut self, next: Option<u64>) {
        if self.pointer_pressed_visual != next {
            if let Some(old) = self.pointer_pressed_visual {
                self.paint_dirty.insert(old);
            }
            if let Some(key) = next {
                self.paint_dirty.insert(key);
            }
            self.pointer_pressed_visual = next;
        }
    }
    fn set_focus(&mut self, next: Option<u64>) {
        self.suspended_grid_focus = None;
        if let Some(key) = next.filter(|key| self.toast_owner(*key).is_none()) {
            self.toast_return_focus = Some(key);
        }
        let next = next.filter(|key| self.in_modal(*key));
        if let Some((bar, trigger)) = next.and_then(|key| self.menubar_focus_owner(key)) {
            self.menubar_stops.insert(bar, trigger);
        }

        if self.focus != next {
            if let Some(old) = self.focus.and_then(|key| self.selects.get_mut(&key)) {
                old.close();
            }
            self.select_popup = None;
            self.select_pressed = None;
            self.focus_epoch = self.focus_epoch.saturating_add(1);
        }
        if let Some(branch) = next.and_then(|key| self.focus_branch(key)) {
            let members: Vec<_> = self
                .actor
                .view()
                .order
                .iter()
                .copied()
                .filter(|key| self.focus_branch(*key) == Some(branch))
                .collect();
            for key in members {
                if let Some(state) = self.actor.view_mut().renderer.get_mut(&key) {
                    state.focus = Some(key) == next;
                    if state.focus {
                        state.focus_epoch = self.focus_epoch;
                    }
                }
            }
        } else if let Some(old) = self.focus
            && let Some(state) = self.actor.view_mut().renderer.get_mut(&old)
        {
            state.focus = false;
        }
        self.actor.persist_renderer();
        if let Some(key) = next
            && let Some(grid) = self.grid_owner(key)
        {
            let index = self.actor.view().bindings[&key].node.action_payload;
            if let Some(state) = self.actor.view_mut().renderer.get_mut(&grid) {
                state.grid_focus = Some(index);
            }
        }
        if self.focus == next {
            return;
        }
        self.paste_target = None;
        self.composing_editor = None;
        if let Some(old) = self.focus.take() {
            if let Some(slider) = self.sliders.get_mut(&old) {
                slider.focus_changed(false);
            }
            if let Some(editor) = self.editors.get_mut(&old) {
                editor.focus_changed(false);
            }
            self.paint_dirty.insert(old);
        }
        if let Some(old) = self.keyboard_pressed.take() {
            self.paint_dirty.insert(old);
        }
        self.focus = next;
        if let Some(key) = next {
            if let Some(editor) = self.editors.get_mut(&key) {
                editor.focus_changed(self.window_focused);
            }
            self.paint_dirty.insert(key);
        }
        self.reconcile_toast_activity();
        self.reconcile_tooltip();
    }
    fn materialize_grid_stop(&mut self, fallback: u64) -> Option<u64> {
        let Some(grid) = self.grid_owner(fallback) else {
            return Some(fallback);
        };
        let node = self.actor.view().bindings[&grid].node;
        let Some(index) = self.actor.view().renderer[&grid].grid_focus else {
            return Some(fallback);
        };
        if let Some(target) = self.grid_target(grid, index) {
            return (self.actor.view().bindings[&target].node.disabled == 0)
                .then_some(target)
                .or(Some(fallback));
        }
        if node.scroll_action == 0
            || index < 0
            || index as u64 >= u64::from(node.grid_rows) * u64::from(node.grid_columns)
        {
            return Some(fallback);
        }
        let parent = self.actor.view().bindings[&fallback].node.parent;
        let stride_key = if self.actor.view().bindings[&parent].node.kind == ROW {
            parent
        } else {
            fallback
        };
        let stride = self
            .tree
            .layout(*self.ids.get(&stride_key)?)
            .ok()?
            .size
            .height;
        let range = self
            .tree
            .layout(*self.ids.get(&grid)?)
            .ok()?
            .scroll_height();
        let y = (index as f32 / node.grid_columns as f32).floor() * stride;
        self.actor.view_mut().renderer.get_mut(&grid)?.scroll = f64::from(y.clamp(0.0, range));
        self.notify_scroll(grid);
        if let Err(error) = self.sync() {
            self.diagnostic = error;
        }
        self.grid_target(grid, index)
            .filter(|key| self.actor.view().bindings[key].node.disabled == 0)
            .or_else(|| {
                self.actor.view().order.iter().copied().find(|key| {
                    self.grid_owner(*key) == Some(grid)
                        && self.actor.view().bindings[key].node.disabled == 0
                })
            })
    }
    fn focus_next(&mut self, reverse: bool) {
        // Match the mounted native tree, including children mounted inside a
        // parent's column rather than the flat actor descriptor allocation order.
        let mut children: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        for key in &self.actor.view().order {
            children
                .entry(self.actor.view().bindings[key].node.parent)
                .or_default()
                .push(*key);
        }
        let mut stack: Vec<_> = children
            .get(&0)
            .into_iter()
            .flatten()
            .rev()
            .copied()
            .collect();
        let mut grid_stops = BTreeMap::new();
        for key in &self.actor.view().order {
            let node = self.actor.view().bindings[key].node;
            if node.disabled != 0 || !self.in_modal(*key) {
                continue;
            }
            if let Some(grid) = self.grid_owner(*key) {
                let preferred = self
                    .actor
                    .view()
                    .renderer
                    .get(&grid)
                    .and_then(|state| state.grid_focus);
                let entry = grid_stops.entry(grid).or_insert(*key);
                if preferred == Some(node.action_payload) {
                    *entry = *key;
                }
            }
        }
        let mut bar_stops = BTreeMap::new();
        for key in &self.actor.view().order {
            let node = self.actor.view().bindings[key].node;
            if node.semantics.role == semantics::MENU_TRIGGER
                && node.disabled == 0
                && let Some((bar, _)) = self.menubar_focus_owner(*key)
            {
                let stop = bar_stops.entry(bar).or_insert(*key);
                if self.menubar_stops.get(&bar) == Some(key) {
                    *stop = *key;
                }
            }
        }
        self.menubar_stops
            .retain(|bar, _| bar_stops.contains_key(bar));
        let mut keys = Vec::new();
        while let Some(key) = stack.pop() {
            let semantic = self.actor.view().bindings[&key].node.semantics;
            if (matches!(
                self.actor.view().bindings[&key].node.kind,
                BUTTON | CHECKBOX | SELECT | SLIDER | ENTRY | TEXTAREA
            ) || semantic.role == semantics::CONTEXT_SURFACE
                || (semantic.role == semantics::TAB_PANEL && semantic.hidden == 0))
                && (!matches!(semantic.role, semantics::TAB | semantics::RADIO)
                    || semantic.tab_stop != 0)
                && (semantic.role != semantics::MENU_TRIGGER
                    || self
                        .menubar_focus_owner(key)
                        .is_none_or(|(bar, _)| bar_stops.get(&bar) == Some(&key)))
                && self.in_modal(key)
                && self.actor.view().bindings[&key].node.disabled == 0
                && self
                    .grid_owner(key)
                    .is_none_or(|grid| grid_stops.get(&grid) == Some(&key))
            {
                keys.push(key);
            }
            stack.extend(children.get(&key).into_iter().flatten().rev().copied());
        }
        if keys.is_empty() {
            self.set_focus(None);
            return;
        }
        let current = self
            .focus
            .and_then(|key| keys.iter().position(|candidate| *candidate == key))
            .or_else(|| {
                self.suspended_grid_focus.and_then(|(grid, _)| {
                    keys.iter()
                        .position(|key| self.grid_owner(*key) == Some(grid))
                })
            });
        let index = match current {
            Some(index) if reverse => (index + keys.len() - 1) % keys.len(),
            Some(index) => (index + 1) % keys.len(),
            None if reverse => keys.len() - 1,
            None => 0,
        };
        let target = self.materialize_grid_stop(keys[index]);
        self.set_focus(target);
        self.pending_reveal = target;
    }
    fn input_response(&self, consume_key_text: bool) -> EventResponse {
        EventResponse {
            handled: true,
            request_redraw: true,
            consume_key_text,
            text_input: Some(
                self.window_focused
                    && self
                        .focus
                        .is_some_and(|key| self.editors.contains_key(&key)),
            ),
            ..EventResponse::IGNORED
        }
    }
    pub fn smoke(mut self, replacement: &Path) -> Self {
        self.smoke = Some(replacement.into());
        self
    }
    // Text and presentation observers do not normally change Taffy ownership.
    // Scroll text is special: an otherwise nonempty scroll container inserts an
    // extra host-owned text leaf only while its text is nonempty.
    fn layout_children_changed(&self) -> bool {
        self.applied_order != self.actor.view().order
            || self.applied.len() != self.actor.view().bindings.len()
            || self.actor.view().bindings.iter().any(|(key, binding)| {
                self.applied.get(key).is_none_or(|(old, text)| {
                    old.parent != binding.node.parent
                        || old.kind != binding.node.kind
                        || old.semantics.role != binding.node.semantics.role
                        || (binding.node.kind == SCROLL
                            && text.is_empty() != binding.text.is_empty())
                })
            })
    }

    fn control_value_is_applied(&self, key: u64, binding: &crate::Binding) -> bool {
        let kind = binding.node.kind;
        (kind != SELECT
            || self
                .selects
                .get(&key)
                .is_some_and(|control| control.selected() == self.actor.widget_integer(key)))
            && (kind != SLIDER
                || self.sliders.get(&key).is_some_and(|control| {
                    Some(i64::from(control.value())) == self.actor.widget_integer(key)
                }))
            && (!matches!(kind, ENTRY | TEXTAREA)
                || self
                    .editors
                    .get(&key)
                    .is_some_and(|editor| editor.current_value() == Some(binding.text.as_str())))
    }

    fn projection_needs_sync(&self) -> bool {
        let view = self.actor.view();
        !self.paint_dirty.is_empty()
            || self.applied_order != view.order
            || self.applied.len() != view.bindings.len()
            || view.bindings.iter().zip(&self.applied).any(
                |((key, binding), (old_key, (node, text)))| {
                    key != old_key
                        || *node != binding.node
                        || *text != binding.text
                        || !self.control_value_is_applied(*key, binding)
                        || !self.icon_is_applied(*key, binding)
                },
            )
    }

    // Both maps have sorted keys. Walk them together rather than doing a tree
    // lookup per node, sharing the comparison for theme and widget updates.
    fn projection_changes(&self) -> (bool, bool) {
        let view = self.actor.view();
        let topology =
            self.applied_order != view.order || self.applied.len() != view.bindings.len();
        let mut projection = topology || !self.paint_dirty.is_empty();
        let mut themes = topology
            || self.themes.len() != view.bindings.len()
            || self.theme_system_dark
                != self
                    .metrics
                    .is_some_and(|m| m.color_scheme == Some(shell::ColorScheme::Dark));
        if projection && themes {
            return (projection, themes);
        }
        for ((key, binding), (old_key, (old, text))) in view.bindings.iter().zip(&self.applied) {
            if key != old_key {
                return (true, true);
            }
            themes = themes
                || old.parent != binding.node.parent
                || old.presentation.theme != binding.node.presentation.theme
                || old.presentation.density != binding.node.presentation.density;
            projection = projection
                || *old != binding.node
                || *text != binding.text
                || !self.control_value_is_applied(*key, binding)
                || !self.icon_is_applied(*key, binding);
            if projection && themes {
                break;
            }
        }
        (projection, themes)
    }

    fn sync_badge_rows(&mut self) {
        #[cfg(test)]
        {
            self.badge_row_syncs += 1;
        }
        let rows: BTreeSet<_> = self
            .actor
            .view()
            .bindings
            .values()
            .filter(|binding| binding.node.semantics.role == semantics::BADGE)
            .map(|binding| binding.node.parent)
            .filter(|parent| {
                self.actor
                    .view()
                    .bindings
                    .get(parent)
                    .is_some_and(|binding| binding.node.kind == ROW)
            })
            .collect();
        self.paint_dirty
            .extend(self.badge_rows.symmetric_difference(&rows).copied());
        self.badge_rows = rows;
    }

    fn sync(&mut self) -> Result<(), String> {
        let (projection_changed, themes_changed) = self.projection_changes();
        if themes_changed {
            self.rebuild_themes();
        }
        self.sync_motion();
        self.reconcile_implementation_watches()?;
        if !projection_changed && self.paint_dirty.is_empty() {
            return Ok(());
        }
        self.sync_icons()?;
        #[cfg(test)]
        {
            self.sync_reconciliations += 1;
        }
        let layout_children_changed = self.layout_children_changed();
        if layout_children_changed {
            self.sync_badge_rows();
        }
        let focus_before_sync = self.focus;
        let opening_popovers: Vec<_> = self
            .actor
            .view()
            .order
            .iter()
            .copied()
            .filter(|key| {
                let n = self.actor.view().bindings[key].node;
                semantics::is_anchored_panel(n.semantics.role)
                    && n.semantics.hidden == 0
                    && self
                        .applied
                        .get(key)
                        .is_none_or(|(old, _)| old.semantics.hidden != 0)
            })
            .collect();
        let mut closed_popover_focus = None;
        let mut ancestor = self.focus;
        while let Some(key) = ancestor {
            let Some((old, _)) = self.applied.get(&key) else {
                break;
            };
            if semantics::is_anchored_panel(old.semantics.role)
                && self
                    .actor
                    .view()
                    .bindings
                    .get(&key)
                    .is_none_or(|b| b.node.semantics.hidden != 0)
            {
                closed_popover_focus = Some(old.semantics.labelled_by);
            }
            ancestor = (old.parent != 0).then_some(old.parent);
        }
        if self.modal.is_none()
            && self
                .actor
                .view()
                .order
                .iter()
                .any(|key| self.actor.view().bindings[key].node.kind == DIALOG)
        {
            self.modal_opener = self.focus;
        }
        let parent_kind_changes: BTreeSet<u64> = self
            .actor
            .view()
            .bindings
            .iter()
            .filter(|(key, binding)| {
                self.applied.get(key).is_some_and(|(old, _)| {
                    old.kind != binding.node.kind || old.content_width != binding.node.content_width
                })
            })
            .map(|(key, _)| *key)
            .collect();
        if self.focus.is_some_and(|key| {
            self.actor
                .view()
                .bindings
                .get(&key)
                .is_some_and(|b| b.node.disabled != 0)
        }) {
            let next = self.focus.and_then(|key| {
                crate::interaction::navigation_focus::recover(self.actor.view(), key)
            });
            self.set_focus(next);
        }
        if self.pressed.is_some_and(|key| {
            self.actor
                .view()
                .bindings
                .get(&key)
                .is_some_and(|b| b.node.disabled != 0)
        }) {
            self.pressed = None;
            self.set_pointer_pressed_visual(None);
        }
        // A programmatic close can remove a focused descendant. Walk the old
        // host-owned ancestry before retiring it and return focus to the surviving
        // trigger, including when closing an outer disclosure around nested ones.
        let mut previous = self.focus;
        let mut disclosure_return = None;
        let mut toast_return = false;
        while let Some(key) = previous {
            let Some((old, _)) = self.applied.get(&key) else {
                break;
            };
            if old.semantics.role == semantics::TOAST_VIEWPORT
                && self
                    .focus
                    .is_some_and(|key| !self.actor.view().bindings.contains_key(&key))
            {
                toast_return = true;
            }
            if old.semantics.role == semantics::DISCLOSURE_PANEL
                && let Some(panel) = self.actor.view().bindings.get(&key)
                && panel.node.semantics.hidden != 0
            {
                disclosure_return = self
                    .actor
                    .view()
                    .bindings
                    .get(&panel.node.semantics.labelled_by)
                    .filter(|trigger| trigger.node.disabled == 0)
                    .map(|trigger| trigger.node.key);
                break;
            }
            previous = Some(old.parent).filter(|parent| *parent != 0);
        }
        if toast_return {
            let target = self.toast_return_focus.filter(|key| {
                self.actor
                    .view()
                    .bindings
                    .get(key)
                    .is_some_and(|b| b.node.disabled == 0)
            });
            self.set_focus(target);
            if target.is_none() {
                self.focus_next(false);
            }
        }
        if let Some(trigger) = disclosure_return {
            self.set_focus(Some(trigger));
        }
        let removed: Vec<_> = self
            .ids
            .keys()
            .filter(|key| {
                self.actor
                    .view()
                    .bindings
                    .get(key)
                    .is_none_or(|binding| self.kinds[key] != binding.node.kind)
            })
            .copied()
            .collect();
        // Projection removal is not an explicit blur. Remember only the host
        // grid identity/index, never a removed widget or plugin callback.
        if let Some(focus) = self.focus.filter(|key| removed.contains(key))
            && let Some((old, _)) = self.applied.get(&focus)
            && old.kind == BUTTON
        {
            let index = old.action_payload;
            let mut parent = old.parent;
            while let Some((node, _)) = self.applied.get(&parent) {
                if node.kind == SCROLL && node.navigate_action != 0 {
                    if self
                        .actor
                        .view()
                        .bindings
                        .get(&parent)
                        .is_some_and(|binding| {
                            binding.node.kind == SCROLL && binding.node.navigate_action != 0
                        })
                    {
                        self.suspended_grid_focus = Some((parent, index));
                    }
                    break;
                }
                parent = node.parent;
            }
        }
        for key in removed {
            if let Some(id) = self.ids.remove(&key) {
                self.tree.remove(id).map_err(|e| e.to_string())?;
            }
            if let Some(child) = self.scroll_children.remove(&key) {
                self.tree.remove(child).map_err(|e| e.to_string())?;
            }
            self.editors.remove(&key);
            self.sliders.remove(&key);
            self.selects.remove(&key);
            self.canvases.remove(&key);
            self.dialog_tops.remove(&key);
            if self.composing_editor == Some(key) {
                self.composing_editor = None;
            }
            self.kinds.remove(&key);
            self.applied.remove(&key);
            self.painted_grid_cells.remove(&key);
            if self.pointer_pressed_visual == Some(key) {
                self.pointer_pressed_visual = None;
            }
            if self.focus == Some(key) {
                self.focus = None;
            }
            if self.paste_target == Some(key) {
                self.paste_target = None;
            }
            if self.keyboard_pressed == Some(key) {
                self.keyboard_pressed = None;
            }
            if self.pressed == Some(key) {
                self.pressed = None;
            }
        }
        // A programmatic selection can hide a focused tab panel without removing
        // its stable placeholder. Return focus to the group's current Tab entry.
        if let Some(focus) = self.focus
            && let Some(panel) = self.actor.view().bindings.get(&focus)
            && panel.node.semantics.hidden != 0
        {
            let parent = self
                .actor
                .view()
                .bindings
                .get(&panel.node.semantics.labelled_by)
                .map(|tab| tab.node.parent);
            let next = self.actor.view().order.iter().copied().find(|key| {
                let node = self.actor.view().bindings[key].node;
                Some(node.parent) == parent
                    && node.semantics.role == semantics::TAB
                    && node.semantics.tab_stop != 0
                    && node.disabled == 0
            });
            self.set_focus(next);
        }
        let returning_focus = self
            .actor
            .view()
            .order
            .iter()
            .copied()
            .filter(|key| {
                let node = self.actor.view().bindings[key].node;
                !self.applied.contains_key(key)
                    && self
                        .focus_branch(*key)
                        .is_some_and(|branch| !self.applied.contains_key(&branch))
                    && self.actor.view().renderer[key].focus
                    && node.disabled == 0
                    && matches!(
                        node.kind,
                        BUTTON | CHECKBOX | SELECT | SLIDER | ENTRY | TEXTAREA
                    )
            })
            .max_by_key(|key| self.actor.view().renderer[key].focus_epoch);
        for key in &self.actor.view().order {
            let binding = &self.actor.view().bindings[key];
            let kind = binding.node.kind;
            let grid_cell = kind == BUTTON && self.grid_owner(*key).is_some();
            if self.painted_grid_cells.contains(key) == grid_cell
                && self.control_value_is_applied(*key, binding)
                && !self.paint_dirty.contains(key)
                && !parent_kind_changes.contains(&binding.node.parent)
                && self
                    .applied
                    .get(key)
                    .is_some_and(|(node, text)| *node == binding.node && *text == binding.text)
            {
                continue;
            }
            let id = if let Some(id) = self.ids.get(key) {
                *id
            } else {
                let id = self
                    .tree
                    .new_leaf(Style::default())
                    .map_err(|e| e.to_string())?;
                self.ids.insert(*key, id);
                self.kinds.insert(*key, kind);
                id
            };
            let mut style = Style {
                size: Size {
                    width: Dimension::percent(1.0),
                    height: Dimension::auto(),
                },
                flex_shrink: 0.0,
                ..Style::default()
            };
            if self
                .actor
                .view()
                .bindings
                .get(&binding.node.parent)
                .is_some_and(|parent| parent.node.kind == ROW)
            {
                style.size.width = Dimension::auto();
                style.min_size.width = LengthPercentageAuto::length(0.0);
                style.flex_basis = Dimension::length(0.0);
                style.flex_grow = 1.0;
                style.flex_shrink = 1.0;
            }
            if let Some(parent) = self.actor.view().bindings.get(&binding.node.parent)
                && parent.node.kind == SCROLL
                && parent.node.content_width > 0.0
            {
                style.size.width = Dimension::length(parent.node.content_width);
            }
            let presentation = binding.node.presentation;
            if presentation.width > 0.0 {
                style.size.width = Dimension::length(presentation.width);
                style.flex_basis = Dimension::auto();
                style.flex_grow = 0.0;
                style.flex_shrink = 1.0;
            }
            if presentation.max_width > 0.0 {
                style.max_size.width = LengthPercentageAuto::length(presentation.max_width);
            }
            let mut paint = Paint {
                text_color: Color::from_rgb8(28, 35, 48),
                font_size: 16.0,
                ..Paint::default()
            };
            if matches!(kind, COLUMN | ROW | DIALOG | CHILD) {
                style.display = Display::Flex;
                if kind == COLUMN && binding.node.height > 0.0 {
                    style.size.height = Dimension::length(binding.node.height);
                }
                if kind == DIALOG {
                    self.dialog_tops.insert(*key, 240.0);
                    style.position = Position::Absolute;
                    style.size.width = Dimension::percent(0.9);
                    style.inset.left = LengthPercentageAuto::percent(0.05);
                    style.inset.top = LengthPercentageAuto::length(240.0);
                    paint.background = Some(Color::WHITE);
                    paint.border = Some((1.0, Color::from_rgb8(170, 180, 195)));
                    paint.border_radii = CornerRadii::uniform(6.0);
                }
                if kind == ROW && self.badge_rows.contains(key) {
                    style.flex_wrap = FlexWrap::Wrap;
                }
                style.flex_direction = if kind == ROW {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                };
                style.gap = Size {
                    width: LengthPercentage::length(binding.node.gap),
                    height: LengthPercentage::length(binding.node.gap),
                };
                style.padding = Rect::length(binding.node.padding);
            } else if kind == SCROLL {
                if semantics::is_anchored_panel(binding.node.semantics.role) {
                    style.padding = Rect::length(binding.node.padding);
                    style.gap = Size {
                        width: LengthPercentage::length(binding.node.gap),
                        height: LengthPercentage::length(binding.node.gap),
                    };
                    paint.background = Some(Color::WHITE);
                    paint.border = Some((1.0, Color::from_rgb8(210, 216, 226)));
                    paint.border_radii = CornerRadii::uniform(10.0);
                }
                style.flex_direction = FlexDirection::Column;
                style.size.height = if binding.node.parent == 0 && binding.node.height == 0.0 {
                    Dimension::percent(1.0)
                } else {
                    Dimension::length(binding.node.height)
                };
                style.overflow = TaffyPoint {
                    x: Overflow::Scroll,
                    y: Overflow::Scroll,
                };
                let child = if let Some(id) = self.scroll_children.get(key) {
                    *id
                } else {
                    let id = self
                        .tree
                        .new_leaf(Style {
                            size: Size {
                                width: if binding.node.content_width > 0.0 {
                                    Dimension::length(binding.node.content_width)
                                } else {
                                    Dimension::percent(1.0)
                                },
                                height: Dimension::auto(),
                            },
                            flex_shrink: 0.0,
                            ..Style::default()
                        })
                        .unwrap();
                    self.scroll_children.insert(*key, id);
                    id
                };
                let mut child_style = self.tree.style(child).unwrap().clone();
                child_style.size.width = if binding.node.content_width > 0.0 {
                    Dimension::length(binding.node.content_width)
                } else {
                    Dimension::percent(1.0)
                };
                if self.tree.style(child).unwrap() != &child_style {
                    self.tree.set_style(child, child_style).unwrap();
                }
                let child_paint = Paint {
                    text: Some(Arc::from(binding.text.as_str())),
                    text_color: self
                        .palette(*key)
                        .map_or(Color::from_rgb8(70, 80, 96), |palette| {
                            theme::color(palette.text)
                        }),
                    font_size: 16.0,
                    ..Paint::default()
                };
                if self.tree.get_node_context(child).is_some_and(|old| {
                    old.text == child_paint.text && old.font_size == child_paint.font_size
                }) {
                    *self.tree.get_node_context_mut(child).unwrap() = child_paint;
                } else {
                    self.tree
                        .set_node_context(child, Some(child_paint))
                        .unwrap();
                }
                if !semantics::is_anchored_panel(binding.node.semantics.role) {
                    paint.background = Some(Color::from_rgb8(246, 247, 249));
                }
            } else if kind == CANVAS {
                style.size.height = Dimension::length(binding.node.height);
                style.overflow = TaffyPoint {
                    x: Overflow::Hidden,
                    y: Overflow::Hidden,
                };
                paint.background = Some(Color::WHITE);
                paint.border = Some((1.0, Color::from_rgb8(170, 180, 195)));
                paint.border_radii = CornerRadii::uniform(6.0);
                self.canvases.insert(
                    *key,
                    blockui_abi::canvas::CanvasScene::from_wire(&binding.text)
                        .map_err(str::to_owned)?,
                );
            } else if kind == PROGRESS {
                style.size.height = Dimension::length(binding.node.height);
            } else if kind == SELECT {
                let value = self
                    .actor
                    .widget_integer(*key)
                    .ok_or("missing select state")?;
                let options_changed = !self.selects.contains_key(key)
                    || self
                        .applied
                        .get(key)
                        .is_none_or(|(node, text)| node.kind != SELECT || text != &binding.text);
                let control = self.selects.entry(*key).or_default();
                if options_changed {
                    let (_, options) = blockui_abi::select::decode_control(&binding.text)
                        .map_err(str::to_owned)?;
                    control.project(options, value, binding.node.disabled == 0)?;
                } else {
                    control.project_value(value, binding.node.disabled == 0)?;
                }
                let label = control
                    .options()
                    .iter()
                    .find(|option| option.value == value)
                    .ok_or("missing select option")?
                    .label
                    .as_str();
                style.size.height = Dimension::length(binding.node.height);
                style.padding = Rect::length(8.0);
                style.padding.right = LengthPercentage::length(28.0);
                paint.text = Some(Arc::from(label));
                paint.background = Some(if binding.node.disabled != 0 {
                    Color::from_rgb8(246, 247, 249)
                } else {
                    Color::WHITE
                });
                paint.border = Some((1.0, Color::from_rgb8(170, 180, 195)));
                paint.border_radii = CornerRadii::uniform(6.0);
                paint.text_color = if binding.node.disabled != 0 {
                    Color::from_rgb8(109, 119, 130)
                } else {
                    Color::from_rgb8(28, 35, 48)
                };
            } else if kind == SLIDER {
                style.size.height = Dimension::length(binding.node.height);
                // Inset the track so endpoint thumbs stay inside the widget bounds.
                style.padding = Rect::length(8.0);
                let range = crate::native_slider::IntegerRange {
                    min: binding.node.range.min,
                    max: binding.node.range.max,
                    step: binding.node.range.step,
                };
                let value = i32::try_from(
                    self.actor
                        .widget_integer(*key)
                        .ok_or("missing slider state")?,
                )
                .map_err(|_| "slider value overflow")?;
                if let Some(slider) = self.sliders.get_mut(key) {
                    slider.project(range, value, binding.node.disabled != 0)?;
                } else {
                    let mut slider =
                        Box::new(crate::native_slider::NativeSlider::new(range, value)?);
                    slider.project(range, value, binding.node.disabled != 0)?;
                    self.sliders.insert(*key, slider);
                }
            } else if matches!(kind, ENTRY | TEXTAREA) {
                style.border = Rect::length(1.0);
                style.size.height = Dimension::length(binding.node.height);
                style.padding = Rect::length(10.0);
                paint.background = Some(if binding.node.disabled != 0 {
                    Color::from_rgb8(246, 247, 249)
                } else {
                    Color::WHITE
                });
                paint.border = Some((1.0, Color::from_rgb8(170, 180, 195)));
                paint.border_radii = CornerRadii::uniform(6.0);
                let newly_mounted = !self.editors.contains_key(key);
                let editor = self.editors.entry(*key).or_insert_with(|| {
                    let mut widget = Box::new(if kind == TEXTAREA {
                        TextInput::multiline()
                    } else {
                        TextInput::new()
                    });
                    widget.mounted();
                    widget.attribute_changed("color", "#1c2330");
                    widget
                });
                if binding.node.read_only != 0
                    && self
                        .applied
                        .get(key)
                        .is_some_and(|(old, _)| old.read_only == 0)
                {
                    editor.handle_event(&UiEvent::Ime(shell::ImeEvent::Disabled));
                    if self.composing_editor == Some(*key) {
                        self.composing_editor = None;
                    }
                    if self.paste_target == Some(*key) {
                        self.paste_target = None;
                    }
                }
                editor.attribute_changed(
                    "disabled",
                    if binding.node.disabled != 0 {
                        "true"
                    } else {
                        "false"
                    },
                );
                editor.attribute_changed(
                    "color",
                    if binding.node.disabled != 0 {
                        "#6d7782"
                    } else {
                        "#1c2330"
                    },
                );
                editor.attribute_changed(
                    "readOnly",
                    if binding.node.read_only != 0 {
                        "true"
                    } else {
                        "false"
                    },
                );
                if editor.current_value() != Some(binding.text.as_str()) {
                    editor.attribute_changed("value", &binding.text);
                }
                if newly_mounted && self.actor.view().renderer[key].selection_saved {
                    let (anchor, head) = self.actor.view().renderer[key].selection;
                    let boundary = |offset: i32| {
                        let mut position = 0;
                        for character in binding.text.chars() {
                            let next = position + character.len_utf16();
                            if next > offset.max(0) as usize {
                                break;
                            }
                            position = next;
                        }
                        position
                    };
                    let config = serde_json::json!({"selection": {
                        "anchor": boundary(anchor), "head": boundary(head)
                    }})
                    .to_string();
                    if let Err(error) = editor.config_changed(&config) {
                        self.diagnostic = error;
                    }
                }
            } else {
                style.size.height = Dimension::length(binding.node.height);
                paint.text = Some(Arc::from(binding.text.as_str()));
                if kind == CHECKBOX {
                    style.padding = Rect::length(6.0);
                    paint.text_color = if binding.node.disabled != 0 {
                        Color::from_rgb8(109, 119, 130)
                    } else {
                        Color::from_rgb8(28, 35, 48)
                    };
                }
                if kind == BUTTON {
                    if presentation.font_size > 0.0 {
                        paint.font_size = presentation.font_size;
                    }
                    style.padding = Rect {
                        left: LengthPercentage::length(8.0),
                        right: LengthPercentage::length(8.0),
                        top: LengthPercentage::length(4.0),
                        bottom: LengthPercentage::length(4.0),
                    };
                    // Reserve border geometry in every variant/state to avoid
                    // shifting labels when enabled or pressed state changes.
                    style.border = Rect::length(1.0);
                    if grid_cell {
                        style.border = Rect::length(1.0);
                        paint.background = Some(Color::WHITE);
                        paint.text_color = if binding.node.disabled != 0 {
                            Color::from_rgb8(109, 119, 130)
                        } else {
                            Color::from_rgb8(28, 35, 48)
                        };
                        paint.border = Some((1.0, Color::from_rgb8(210, 216, 225)));
                        paint.border_radii = CornerRadii::uniform(0.0);
                    } else if binding.node.button_variant == 2 {
                        let disabled = binding.node.disabled != 0;
                        paint.background = Some(if disabled {
                            Color::from_rgb8(240, 243, 247)
                        } else {
                            Color::from_rgb8(255, 245, 245)
                        });
                        paint.text_color = if disabled {
                            Color::from_rgb8(109, 119, 130)
                        } else {
                            Color::from_rgb8(166, 32, 40)
                        };
                        paint.border = Some((
                            1.0,
                            if disabled {
                                Color::from_rgb8(210, 216, 225)
                            } else {
                                Color::from_rgb8(239, 174, 174)
                            },
                        ));
                        paint.border_radii = CornerRadii::uniform(6.0);
                    } else if binding.node.button_variant == 1 {
                        style.border = Rect::length(1.0);
                        paint.background = Some(Color::from_rgb8(240, 243, 247));
                        paint.text_color = if binding.node.disabled != 0 {
                            Color::from_rgb8(109, 119, 130)
                        } else {
                            Color::from_rgb8(28, 35, 48)
                        };
                        paint.border = Some((1.0, Color::from_rgb8(210, 216, 225)));
                        paint.border_radii = CornerRadii::uniform(6.0);
                    } else {
                        let disabled = binding.node.disabled != 0;
                        paint.background = Some(if disabled {
                            Color::from_rgb8(240, 243, 247)
                        } else {
                            Color::from_rgb8(31, 88, 155)
                        });
                        paint.text_color = if disabled {
                            Color::from_rgb8(109, 119, 130)
                        } else {
                            Color::WHITE
                        };
                        if disabled {
                            style.border = Rect::length(1.0);
                            paint.border = Some((1.0, Color::from_rgb8(210, 216, 225)));
                        }
                        paint.border_radii = CornerRadii::uniform(6.0);
                    }
                }
            }
            if kind == LABEL {
                if binding.node.semantics.role == semantics::BADGE {
                    paint.font_size = 12.0;
                }
                if presentation.font_size > 0.0 {
                    paint.font_size = presentation.font_size;
                }
                // Text is measured by the shared shaping/layout path. Keep the
                // single-line rhythm as a minimum while allowing wrapped labels
                // to grow, instead of clipping them to a fixed node height.
                paint.wrap_text = true;
                style.size.height = Dimension::auto();
                style.min_size.height =
                    LengthPercentageAuto::length(binding.node.height.max(paint.font_size * 1.4));
                paint.text_color = match presentation.text_tone {
                    1 => Color::from_rgb8(100, 116, 139),
                    2 => Color::from_rgb8(31, 88, 155),
                    3 => Color::from_rgb8(166, 32, 40),
                    _ => paint.text_color,
                };
                if presentation.text_tone == 3 && binding.text.is_empty() {
                    style.display = taffy::Display::None;
                }
            }
            if matches!(kind, COLUMN | ROW) {
                paint.background = match presentation.surface {
                    1 => Some(Color::from_rgb8(246, 247, 249)),
                    2 => Some(Color::WHITE),
                    _ => paint.background,
                };
                paint.border_radii = CornerRadii::uniform(presentation.radius);
                if presentation.surface == 2 {
                    // Panel recipes share a subtle border; account for its width
                    // in layout so content and rounded paint never overlap.
                    style.border = Rect::length(1.0);
                    paint.border = Some((1.0, Color::from_rgb8(226, 230, 236)));
                }
            }
            if matches!(
                binding.node.semantics.role,
                semantics::HORIZONTAL_SEPARATOR | semantics::VERTICAL_SEPARATOR
            ) {
                paint.background = Some(Color::from_rgb8(226, 230, 236));
                style.flex_grow = 0.0;
                style.flex_shrink = 0.0;
                if binding.node.semantics.role == semantics::VERTICAL_SEPARATOR {
                    style.size.width = Dimension::length(1.0);
                    style.min_size.width = LengthPercentageAuto::length(1.0);
                    style.flex_basis = Dimension::length(1.0);
                } else {
                    style.size.height = Dimension::length(1.0);
                }
            }
            if binding.node.semantics.role == semantics::BADGE {
                style.size.width = Dimension::auto();
                style.max_size.width = LengthPercentageAuto::percent(1.0);
                style.flex_basis = Dimension::auto();
                style.flex_grow = 0.0;
                style.flex_shrink = 1.0;
                style.align_self = Some(AlignSelf::FLEX_START);
                style.padding = Rect {
                    left: LengthPercentage::length(8.0),
                    right: LengthPercentage::length(8.0),
                    top: LengthPercentage::length(2.0),
                    bottom: LengthPercentage::length(2.0),
                };
                style.border = Rect::length(1.0);
                let (background, foreground, border) = match binding.node.button_variant {
                    1 => (
                        Color::from_rgb8(240, 243, 247),
                        Color::from_rgb8(28, 35, 48),
                        Color::from_rgb8(240, 243, 247),
                    ),
                    2 => (
                        Color::WHITE,
                        Color::from_rgb8(28, 35, 48),
                        Color::from_rgb8(210, 216, 225),
                    ),
                    3 => (
                        Color::from_rgb8(166, 32, 40),
                        Color::WHITE,
                        Color::from_rgb8(166, 32, 40),
                    ),
                    _ => (
                        Color::from_rgb8(28, 35, 48),
                        Color::WHITE,
                        Color::from_rgb8(28, 35, 48),
                    ),
                };
                paint.background = Some(background);
                if presentation.text_tone == 0 {
                    paint.text_color = foreground;
                }
                paint.border = Some((1.0, border));
                paint.border_radii = CornerRadii::uniform(12.0);
            }
            if binding.node.semantics.role == semantics::TOOLTIP {
                paint.background = Some(Color::from_rgb8(28, 35, 48));
                paint.text_color = Color::WHITE;
                paint.border_radii = CornerRadii::uniform(6.0);
                style.padding = Rect::length(8.0);
                style.flex_shrink = 0.0;
            }
            if binding.node.semantics.hidden != 0 {
                style.display = Display::None;
            }
            self.style_choice(binding, &mut style, &mut paint);
            if semantics::is_menu_item(binding.node.semantics.role) {
                paint.background = Some(Color::WHITE);
                paint.border = None;
                paint.border_radii = CornerRadii::uniform(4.0);
                paint.font_size = 14.0;
                if binding.node.semantics.role == semantics::MENU_CHECKBOX {
                    paint.text = Some(Arc::from(format!(
                        "{} {}",
                        if binding.node.semantics.selected != 0 {
                            "☑"
                        } else {
                            "☐"
                        },
                        binding.text
                    )));
                }
            }
            if binding.node.semantics.role == semantics::MENU_RADIO {
                paint.text = Some(Arc::from(format!(
                    "{} {}",
                    if binding.node.semantics.selected != 0 {
                        "◉"
                    } else {
                        "○"
                    },
                    binding.text
                )));
            }
            if binding.node.semantics.role == semantics::MENU_LABEL {
                paint.font_size = 12.0;
                paint.text_color = Color::from_rgb8(100, 112, 132);
                style.padding = Rect {
                    left: LengthPercentage::length(8.0),
                    right: LengthPercentage::length(8.0),
                    top: LengthPercentage::length(4.0),
                    bottom: LengthPercentage::length(4.0),
                };
            }
            if binding.node.semantics.role == semantics::SUBMENU_TRIGGER {
                paint.text = Some(Arc::from(format!("{}  ›", binding.text)));
            }
            self.style_disclosure(binding, &mut style, &mut paint);
            if kind == BUTTON && self.toast_owner(*key).is_some() {
                style.size.height = Dimension::auto();
                style.min_size.height = LengthPercentageAuto::length(36.0);
                paint.font_size = 12.0;
                paint.wrap_text = true;
            }
            self.style_theme(binding, &mut style, &mut paint);
            if presentation.fill_height != 0 {
                style.min_size.height = LengthPercentageAuto::length(0.0);
                style.flex_shrink = 1.0;
                if self
                    .actor
                    .view()
                    .bindings
                    .get(&binding.node.parent)
                    .is_some_and(|p| p.node.kind == COLUMN)
                {
                    style.size.height = Dimension::auto();
                    style.flex_basis = Dimension::length(0.0);
                    style.flex_grow = 1.0;
                } else {
                    style.size.height = Dimension::percent(1.0);
                }
            }
            if let Some(editor) = self.editors.get_mut(key) {
                editor.style_changed(&shell::WidgetStyle::from(&paint));
            }
            let accent = self
                .palette(*key)
                .map_or(Color::from_rgb8(31, 88, 155), |p| theme::color(p.accent));
            if let Some(slider) = self.sliders.get_mut(key) {
                let mut widget_style = shell::WidgetStyle::from(&paint);
                widget_style.color = accent;
                slider.style_changed(&widget_style);
            }
            self.style_icon(binding, &mut style, &mut paint);
            let base = motion::HoverPaint::capture(&paint);
            let mut hovered_paint = paint.clone();
            self.apply_hover_style(*key, grid_cell, &mut hovered_paint);
            self.hover_paints
                .insert(*key, (base, motion::HoverPaint::capture(&hovered_paint)));
            self.apply_hover(*key, grid_cell, &mut paint);
            if self.focus == Some(*key)
                && self.window_focused
                && (matches!(kind, BUTTON | CHECKBOX | SELECT | SLIDER | ENTRY | TEXTAREA)
                    || binding.node.semantics.role == semantics::TAB_PANEL)
            {
                paint.outline_width = 2.0;
                paint.outline_offset = if grid_cell { 0.0 } else { 2.0 };
                paint.outline_color = Some(
                    self.palette(*key)
                        .map_or(Color::from_rgb8(31, 88, 155), |p| theme::color(p.focus)),
                );
            }
            if matches!(kind, BUTTON | CHECKBOX)
                && binding.node.disabled == 0
                && (self.keyboard_pressed == Some(*key)
                    || self.pointer_pressed_visual == Some(*key))
            {
                paint.background = Some(if binding.node.semantics.expanded != 0 {
                    Color::from_rgb8(238, 241, 246)
                } else if binding.node.semantics.role == semantics::TAB {
                    Color::from_rgb8(225, 235, 249)
                } else if kind == BUTTON && !grid_cell && binding.node.button_variant == 2 {
                    Color::from_rgb8(255, 225, 225)
                } else if kind == CHECKBOX || grid_cell || binding.node.button_variant == 1 {
                    Color::from_rgb8(225, 235, 249)
                } else {
                    Color::from_rgb8(21, 62, 112)
                });
            }
            if binding.node.disabled == 0
                && (self.keyboard_pressed == Some(*key)
                    || self.pointer_pressed_visual == Some(*key))
            {
                self.theme_interaction(*key, &mut paint, true);
            }
            if let Some((width, _)) = paint.border {
                style.border = Rect::length(width);
            }
            let paint_update = self.tree.style(id).is_ok_and(|old| *old == style)
                && self.tree.get_node_context(id).is_some_and(|old| {
                    old.text == paint.text
                        && old.font_size == paint.font_size
                        && old.font_weight == paint.font_weight
                        && old.font_italic == paint.font_italic
                        && old.font_family == paint.font_family
                        && old.letter_spacing == paint.letter_spacing
                        && old.line_height == paint.line_height
                        && old.wrap_text == paint.wrap_text
                        && old.text_ellipsis == paint.text_ellipsis
                        && old.text_max_lines == paint.text_max_lines
                        && old.intrinsic_size == paint.intrinsic_size
                });
            if paint_update {
                // Presentation changes with identical layout and measurement inputs
                // only replace paint; node metadata need not be identical.
                *self
                    .tree
                    .get_node_context_mut(id)
                    .expect("retained theme context") = paint;
            } else {
                self.tree.set_style(id, style).map_err(|e| e.to_string())?;
                self.tree
                    .set_node_context(id, Some(paint))
                    .map_err(|e| e.to_string())?;
            }
            self.applied
                .insert(*key, (binding.node, binding.text.clone()));
            if grid_cell {
                self.painted_grid_cells.insert(*key);
            } else {
                self.painted_grid_cells.remove(key);
            }
        }
        if layout_children_changed {
            let mut layout_children: BTreeMap<u64, Vec<NodeId>> = BTreeMap::new();
            for child in &self.actor.view().order {
                let node = self.actor.view().bindings[child].node;
                if !matches!(
                    node.semantics.role,
                    semantics::TOAST_VIEWPORT
                        | semantics::POPOVER_PANEL
                        | semantics::MENU_PANEL
                        | semantics::TOOLTIP
                ) {
                    layout_children
                        .entry(node.parent)
                        .or_default()
                        .push(self.ids[child]);
                }
            }
            for key in &self.actor.view().order {
                if !matches!(
                    self.actor.view().bindings[key].node.kind,
                    COLUMN | ROW | DIALOG | CHILD | SCROLL
                ) {
                    continue;
                }
                let mut children = layout_children.remove(key).unwrap_or_default();
                if self.actor.view().bindings[key].node.kind == SCROLL
                    && (!self.actor.view().bindings[key].text.is_empty() || children.is_empty())
                {
                    children.insert(0, self.scroll_children[key]);
                }
                let id = self.ids[key];
                if self.tree.children(id).map_err(|e| e.to_string())? != children {
                    self.tree
                        .set_children(id, &children)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        let modal = self
            .actor
            .view()
            .order
            .iter()
            .rev()
            .copied()
            .find(|key| self.actor.view().bindings[key].node.kind == DIALOG);
        if modal != self.modal {
            let opening = modal.is_some();
            self.modal = modal;
            if opening {
                self.set_focus(None);
                self.focus_next(false);
            } else {
                let opener = self.modal_opener.take().filter(|key| {
                    self.actor.view().bindings.get(key).is_some_and(|binding| {
                        binding.node.disabled == 0
                            && matches!(
                                binding.node.kind,
                                BUTTON | CHECKBOX | SELECT | SLIDER | ENTRY | TEXTAREA
                            )
                    })
                });
                self.set_focus(opener);
                if opener.is_none() {
                    // Navigation may remove the dialog opener. Give keyboard
                    // users a usable destination in the current projection.
                    self.focus_next(false);
                }
            }
        }
        if let Some(target) = returning_focus.filter(|key| self.in_modal(*key)) {
            self.set_focus(Some(target));
        }
        if let Some((grid, index)) = self.suspended_grid_focus {
            if self.actor.view().bindings.get(&grid).is_none_or(|binding| {
                binding.node.kind != SCROLL || binding.node.navigate_action == 0
            }) {
                self.suspended_grid_focus = None;
            } else if let Some(target) = self.grid_target(grid, index)
                && self.actor.view().bindings[&target].node.disabled == 0
            {
                self.set_focus(Some(target));
            }
        }
        if let Some(key) = closed_popover_focus.filter(|key| {
            self.actor
                .view()
                .bindings
                .get(key)
                .is_some_and(|b| b.node.disabled == 0)
                && self.in_modal(*key)
        }) {
            self.set_focus(Some(key));
        }
        for panel in opening_popovers {
            if self.actor.view().bindings[&panel].node.semantics.role == semantics::MENU_PANEL {
                let items = self.menu_items(panel);
                let last = self.menu_open_last.take() == Some(panel);
                let target = self.menus.entry(panel).or_default().open(&items, last);
                self.set_focus(target.or(Some(panel)));
                self.pending_reveal = target;
                continue;
            }
            let target = self.actor.view().order.iter().copied().find(|key| {
                self.popover_contains(panel, *key)
                    && self.actor.view().bindings[key].node.disabled == 0
                    && matches!(
                        self.actor.view().bindings[key].node.kind,
                        BUTTON | CHECKBOX | ENTRY | TEXTAREA | SELECT | SLIDER
                    )
            });
            if let Some(target) = target {
                self.set_focus(Some(target));
            }
        }
        self.reconcile_menus();
        self.paint_dirty.clear();
        if self.focus != focus_before_sync {
            self.paint_dirty
                .extend(focus_before_sync.into_iter().chain(self.focus));
        }
        self.applied_order.clone_from(&self.actor.view().order);
        Ok(())
    }
    pub fn replace(&mut self, path: &Path) -> Result<(), String> {
        self.persist_native_renderer();
        self.actor.reload(path)?;
        self.sync()
    }
    fn sync_editor_values(&mut self, tcx: &mut TextContext) {
        for (key, editor) in &mut self.editors {
            editor.prepare_for_event(tcx);
            if let Some(value) = editor.current_value() {
                // A host action may have updated the slot since the last native sync.
                // Only a change from the value actually applied to the editor is an edit.
                if !self
                    .applied
                    .get(key)
                    .is_some_and(|(_, applied)| applied != value)
                {
                    continue;
                }
                if let Err(error) = self.actor.set_widget_text(*key, value) {
                    self.diagnostic = error;
                }
            }
        }
    }
    fn append_select_popup(&mut self, nodes: &mut Vec<PlacedNode>, width: u32, height: u32) {
        let previous = self.select_popup.take();
        let Some(owner) = self.focus.filter(|key| {
            self.selects
                .get(key)
                .is_some_and(|control| control.is_open())
        }) else {
            return;
        };
        let Some(anchor) = nodes
            .iter()
            .find(|node| node.node_id == self.ids[&owner])
            .cloned()
        else {
            self.selects.get_mut(&owner).unwrap().close();
            self.select_pressed = None;
            return;
        };
        let control = &self.selects[&owner];
        let active = control
            .options()
            .iter()
            .position(|option| Some(option.value) == control.active())
            .unwrap_or(0);
        let Some(layout) = crate::select_popup::PopupLayout::new(
            anchor.rect,
            [width as f32, height as f32],
            control.options().len(),
            active,
        ) else {
            self.selects.get_mut(&owner).unwrap().close();
            self.select_pressed = None;
            return;
        };
        let layout = if let Some((_, old_layout, _)) =
            previous.filter(|(old_owner, _, _)| *old_owner == owner)
        {
            layout.retain_window(old_layout.start, control.options().len(), active)
        } else {
            layout
        };
        while self.select_rows.len() < layout.end - layout.start {
            self.select_rows.push(
                self.tree
                    .new_leaf(Style::default())
                    .expect("select overlay leaf"),
            );
        }
        let mut values = Vec::new();
        for index in layout.start..layout.end {
            let option = &control.options()[index];
            let rect = layout.row_rect(index).unwrap();
            let mut row = anchor.clone();
            row.node_id = self.select_rows[index - layout.start];
            row.parent_node_id = None;
            row.depth = 0;
            row.rect = rect;
            row.content_origin = [rect[0] + 8.0, rect[1] + 6.0];
            row.content_size = [(rect[2] - rect[0] - 16.0).max(0.0), 20.0];
            row.clip = Some(rect);
            row.clip_depth = None;
            row.clip_radius = 0.0;
            row.clip_radii = CornerRadii::default();
            row.own_clip = Some(rect);
            row.own_clip_radius = 0.0;
            row.own_clip_radii = CornerRadii::default();
            row.border_widths = [
                if index == layout.start { 1.0 } else { 0.0 },
                1.0,
                if index + 1 == layout.end { 1.0 } else { 0.0 },
                1.0,
            ];
            row.scroll = Default::default();
            row.paint = Paint {
                text: Some(Arc::from(option.label.as_str())),
                font_size: 16.0,
                text_color: if option.enabled {
                    Color::from_rgb8(28, 35, 48)
                } else {
                    Color::from_rgb8(109, 119, 130)
                },
                background: Some(if Some(option.value) == control.active() {
                    Color::from_rgb8(225, 235, 249)
                } else {
                    Color::WHITE
                }),
                border: Some((1.0, Color::from_rgb8(170, 180, 195))),
                ..Paint::default()
            };
            if let Some(p) = self.palette(owner) {
                row.paint.font_size = 14.0;
                row.paint.text_color =
                    theme::color(if option.enabled { p.text } else { p.text_muted });
                row.paint.background =
                    Some(theme::color(if Some(option.value) == control.active() {
                        p.hover
                    } else {
                        p.panel
                    }));
                row.paint.border = Some((1.0, theme::color(p.border)));
            }
            nodes.push(row);
            values.push(option.value);
        }
        self.select_popup = Some((owner, layout, values));
    }
    fn hit(&self, point: shell::Point) -> Option<u64> {
        self.actor
            .view()
            .order
            .iter()
            .rev()
            .filter(|key| self.toast_owner(**key).is_some() || self.popover_owner(**key).is_some())
            .chain(self.actor.view().order.iter().rev().filter(|key| {
                self.toast_owner(**key).is_none() && self.popover_owner(**key).is_none()
            }))
            .find(|key| {
                if matches!(
                    self.actor.view().bindings[key].node.semantics.role,
                    semantics::TOAST_VIEWPORT | semantics::TOOLTIP
                ) || !self.in_modal(**key)
                {
                    return false;
                }
                // A scroll observer can publish new rows between paint frames.
                // Unmeasured nodes cannot be pointer targets yet.
                let Some(&id) = self.ids.get(key) else {
                    return false;
                };
                self.placed.iter().any(|node| {
                    node.node_id == id
                        && point.x >= node.rect[0] as f64
                        && point.x <= node.rect[2] as f64
                        && point.y >= node.rect[1] as f64
                        && point.y <= node.rect[3] as f64
                        && node.clip.is_none_or(|clip| {
                            point.x >= f64::from(clip[0])
                                && point.x <= f64::from(clip[2])
                                && point.y >= f64::from(clip[1])
                                && point.y <= f64::from(clip[3])
                        })
                })
            })
            .copied()
    }
}
impl<M: NativeModel> Drop for NativeSource<M> {
    fn drop(&mut self) {
        self.actor_watches.clear();
        self.implementation_watches.clear();
        self.watcher.take(); // stop/join producer before draining snapshots
        if let Some(receiver) = &self.builds {
            for result in receiver.try_iter() {
                if let Some(path) = result.artifact {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}
impl<M: NativeModel> FrameSource for NativeSource<M> {
    fn set_semantics_enabled(&mut self, enabled: bool) {
        self.semantics_enabled = enabled;
        if enabled {
            self.refresh_semantics();
        } else {
            self.semantics = None;
        }
    }
    fn semantic_snapshot(&self) -> Option<Arc<shell::SemanticSnapshot>> {
        self.semantics.clone()
    }
    fn handle_semantic_action(&mut self, action: shell::SemanticAction) -> bool {
        self.persist_native_renderer();
        if !self.semantics_enabled {
            return false;
        }
        let target = match action {
            shell::SemanticAction::Click { target }
            | shell::SemanticAction::Focus { target }
            | shell::SemanticAction::Blur { target }
            | shell::SemanticAction::ScrollIntoView { target } => target,
        };
        if !self.in_modal(target) {
            return false;
        }
        if let Some((owner, Some(value))) = self
            .select_semantic_ids
            .iter()
            .find_map(|(key, id)| (*id == target).then_some(*key))
        {
            if !self
                .selects
                .get(&owner)
                .is_some_and(|control| control.is_open())
            {
                return false;
            }
            match action {
                shell::SemanticAction::Click { .. } => {
                    let valid = self.selects[&owner]
                        .options()
                        .iter()
                        .any(|option| option.value == value && option.enabled);
                    if !valid {
                        return false;
                    }
                    let request = self.selects.get_mut(&owner).unwrap().choose(value);
                    if let Some(value) = request
                        && let Err(error) = self.actor.set_widget_select(owner, value)
                    {
                        self.diagnostic = error;
                    }
                    self.select_popup = None;
                    self.select_semantic_ids.clear();
                    self.select_pressed = None;
                    self.paint_dirty.insert(owner);
                    return true;
                }
                shell::SemanticAction::Focus { .. } => {
                    if !self.selects.get_mut(&owner).unwrap().highlight(value)
                        && !self.selects[&owner]
                            .options()
                            .iter()
                            .any(|option| option.value == value && option.enabled)
                    {
                        return false;
                    }
                    self.paint_dirty.insert(owner);
                    return true;
                }
                _ => return false,
            }
        }
        let Some(binding) = self.actor.view().bindings.get(&target) else {
            return false;
        };
        let node = binding.node;
        if node.disabled != 0 || node.semantics.hidden != 0 {
            return false;
        }
        match action {
            shell::SemanticAction::Click { .. } if node.kind == SELECT && node.disabled == 0 => {
                let Some(control) = self.selects.get_mut(&target) else {
                    return false;
                };
                let request = control.key(crate::select_model::Key::Activate, false);
                self.select_pressed = None;
                if let Some(value) = request
                    && let Err(error) = self.actor.set_widget_select(target, value)
                {
                    self.diagnostic = error;
                }
                self.set_focus(Some(target));
                self.paint_dirty.insert(target);
            }
            shell::SemanticAction::Click { .. } if matches!(node.kind, BUTTON | CHECKBOX) => {
                if let Err(error) = self.actor.dispatch_widget(target) {
                    self.diagnostic = error;
                    return false;
                }
                if let Err(error) = self.sync() {
                    self.diagnostic = error;
                    return false;
                }
            }
            shell::SemanticAction::Focus { .. }
                if matches!(
                    node.kind,
                    BUTTON | CHECKBOX | SELECT | SLIDER | ENTRY | TEXTAREA
                ) || node.semantics.role == semantics::TAB_PANEL =>
            {
                self.set_focus(Some(target));
                self.reveal_widget(target);
            }
            shell::SemanticAction::Blur { .. } if self.focus == Some(target) => {
                self.set_focus(None)
            }
            shell::SemanticAction::ScrollIntoView { .. } => {
                if !self.reveal_widget(target) {
                    return false;
                }
            }
            _ => return false,
        }
        self.refresh_semantics();
        true
    }
    fn set_device_scale(&mut self, scale: f64) {
        self.scale = scale;
    }
    fn prepare_for_event(&mut self, tcx: &mut TextContext) {
        self.sync_editor_values(tcx);
    }
    fn build_frame(&mut self, tcx: &mut TextContext, width: u32, height: u32) -> Vec<PlacedNode> {
        self.poll_menu_hover();
        // Winit deadline wakes request a redraw, not a proxy wake. Advance due
        // runtime services here so their deadlines cannot keep scheduling frames.
        if self
            .actor
            .async_deadline()
            .is_some_and(|deadline| deadline <= Instant::now())
        {
            self.poll_async();
        }
        self.sync_editor_values(tcx);
        if let Err(error) = self.sync() {
            self.diagnostic = error;
        }
        // Opening/closing an overlay may change focus after contexts were
        // projected. Repaint those contexts before publishing this frame.
        if !self.paint_dirty.is_empty()
            && let Err(error) = self.sync()
        {
            self.diagnostic = error;
        }
        let root_key = self
            .actor
            .view()
            .order
            .iter()
            .find(|key| self.actor.view().bindings[key].node.parent == 0)
            .unwrap();
        let mut root = self.ids[root_key];
        let mut offsets = self
            .actor
            .view()
            .renderer
            .iter()
            .filter_map(|(key, state)| {
                self.ids
                    .get(key)
                    .map(|id| (*id, [state.scroll_x as f32, state.scroll as f32]))
            })
            .collect::<HashMap<_, _>>();
        let mut nodes = shell::layout::compute_and_walk_with_scroll(
            &mut self.tree,
            root,
            width as f32,
            height as f32,
            tcx,
            &offsets,
        );
        // Floating panels are measured independently below; revealing against
        // their authored placeholder would use the wrong viewport and range.
        let overlay_reveal = self
            .pending_reveal
            .filter(|target| self.popover_owner(*target).is_some());
        if overlay_reveal.is_some() {
            self.pending_reveal = None;
        }
        // Newly materialized grid destinations have no measured rectangle until
        // this layout pass. Reveal only now, then apply any viewport projection.
        if let Some(target) = self.pending_reveal.take() {
            self.reveal_widget(target);
            if let Err(error) = self.sync() {
                self.diagnostic = error;
            }
            let current_root = self
                .actor
                .view()
                .order
                .iter()
                .find(|key| self.actor.view().bindings[key].node.parent == 0)
                .unwrap();
            root = self.ids[current_root];
            offsets = self
                .actor
                .view()
                .renderer
                .iter()
                .filter_map(|(key, state)| {
                    self.ids
                        .get(key)
                        .map(|id| (*id, [state.scroll_x as f32, state.scroll as f32]))
                })
                .collect();
            nodes = shell::layout::compute_and_walk_with_scroll(
                &mut self.tree,
                root,
                width as f32,
                height as f32,
                tcx,
                &offsets,
            );
        }
        // Shell flattening consumes offsets as supplied. Normalize retained
        // state against newly measured content, then re-flatten if it changed.
        let mut clamped_scrolls = Vec::new();
        for key in self.actor.view().order.clone() {
            if self.actor.view().bindings[&key].node.kind != SCROLL {
                continue;
            }
            let id = self.ids[&key];
            if let Some(placed) = nodes.iter().find(|node| node.node_id == id) {
                let offset = [
                    placed.scroll.offset[0].clamp(0.0, placed.scroll.range[0]),
                    placed.scroll.offset[1].clamp(0.0, placed.scroll.range[1]),
                ];
                let state = self.actor.view_mut().renderer.get_mut(&key).unwrap();
                state.scroll_x = f64::from(offset[0]);
                state.scroll = f64::from(offset[1]);
                if placed.scroll.offset != offset {
                    offsets.insert(id, offset);
                    clamped_scrolls.push(key);
                }
            }
        }
        if !clamped_scrolls.is_empty() {
            for key in clamped_scrolls {
                self.notify_scroll(key);
            }
            // Observers can replace the visible projection. Do not flatten old
            // IDs after dispatch; synchronize and measure the current tree.
            if let Err(error) = self.sync() {
                self.diagnostic = error;
            }
            let current_root = self
                .actor
                .view()
                .order
                .iter()
                .find(|key| self.actor.view().bindings[key].node.parent == 0)
                .unwrap();
            root = self.ids[current_root];
            offsets = self
                .actor
                .view()
                .renderer
                .iter()
                .filter_map(|(key, state)| {
                    self.ids
                        .get(key)
                        .map(|id| (*id, [state.scroll_x as f32, state.scroll as f32]))
                })
                .collect();
            nodes = shell::layout::compute_and_walk_with_scroll(
                &mut self.tree,
                root,
                width as f32,
                height as f32,
                tcx,
                &offsets,
            );
        }
        // Report measured ports once per changed size/mount/action. Observers
        // may replace projections; remeasure current IDs before painting.
        self.observed_viewports.retain(|key, _| {
            self.ids.contains_key(key)
                && self
                    .actor
                    .view()
                    .bindings
                    .get(key)
                    .is_some_and(|binding| binding.node.viewport_action != 0)
        });
        let changes: Vec<_> = self
            .actor
            .view()
            .order
            .iter()
            .filter_map(|key| {
                let node = self.actor.view().bindings[key].node;
                if node.kind != SCROLL || node.viewport_action == 0 {
                    return None;
                }
                let id = self.ids[key];
                let placed = nodes.iter().find(|placed| placed.node_id == id)?;
                let port = placed.scroll.port;
                let size = [(port[2] - port[0]).max(0.0), (port[3] - port[1]).max(0.0)];
                let record = (id, node.viewport_action, size);
                (self.observed_viewports.get(key) != Some(&record)).then_some((*key, record))
            })
            .collect();
        let mut notified = false;
        for (key, record) in changes {
            if self.ids.get(&key) != Some(&record.0)
                || self
                    .actor
                    .view()
                    .bindings
                    .get(&key)
                    .is_none_or(|binding| binding.node.viewport_action != record.1)
            {
                continue;
            }
            self.observed_viewports.insert(key, record);
            notified = true;
            if let Err(error) = self.actor.observe_viewport(
                key,
                blockui_abi::LocalPoint {
                    x: record.2[0],
                    y: record.2[1],
                },
            ) {
                self.diagnostic = error;
            }
        }
        if notified {
            if let Err(error) = self.sync() {
                self.diagnostic = error;
            }
            let current_root = self
                .actor
                .view()
                .order
                .iter()
                .find(|key| self.actor.view().bindings[key].node.parent == 0)
                .unwrap();
            root = self.ids[current_root];
            offsets = self
                .actor
                .view()
                .renderer
                .iter()
                .filter_map(|(key, state)| {
                    self.ids
                        .get(key)
                        .map(|id| (*id, [state.scroll_x as f32, state.scroll as f32]))
                })
                .collect();
            nodes = shell::layout::compute_and_walk_with_scroll(
                &mut self.tree,
                root,
                width as f32,
                height as f32,
                tcx,
                &offsets,
            );
        }
        // Viewport callbacks may change the parent layout or recreate the modal
        // recipe. Center only after the final responsive projection is laid out.
        if let Some(modal) = self.modal
            && let Some(id) = self.ids.get(&modal).copied()
            && let Some(node) = nodes.iter().find(|node| node.node_id == id)
        {
            let desired = ((height as f32 - (node.rect[3] - node.rect[1])) * 0.5).max(16.0);
            let delta = desired - node.rect[1];
            if delta.abs() > 0.01 {
                let top = self.dialog_tops[&modal] + delta;
                let mut style = self.tree.style(id).unwrap().clone();
                style.inset.top = LengthPercentageAuto::length(top);
                self.tree.set_style(id, style).unwrap();
                self.dialog_tops.insert(modal, top);
                nodes = shell::layout::compute_and_walk_with_scroll(
                    &mut self.tree,
                    root,
                    width as f32,
                    height as f32,
                    tcx,
                    &offsets,
                );
            }
        }
        self.append_toasts(&mut nodes, tcx, width, height, overlay_reveal);
        self.reconcile_tooltip();
        self.append_tooltip(&mut nodes, tcx, width, height);
        for node in &mut nodes {
            if let Some(modal) = self.modal
                && self.ids.get(&modal) == Some(&node.node_id)
            {
                use shell::PaintScene;
                use vello_common::{
                    kurbo::{Affine, Rect},
                    peniko::Fill,
                };
                let [x, y] = node.content_origin;
                let padding = f64::from(self.actor.view().bindings[&modal].node.padding);
                let mut paint = shell::widget::PaintContext::new(
                    node.content_size[0],
                    node.content_size[1],
                    self.scale,
                    tcx,
                );
                paint.scene_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::from_rgba8(15, 23, 42, 80),
                    None,
                    &Rect::new(
                        -f64::from(x),
                        -f64::from(y),
                        f64::from(width) - f64::from(x),
                        f64::from(height) - f64::from(y),
                    ),
                );
                paint.scene_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    node.paint.background.unwrap_or(Color::WHITE),
                    None,
                    &Rect::new(
                        -padding,
                        -padding,
                        f64::from(node.content_size[0]) + padding,
                        f64::from(node.content_size[1]) + padding,
                    )
                    .to_rounded_rect(8.0),
                );
                node.paint.widget = Some(Arc::new(paint.finish()));
            }
        }
        let select_nodes: std::collections::HashMap<_, _> = self
            .selects
            .keys()
            .filter_map(|key| self.ids.get(key).map(|id| (*id, *key)))
            .collect();
        for node in &mut nodes {
            if let Some(key) = select_nodes.get(&node.node_id)
                && let Some(control) = self.selects.get(key)
            {
                use shell::PaintScene;
                use vello_common::kurbo::{Affine, BezPath, Stroke};
                let mut paint = shell::widget::PaintContext::new(
                    node.content_size[0],
                    node.content_size[1],
                    self.scale,
                    tcx,
                );
                let x = f64::from(node.content_size[0]) + 14.0;
                let y = f64::from(node.content_size[1]) * 0.5;
                let direction = if control.is_open() { -1.0 } else { 1.0 };
                let mut chevron = BezPath::new();
                chevron.move_to((x - 4.0, y - 2.0 * direction));
                chevron.line_to((x, y + 2.0 * direction));
                chevron.line_to((x + 4.0, y - 2.0 * direction));
                paint.scene_mut().stroke(
                    &Stroke::new(1.5),
                    Affine::IDENTITY,
                    node.paint.text_color,
                    None,
                    &chevron,
                );
                node.paint.widget = Some(Arc::new(paint.finish()));
            }
        }
        for node in &mut nodes {
            if let Some((key, _)) = self.ids.iter().find(|(_, id)| **id == node.node_id)
                && let Some(scene) = self.canvases.get(key)
            {
                use shell::PaintScene;
                use vello_common::{
                    kurbo::{Affine, Circle, Stroke},
                    peniko::Fill,
                };
                let mut paint = shell::widget::PaintContext::new(
                    node.content_size[0],
                    node.content_size[1],
                    self.scale,
                    tcx,
                );
                let color = |rgba: u32| {
                    let [r, g, b, a] = rgba.to_be_bytes();
                    Color::from_rgba8(r, g, b, a)
                };
                for circle in &scene.circles {
                    let shape = Circle::new(
                        (f64::from(circle.center.x), f64::from(circle.center.y)),
                        f64::from(circle.radius),
                    );
                    if let Some(fill) = circle.fill {
                        paint.scene_mut().fill(
                            Fill::NonZero,
                            Affine::IDENTITY,
                            color(fill),
                            None,
                            &shape,
                        );
                    }
                    if let Some(stroke) = circle.stroke {
                        paint.scene_mut().stroke(
                            &Stroke::new(f64::from(circle.stroke_width)),
                            Affine::IDENTITY,
                            color(stroke),
                            None,
                            &shape,
                        );
                    }
                }
                node.paint.widget = Some(Arc::new(paint.finish()));
            }
        }
        for node in &mut nodes {
            if let Some((key, _)) = self.ids.iter().find(|(_, id)| **id == node.node_id)
                && self.actor.view().bindings[key].node.kind == PROGRESS
            {
                use shell::PaintScene;
                use vello_common::{
                    kurbo::{Affine, Rect},
                    peniko::Fill,
                };
                let fraction: f64 = self.actor.view().bindings[key]
                    .text
                    .parse()
                    .expect("validated progress fraction");
                let width = f64::from(node.content_size[0]);
                let height = f64::from(node.content_size[1]);
                let mut paint = shell::widget::PaintContext::new(
                    node.content_size[0],
                    node.content_size[1],
                    self.scale,
                    tcx,
                );
                paint.scene_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    self.palette(*key)
                        .map_or(Color::from_rgb8(221, 229, 243), |p| theme::color(p.border)),
                    None,
                    &Rect::new(0.0, 0.0, width, height).to_rounded_rect(4.0),
                );
                if fraction > 0.0 {
                    paint.scene_mut().fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        self.palette(*key)
                            .map_or(Color::from_rgb8(37, 99, 235), |p| theme::color(p.accent)),
                        None,
                        &Rect::new(0.0, 0.0, width * fraction, height).to_rounded_rect(4.0),
                    );
                }
                node.paint.widget = Some(Arc::new(paint.finish()));
            }
        }
        self.paint_icons(&mut nodes, tcx);
        self.paint_choices(&mut nodes, tcx);
        self.paint_disclosure(&mut nodes, tcx);
        for node in &mut nodes {
            if let Some((_, slider)) = self
                .sliders
                .iter_mut()
                .find(|(key, _)| self.ids[key] == node.node_id)
            {
                let [x, y] = node.content_origin;
                slider.layout_changed(WidgetGeometry {
                    content_size: node.content_size,
                    device_scale: self.scale,
                    local_to_window: [1.0, 0.0, 0.0, 1.0, x as f64, y as f64],
                    window_to_local: [1.0, 0.0, 0.0, 1.0, -(x as f64), -(y as f64)],
                });
                let mut paint = shell::widget::PaintContext::new(
                    node.content_size[0],
                    node.content_size[1],
                    self.scale,
                    tcx,
                );
                slider.paint(&mut paint);
                node.paint.widget = Some(Arc::new(paint.finish()));
            }
        }
        for node in &mut nodes {
            let Some((_, editor)) = self
                .editors
                .iter_mut()
                .find(|(key, _)| self.ids[key] == node.node_id)
            else {
                continue;
            };
            let [x, y] = node.content_origin;
            editor.layout_changed(WidgetGeometry {
                content_size: node.content_size,
                device_scale: self.scale,
                local_to_window: [1.0, 0.0, 0.0, 1.0, x as f64, y as f64],
                window_to_local: [1.0, 0.0, 0.0, 1.0, -(x as f64), -(y as f64)],
            });
            let mut paint = shell::widget::PaintContext::new(
                node.content_size[0],
                node.content_size[1],
                self.scale,
                tcx,
            );
            editor.paint(&mut paint);
            node.paint.widget = Some(Arc::new(paint.finish()));
        }
        self.painted_scroll = self
            .actor
            .view()
            .renderer
            .iter()
            .map(|(key, state)| (*key, [state.scroll_x, state.scroll]))
            .collect();
        self.placed = nodes.clone();
        self.frames += 1;
        if let Some(updated) = self.scope_smoke.take() {
            let input = self
                .actor
                .root_widget(key("editor.title"))
                .expect("mounted scope input");
            let scroll = self
                .actor
                .root_widget(key("scope.scroll"))
                .expect("mounted scope scroll");
            assert!(self.metrics.is_some());
            if !updated {
                self.handle_event(UiEvent::Focus(true));
                self.set_focus(Some(input));
                self.editors
                    .get_mut(&input)
                    .unwrap()
                    .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
                    .unwrap();
                self.actor
                    .view_mut()
                    .renderer
                    .get_mut(&scroll)
                    .unwrap()
                    .scroll = 137.0;
                let widget = (&**self.editors.get(&input).unwrap() as *const TextInput) as usize;
                let node = self.ids[&input];
                let selection = self.editors[&input].text_selection();
                let storage = self.actor.root_actor().state_address();
                let slot = self
                    .actor
                    .root_actor()
                    .slot_address("editor.document.title");
                let root = self.actor.root_actor().view.bindings[&key("root.build")]
                    .text
                    .clone();
                self.actor
                    .root_actor_mut()
                    .dispatch(Message {
                        action: key("update"),
                        value: 0,
                    })
                    .unwrap();
                self.actor.tick().unwrap();
                self.sync().unwrap();
                assert_eq!(
                    self.actor.root_actor().view.bindings[&key("root.build")].text,
                    root
                );
                assert_eq!(
                    (&**self.editors.get(&input).unwrap() as *const TextInput) as usize,
                    widget
                );
                assert_eq!(self.ids[&input], node);
                assert_eq!(self.focus, Some(input));
                assert_eq!(self.editors[&input].text_selection(), selection);
                assert_eq!(self.actor.root_actor().state_address(), storage);
                assert_eq!(
                    self.actor
                        .root_actor()
                        .slot_address("editor.document.title"),
                    slot
                );
                assert_eq!(self.actor.view().renderer[&scroll].scroll, 137.0);
                self.scope_smoke = Some(true);
            } else {
                assert_eq!(self.actor.view().renderer[&scroll].scroll, 137.0);
                assert_eq!(self.focus, Some(input));
                assert!(
                    self.actor
                        .view()
                        .bindings
                        .values()
                        .any(|binding| binding.text == "Panel 3")
                );
                eprintln!(
                    "PASS native local scope update retains editor, Taffy node, selection, focus, scroll and state without root rebuild"
                );
                if self.scope_watch {
                    self.scope_watch_expected = Some(ScopeWatchProof {
                        input,
                        scroll,
                        node: self.ids[&input],
                        widget: (&**self.editors.get(&input).unwrap() as *const TextInput) as usize,
                        state: self.actor.root_actor().state_address(),
                        slot: self
                            .actor
                            .root_actor()
                            .slot_address("editor.document.title"),
                        version: self.actor.root_actor().implementation_version(),
                        failed: false,
                    });
                    eprintln!("SCOPE_WATCH_READY");
                } else {
                    self.capture = true;
                }
            }
        }

        if let Some(mut proof) = self.editor_window.take() {
            match proof.expected {
                None => {
                    assert!(self.metrics.is_some()); // Window exists and delivered real metrics.
                    let input = *self
                        .actor
                        .view()
                        .bindings
                        .iter()
                        .find(|(_, b)| b.node.slot == key("editor.draft"))
                        .unwrap()
                        .0;
                    let button = *self
                        .actor
                        .view()
                        .bindings
                        .iter()
                        .find(|(_, b)| b.node.kind == BUTTON && b.node.action == key("editor.save"))
                        .unwrap()
                        .0;
                    let rectangle = self
                        .placed
                        .iter()
                        .find(|node| node.node_id == self.ids[&button])
                        .unwrap()
                        .rect;
                    let position = shell::Point {
                        x: ((rectangle[0] + rectangle[2]) / 2.0) as f64,
                        y: ((rectangle[1] + rectangle[3]) / 2.0) as f64,
                    };
                    self.handle_event(UiEvent::Focus(true));
                    for phase in [PointerPhase::Down, PointerPhase::Up] {
                        self.handle_event(UiEvent::Pointer(shell::PointerEvent {
                            phase,
                            position,
                            button: Some(shell::PointerButton::Primary),
                            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
                            modifiers: Default::default(),
                            properties: Default::default(),
                        }));
                    }
                    assert!(self.actor.root_actor().boolean("editor.busy"));
                    self.set_focus(Some(input));
                    self.editors
                        .get_mut(&input)
                        .unwrap()
                        .config_changed(r#"{"selection":{"anchor":12,"head":12}}"#)
                        .unwrap();
                    self.prepare_for_event(tcx);
                    proof.expected = Some((
                        input,
                        self.actor.root_actor().state_address(),
                        self.actor.root_actor().slot_address("editor.draft"),
                        (&**self.editors.get(&input).unwrap() as *const TextInput) as usize,
                    ));
                    self.replace(&proof.replacement).unwrap();
                    self.handle_event(UiEvent::TextInput(" newer".into()));
                    self.prepare_for_event(tcx);
                    assert_eq!(
                        self.actor.root_actor().text("editor.draft"),
                        "Window draft newer"
                    );
                    self.editor_window = Some(proof);
                }
                Some(_) if self.actor.root_actor().boolean("editor.busy") => {
                    self.editor_window = Some(proof);
                }
                Some((input, storage, slot, widget)) => {
                    assert_eq!(self.actor.root_actor().state_address(), storage);
                    assert_eq!(self.actor.root_actor().slot_address("editor.draft"), slot);
                    assert_eq!(
                        (&**self.editors.get(&input).unwrap() as *const TextInput) as usize,
                        widget
                    );
                    assert_eq!(self.focus, Some(input));
                    if proof.loading {
                        assert_eq!(
                            self.actor.root_actor().text("editor.draft"),
                            "Loaded window 😀\nSecond line"
                        );
                        assert_eq!(
                            self.editors[&input].current_value(),
                            Some("Loaded window 😀\nSecond line")
                        );
                        assert_eq!(self.actor.root_actor().text("editor.status"), "Loaded.");
                        assert_eq!(self.actor.root_actor().io_result_count(), 0);
                        assert!(
                            self.actor
                                .view()
                                .bindings
                                .values()
                                .any(|b| b.text == "Loaded.")
                        );
                        self.capture = true;
                        eprintln!(
                            "PASS active native editor window loads Unicode contents through retained widget/slot/focus and presents Loaded status"
                        );
                    } else {
                        assert_eq!(
                            self.actor.root_actor().text("editor.draft"),
                            "Window draft newer"
                        );
                        assert_eq!(
                            self.editors[&input].current_value(),
                            Some("Window draft newer")
                        );
                        assert_eq!(std::fs::read(&proof.file).unwrap(), b"Window draft");
                        assert!(
                            self.actor
                                .view()
                                .bindings
                                .values()
                                .any(|b| b.text == "Unsaved changes.")
                        );
                        eprintln!(
                            "PASS active native editor window saves, reloads and edits with retained storage/slot/widget/focus, snapshot disk contents and unsaved status"
                        );
                        std::fs::write(&proof.file, "Loaded window 😀\nSecond line").unwrap();
                        let button = *self
                            .actor
                            .view()
                            .bindings
                            .iter()
                            .find(|(_, b)| {
                                b.node.action == key("editor.load") && b.node.kind == BUTTON
                            })
                            .unwrap()
                            .0;
                        let rectangle = self
                            .placed
                            .iter()
                            .find(|node| node.node_id == self.ids[&button])
                            .unwrap()
                            .rect;
                        let position = shell::Point {
                            x: ((rectangle[0] + rectangle[2]) / 2.0) as f64,
                            y: ((rectangle[1] + rectangle[3]) / 2.0) as f64,
                        };
                        for phase in [PointerPhase::Down, PointerPhase::Up] {
                            self.handle_event(UiEvent::Pointer(shell::PointerEvent {
                                phase,
                                position,
                                button: Some(shell::PointerButton::Primary),
                                buttons: if phase == PointerPhase::Down { 1 } else { 0 },
                                modifiers: Default::default(),
                                properties: Default::default(),
                            }));
                        }
                        assert!(self.actor.root_actor().boolean("editor.busy"));
                        self.set_focus(Some(input));
                        proof.loading = true;
                        self.editor_window = Some(proof);
                    }
                }
            }
        }
        if self.frames == 1
            && let Some(path) = self.io_smoke.take()
        {
            let input = key("graph.notes");
            self.focus = Some(input);
            self.editors.get_mut(&input).unwrap().focus_changed(true);
            self.io_expected = Some((
                self.actor.root_actor().state_address(),
                self.actor.root_actor().slot_address("notes"),
                (&**self.editors.get(&input).unwrap() as *const TextInput) as usize,
            ));
            self.actor
                .root_actor_mut()
                .dispatch(Message {
                    action: key(if self.save_smoke.is_some() {
                        "io_write"
                    } else {
                        "io_start"
                    }),
                    value: if self.save_smoke.is_some() { 7 } else { 0 },
                })
                .unwrap();
            self.replace(&path).unwrap();
        } else if self.io_expected.is_some() && self.actor.root_actor().integer("count") == 1 {
            let (storage, slot, widget) = self.io_expected.take().unwrap();
            let input = key("graph.notes");
            assert!(self.metrics.is_some());
            assert_eq!(self.actor.root_actor().state_address(), storage);
            assert_eq!(self.actor.root_actor().slot_address("notes"), slot);
            assert_eq!(
                (&**self.editors.get(&input).unwrap() as *const TextInput) as usize,
                widget
            );
            assert_eq!(self.focus, Some(input));
            let expected = if let Some(path) = &self.save_smoke {
                assert_eq!(std::fs::read(path).unwrap(), b"saved");
                "v2:"
            } else {
                "v2:Window async document"
            };
            assert_eq!(self.actor.root_actor().text("notes"), expected);
            assert_eq!(self.editors[&input].current_value(), Some(expected));
            assert_eq!(
                self.actor.view().bindings[&key("graph.caption.one")].text,
                "Derived: 3"
            );
            self.capture = true;
            if self.save_smoke.is_some() {
                eprintln!(
                    "PASS actual native atomic save wakes window and updates through replacement while retaining editor/focus/state"
                );
            }
            eprintln!(
                "PASS actual native async completion wakes window and updates through replacement while retaining editor/focus/state"
            );
        }
        if self.frames == 1 && self.watch_expected.is_some() {
            eprintln!("WATCH_READY");
        }
        if self.frames == 1
            && let Some(path) = self.graph_smoke.take()
        {
            assert_eq!(
                self.actor.root_actor().computed("double"),
                Some(&crate::Value::Integer(2))
            );
            let input = key("graph.notes");
            let widget = (&**self.editors.get(&input).unwrap() as *const TextInput) as usize;
            let storage = self.actor.root_actor().state_address();
            let slot = self.actor.root_actor().slot_address("count");
            let window = self.metrics;
            assert!(window.is_some());
            self.replace(&path).unwrap();
            assert_eq!(self.metrics, window);
            assert_eq!(self.actor.root_actor().state_address(), storage);
            assert_eq!(self.actor.root_actor().slot_address("count"), slot);
            assert_eq!(
                self.actor.root_actor().computed("double"),
                Some(&crate::Value::Integer(3))
            );
            assert_eq!(
                self.actor.view().bindings[&key("graph.caption.one")].text,
                "Derived: 3"
            );
            assert_eq!(
                self.actor.view().bindings[&key("graph.caption.two")].text,
                "Derived: 3"
            );
            assert_eq!(
                (&**self.editors.get(&input).unwrap() as *const TextInput) as usize,
                widget
            );
            assert_eq!(self.focus, Some(input));
            assert_eq!(
                self.editors[&input].current_value(),
                Some("Native computed state stays")
            );
            eprintln!(
                "PASS actual native computed replacement retains state and updates both shared dependents"
            );
            self.capture = true;
        }
        if self.frames == 1
            && let Some(path) = self.smoke.take()
        {
            let input_key = key("counter.notes");
            self.focus = Some(input_key);
            self.editors
                .get_mut(&input_key)
                .unwrap()
                .focus_changed(true);
            self.actor
                .root_actor_mut()
                .set_text(key("notes"), "Native state stays here")
                .unwrap();
            self.actor
                .view_mut()
                .renderer
                .get_mut(&key("counter.scroll"))
                .unwrap()
                .scroll = 137.0;
            let window = self.metrics;
            assert!(
                window.is_some(),
                "native capture requires real window metrics"
            );
            self.replace(&path).unwrap();
            assert_eq!(self.metrics, window);
            self.capture = true;
        }
        if let Some((report, saw_cut)) = &mut self.clipboard_smoke {
            let notes = self.actor.root_actor().text("notes");
            *saw_cut |= notes.is_empty();
            let value = serde_json::json!({"notes": notes, "saw_cut": *saw_cut, "focused": self.window_focused});
            if let Err(error) = std::fs::write(report, value.to_string()) {
                self.diagnostic = error.to_string();
            }
            if *saw_cut && notes == "clipboard complete 😀" {
                self.capture = true;
            }
        }
        self.append_select_popup(&mut nodes, width, height);
        // Re-hit-test stationary pointers after scrolling, layout, modal or DLL
        // replacement. Hover changes paint only, so preserve widget fragments
        // and measured geometry while refreshing the affected paint contexts.
        if self.refresh_hover() {
            let dirty = self.paint_dirty.clone();
            if let Err(error) = self.sync() {
                self.diagnostic = error;
            }
            for node in &mut nodes {
                if dirty
                    .iter()
                    .any(|key| self.ids.get(key) == Some(&node.node_id))
                    && let Some(paint) = self.tree.get_node_context(node.node_id)
                {
                    let widget = node.paint.widget.take();
                    node.paint = paint.clone();
                    node.paint.widget = widget;
                }
            }
        }
        self.paint_motion(&mut nodes);
        self.refresh_semantics();
        nodes
    }
    fn ime_state(&self) -> Option<shell::ImeState> {
        if !self.window_focused {
            return None;
        }
        let key = self.focus?;
        if self.actor.view().bindings.get(&key)?.node.read_only != 0 {
            return None;
        }
        let editor = self.editors.get(&key)?;
        let state = editor.ime_state()?;
        let node = self.placed.iter().find(|n| n.node_id == self.ids[&key])?;
        let mut area = state.cursor_area;
        area[0] += node.content_origin[0];
        area[1] += node.content_origin[1];
        Some(shell::ImeState {
            client_id: key,
            cursor_area: area.map(f64::from),
            surrounding_text: state.surrounding_text,
            surrounding_cursor: state.surrounding_cursor,
            surrounding_anchor: state.surrounding_anchor,
            selection_utf16: state.selection_utf16,
            selection_reversed: state.selection_reversed,
            marked_range_utf16: state.marked_range_utf16,
            purpose: state.purpose,
            multiline: state.multiline,
            completion: state.completion,
            spellcheck: state.spellcheck,
        })
    }
    fn base_color(&self) -> Color {
        Color::from_rgb8(250, 251, 253)
    }
    fn has_anim(&self) -> bool {
        self.capture || self.motion.is_active()
    }
    fn animation_deadline(&self) -> Option<Instant> {
        self.editors
            .values()
            .filter_map(|e| e.animation_deadline())
            .chain(self.actor.async_deadline())
            .chain(
                self.menu_hover
                    .deadline()
                    .and_then(|ms| self.motion_epoch.checked_add(Duration::from_millis(ms))),
            )
            .chain(
                self.tooltip
                    .deadline()
                    .and_then(|d| self.motion_epoch.checked_add(d)),
            )
            .min()
    }
    fn set_wake_callback(&mut self, wake: shell::WakeCallback) {
        self.actor.set_wake(wake.clone());
        *self.wake.lock().unwrap() = Some(wake);
    }
    fn poll_async(&mut self) -> bool {
        self.persist_native_renderer();
        let mut changed = false;
        for editor in self.editors.values_mut() {
            changed |= editor.poll_async();
        }
        if let Some(receiver) = &self.builds {
            let results: Vec<_> = receiver.try_iter().collect();
            for result in results {
                self.diagnostic = result.message;
                if let Some(path) = result.artifact {
                    let replacement_started = Instant::now();
                    if let Err(error) = self.replace(&path) {
                        self.diagnostic = error;
                    } else {
                        changed = true;
                        self.diagnostic.push_str(&format!(
                            "; replaced in {:.1} ms",
                            replacement_started.elapsed().as_secs_f64() * 1000.0
                        ));
                    }
                    let _ = std::fs::remove_file(path);
                }
                if let Some(proof) = &mut self.scope_watch_expected {
                    assert_eq!(self.actor.root_actor().state_address(), proof.state);
                    assert_eq!(
                        self.actor
                            .root_actor()
                            .slot_address("editor.document.title"),
                        proof.slot
                    );
                    assert_eq!(self.actor.root_actor().integer("editor.document.count"), 3);
                    assert_eq!(
                        self.actor.root_actor().text("editor.document.title"),
                        "draft!"
                    );
                    assert_eq!(self.ids[&proof.input], proof.node);
                    assert_eq!(
                        (&**self.editors.get(&proof.input).unwrap() as *const TextInput) as usize,
                        proof.widget
                    );
                    assert_eq!(
                        self.editors[&proof.input]
                            .text_selection()
                            .as_ref()
                            .map(|s| (s.anchor, s.head)),
                        Some((1, 4))
                    );
                    assert_eq!(self.focus, Some(proof.input));
                    assert_eq!(self.actor.view().renderer[&proof.scroll].scroll, 137.0);
                    if changed {
                        assert!(
                            proof.failed,
                            "scope watch proof must see compilation failure first"
                        );
                        assert_ne!(
                            self.actor.root_actor().implementation_version(),
                            proof.version
                        );
                        assert!(
                            self.actor
                                .view()
                                .bindings
                                .values()
                                .any(|binding| binding.text == "Reloaded panel 3")
                        );
                        self.capture = true;
                        eprintln!(
                            "PASS native scope save/error/recovery retains state, widget, selection, focus and scroll"
                        );
                    } else {
                        assert_eq!(
                            self.actor.root_actor().implementation_version(),
                            proof.version
                        );
                        proof.failed = true;
                    }
                }
                let watch_input = self.actor.root_widget(key("counter.notes"));
                let watch_scroll = self.actor.root_widget(key("counter.scroll"));
                if let Some((state, slot, version, failed)) = &mut self.watch_expected {
                    assert_eq!(self.actor.root_actor().state_address(), *state);
                    assert_eq!(self.actor.root_actor().slot_address("count"), *slot);
                    assert_eq!(self.actor.root_actor().integer("count"), 1);
                    assert_eq!(
                        self.actor.root_actor().text("notes"),
                        "Watch keeps unsaved notes"
                    );
                    assert_eq!(self.focus, watch_input);
                    assert_eq!(
                        self.actor.view().renderer[&watch_scroll.unwrap()].scroll,
                        137.0
                    );
                    if changed {
                        assert!(*failed, "watch proof must see a compiler failure first");
                        assert_ne!(self.actor.root_actor().implementation_version(), *version);
                        self.actor
                            .root_actor_mut()
                            .dispatch(Message {
                                action: key("increment"),
                                value: 0,
                            })
                            .unwrap();
                        assert_eq!(self.actor.root_actor().integer("count"), 3);
                        self.capture = true;
                        eprintln!(
                            "PASS native save watcher retains state through failed build and successful replacement"
                        );
                    } else {
                        assert_eq!(self.actor.root_actor().implementation_version(), *version);
                        *failed = true;
                    }
                }
                eprintln!("{}", self.diagnostic);
            }
        }
        if let Err(error) = self.reconcile_implementation_watches() {
            self.diagnostic = error;
        }
        let results: Vec<_> = self
            .actor_watches
            .values()
            .flat_map(|target| {
                target
                    .builds
                    .try_iter()
                    .map(|result| (target.target, target.lifecycle, result))
            })
            .collect();
        for (target, lifecycle, result) in results {
            self.diagnostic = format!("actor {}: {}", target.0, result.message);
            if let Some(path) = result.artifact {
                if self.actor.lifecycle(target) == Some(lifecycle) {
                    let replacement_started = Instant::now();
                    match self.actor.reload_target(target, &path) {
                        Ok(()) => {
                            changed = true;
                            self.diagnostic.push_str(&format!(
                                "; replaced in {:.1} ms",
                                replacement_started.elapsed().as_secs_f64() * 1000.0
                            ));
                        }
                        Err(error) => self.diagnostic = format!("actor {}: {error}", target.0),
                    }
                }
                let _ = std::fs::remove_file(path);
            }
            self.actor_diagnostics
                .insert(target, self.diagnostic.clone());
            eprintln!("{}", self.diagnostic);
        }
        // A build belongs to its implementation, not the instances alive when
        // compilation started. Publish to the current members between turns.
        let children = self.actor.declared_children();
        let group_results: Vec<_> = self
            .implementation_watches
            .iter()
            .flat_map(|(identity, target)| {
                let members: Vec<_> = children
                    .iter()
                    .filter(|(id, implementation)| {
                        implementation == identity && !self.actor_watches.contains_key(id)
                    })
                    .filter_map(|(id, _)| {
                        self.actor.lifecycle(*id).map(|lifecycle| (*id, lifecycle))
                    })
                    .collect();
                // Keep the bounded queued artifact until an instance exists again.
                let results: Vec<_> = if members.is_empty() {
                    Vec::new()
                } else {
                    target.builds.try_iter().collect()
                };
                results
                    .into_iter()
                    .map(move |result| (target.name.clone(), members.clone(), result))
            })
            .collect();
        for (name, members, result) in group_results {
            let ids: Vec<_> = members
                .into_iter()
                .filter_map(|(id, lifecycle)| {
                    (self.actor.lifecycle(id) == Some(lifecycle)).then_some(id)
                })
                .collect();
            let diagnostic = if let Some(path) = &result.artifact {
                if ids.is_empty() {
                    "no current implementation members".to_owned()
                } else {
                    let replacement_started = Instant::now();
                    match self.actor.reload_targets(&ids, path) {
                        Ok(()) => {
                            changed = true;
                            format!(
                                "{}; replaced in {:.1} ms",
                                result.message,
                                replacement_started.elapsed().as_secs_f64() * 1000.0
                            )
                        }
                        Err(error) => error,
                    }
                }
            } else {
                result.message.clone()
            };
            let actors: Vec<_> = ids.iter().map(|id| id.0).collect();
            eprintln!("implementation {name:?}, actors {actors:?}: {diagnostic}");
            for target in ids {
                self.diagnostic = format!("actor {}: {diagnostic}", target.0);
                self.actor_diagnostics
                    .insert(target, self.diagnostic.clone());
            }
            if let Some(path) = result.artifact {
                let _ = std::fs::remove_file(path);
            }
        }
        if let Err(error) = self.actor.tick() {
            self.diagnostic = error;
        }
        if let Err(error) = self.reconcile_implementation_watches() {
            self.diagnostic = error;
        }
        let view = self.actor.view();
        changed
            || self.projection_needs_sync()
            || view.renderer.iter().any(|(key, state)| {
                self.painted_scroll.get(key) != Some(&[state.scroll_x, state.scroll])
            })
            || self
                .ids
                .values()
                .any(|id| self.tree.dirty(*id).unwrap_or(true))
    }
    fn handle_event(&mut self, event: UiEvent) -> EventResponse {
        self.persist_native_renderer();
        if let UiEvent::WindowMetrics(metrics) = event {
            let changed = self
                .metrics
                .is_none_or(|old| old.color_scheme != metrics.color_scheme);
            self.metrics = Some(metrics);
            return if changed {
                EventResponse {
                    request_redraw: true,
                    ..EventResponse::IGNORED
                }
            } else {
                EventResponse::IGNORED
            };
        }
        if matches!(&event, UiEvent::Paste(_)) {
            let target = self.paste_target.take();
            if !self.window_focused
                || target.is_none()
                || target != self.focus
                || !target.is_some_and(|key| self.editors.contains_key(&key))
            {
                return EventResponse::IGNORED;
            }
        }
        if let UiEvent::Focus(focused) = &event {
            self.window_focused = *focused;
            if !focused {
                self.menu_hover.reset();
                self.menu_pointer_active = false;
                if let Some(panel) = self.active_popover().filter(|panel| {
                    self.actor.view().bindings[panel].node.semantics.role == semantics::MENU_PANEL
                }) && let Err(error) = self.actor.dismiss_popover(self.menu_root(panel))
                {
                    self.diagnostic = error;
                }
                for control in self.selects.values_mut() {
                    control.close();
                }
                self.select_popup = None;
                self.select_pressed = None;
                self.pointer_position = None;
                self.tooltip.reset();
                self.refresh_hover();
                self.paste_target = None;
                self.composing_editor = None;
            }
            if let Some(key) = self.keyboard_pressed.take() {
                self.paint_dirty.insert(key);
            }
            self.pressed = None;
            self.set_pointer_pressed_visual(None);
            if let Some(key) = self.focus {
                if let Some(slider) = self.sliders.get_mut(&key) {
                    slider.focus_changed(*focused);
                }
                if let Some(editor) = self.editors.get_mut(&key) {
                    editor.focus_changed(*focused);
                }
                self.paint_dirty.insert(key);
            }
            self.reconcile_toast_activity();
            return self.input_response(false);
        }
        if let UiEvent::Key(key) = &event {
            if key.phase == shell::KeyPhase::Down {
                self.menu_pointer_active = false;
                self.menu_hover.reset();
            }
            if key.synthetic {
                if let Some(old) = self.keyboard_pressed.take() {
                    self.paint_dirty.insert(old);
                }
                return self.input_response(true);
            }
            if self.window_focused
                && key.key == "Escape"
                && key.phase == shell::KeyPhase::Down
                && !key.repeat
                && key.modifiers.is_empty()
                && (self.tooltip.visible().is_some() || self.tooltip.deadline().is_some())
            {
                self.dismiss_tooltip();
                return self.input_response(true);
            }
            if self.window_focused
                && key.phase == shell::KeyPhase::Down
                && !key.repeat
                && key.modifiers.is_empty()
                && matches!(key.key.as_str(), "Enter" | " " | "Space")
            {
                self.dismiss_tooltip();
            }
            if self.window_focused
                && key.phase == shell::KeyPhase::Down
                && !key.repeat
                && key.modifiers.is_empty()
                && self.navigate_toast(&key.key)
            {
                return self.input_response(true);
            }
            if self.window_focused
                && key.key == "Escape"
                && let Some(focus) = self.focus
                && self
                    .selects
                    .get(&focus)
                    .is_some_and(|control| control.is_open())
            {
                if key.phase == shell::KeyPhase::Down && !key.repeat && key.modifiers.is_empty() {
                    self.selects.get_mut(&focus).unwrap().close();
                    self.select_popup = None;
                    self.select_pressed = None;
                    self.paint_dirty.insert(focus);
                }
                return self.input_response(true);
            }
            if self.window_focused
                && key.key == "Escape"
                && let Some(panel) = self.active_popover()
            {
                if key.phase == shell::KeyPhase::Down
                    && !key.repeat
                    && key.modifiers.is_empty()
                    && let Err(error) = self.actor.dismiss_popover(panel)
                {
                    self.diagnostic = error;
                }
                return self.input_response(true);
            }
            if self.window_focused
                && key.key == "Escape"
                && let Some(modal) = self.modal
            {
                if key.phase == shell::KeyPhase::Down
                    && !key.repeat
                    && !key.synthetic
                    && key.modifiers.is_empty()
                    && let Err(error) = self.actor.dismiss_dialog(modal)
                {
                    self.diagnostic = error;
                }
                return self.input_response(true);
            }
            if self.window_focused
                && !key.synthetic
                && key.phase == shell::KeyPhase::Down
                && ((key.key == "ContextMenu" && key.modifiers.is_empty())
                    || (key.key == "F10" && key.modifiers == shell::Modifiers::SHIFT))
                && let Some(trigger) = self.focus.and_then(|id| self.context_trigger_for(id))
            {
                if !key.repeat {
                    self.open_context_menu(trigger, None);
                }
                return self.input_response(true);
            }
            if self.window_focused
                && !key.synthetic
                && key.phase == shell::KeyPhase::Down
                && key.modifiers.is_empty()
                && (self.navigate_menu(key)
                    || self.navigate_disclosure(&key.key)
                    || self.navigate_choice(&key.key))
            {
                return self.input_response(true);
            }
            if self.window_focused
                && key.phase == shell::KeyPhase::Down
                && key.modifiers.is_empty()
                && let Some(focus) = self.focus
                && self.selects.contains_key(&focus)
            {
                let page_rows = self
                    .select_popup
                    .as_ref()
                    .filter(|(owner, _, _)| *owner == focus)
                    .map_or(8, |(_, layout, _)| layout.end - layout.start);
                let movement = match key.key.as_str() {
                    "ArrowUp" => Some(crate::select_model::Key::Up),
                    "ArrowDown" => Some(crate::select_model::Key::Down),
                    "Home" => Some(crate::select_model::Key::Home),
                    "End" => Some(crate::select_model::Key::End),
                    "PageUp" => Some(crate::select_model::Key::PageUp(page_rows)),
                    "PageDown" => Some(crate::select_model::Key::PageDown(page_rows)),
                    "Enter" | " " | "Space" => Some(crate::select_model::Key::Activate),
                    _ => None,
                };
                if let Some(movement) = movement {
                    self.select_pressed = None;
                    let requested = self
                        .selects
                        .get_mut(&focus)
                        .unwrap()
                        .key(movement, key.repeat);
                    if let Some(value) = requested
                        && let Err(error) = self.actor.set_widget_select(focus, value)
                    {
                        self.diagnostic = error;
                    }
                    self.paint_dirty.insert(focus);
                    return self.input_response(true);
                }
            }
            use shell::{KeyPhase, Modifiers};
            let primary = if cfg!(target_os = "macos") {
                Modifiers::META
            } else {
                Modifiers::CONTROL
            };
            if self.window_focused && key.modifiers == primary {
                let letter = key.key.to_ascii_lowercase();
                let code = letter
                    .as_bytes()
                    .first()
                    .copied()
                    .filter(|_| letter.len() == 1)
                    .map(u32::from);
                let declared: Vec<_> = self
                    .actor
                    .view()
                    .bindings
                    .iter()
                    .filter(|(_, binding)| {
                        code == Some(binding.node.shortcut)
                            && binding.node.shortcut != 0
                            && letter.len() == 1
                    })
                    .map(|(identity, _)| *identity)
                    .collect();
                if declared.len() > 1 {
                    self.diagnostic = "ambiguous primary shortcut across mounted actors".into();
                    return self.input_response(true);
                }
                let target = declared
                    .first()
                    .copied()
                    .or_else(|| self.primary_shortcuts.get(&letter).copied());
                if let Some(widget) = target
                    && self.in_modal(widget)
                    && self
                        .actor
                        .view()
                        .bindings
                        .get(&widget)
                        .is_some_and(|binding| binding.node.kind == BUTTON)
                {
                    if self.composing_editor.is_some() {
                        return self.input_response(true);
                    }
                    if key.phase == KeyPhase::Down
                        && !key.repeat
                        && let Err(error) = self.actor.dispatch_widget(widget)
                    {
                        self.diagnostic = error;
                    }
                    return self.input_response(true);
                }
            }
            let shortcut = key
                .modifiers
                .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META);
            if self.window_focused && !key.synthetic && !shortcut && key.key == "Tab" {
                if key.phase == KeyPhase::Down && !key.repeat {
                    if let Some(panel) = self.active_popover().filter(|panel| {
                        self.actor.view().bindings[panel].node.semantics.role
                            == semantics::MENU_PANEL
                    }) {
                        let panel = self.menu_root(panel);
                        let anchor = self.actor.view().bindings[&panel]
                            .node
                            .semantics
                            .labelled_by;
                        self.set_focus(Some(anchor));
                        if let Err(error) = self.actor.dismiss_popover(panel) {
                            self.diagnostic = error;
                        } else {
                            self.focus_next(key.modifiers.shift());
                        }
                        return self.input_response(true);
                    }
                    self.focus_next(key.modifiers.shift());
                    if let Some(panel) = self.active_popover() {
                        let anchor = self.actor.view().bindings[&panel]
                            .node
                            .semantics
                            .labelled_by;
                        if self.focus.is_none_or(|focus| {
                            focus != anchor && !self.popover_contains(panel, focus)
                        }) && let Err(error) = self.actor.dismiss_popover(panel)
                        {
                            self.diagnostic = error;
                        }
                    }
                }
                return self.input_response(true);
            }
            if let Some(focus) = self.focus
                && self
                    .actor
                    .view()
                    .bindings
                    .get(&focus)
                    .is_some_and(|binding| matches!(binding.node.kind, BUTTON | CHECKBOX))
                && matches!(key.key.as_str(), "Enter" | " " | "Space")
            {
                if self.actor.view().bindings[&focus].node.disabled != 0 {
                    return self.input_response(true);
                }
                let mut activate = false;
                if !self.window_focused || key.synthetic || shortcut {
                    if let Some(old) = self.keyboard_pressed.take() {
                        self.paint_dirty.insert(old);
                    }
                } else if key.key == "Enter" {
                    activate = key.phase == KeyPhase::Down && !key.repeat;
                } else if key.phase == KeyPhase::Down && !key.repeat {
                    self.keyboard_pressed = Some(focus);
                    self.paint_dirty.insert(focus);
                } else if key.phase == KeyPhase::Up {
                    activate = self.keyboard_pressed.take() == Some(focus);
                    self.paint_dirty.insert(focus);
                }
                if activate && let Err(error) = self.actor.dispatch_widget(focus) {
                    self.diagnostic = error;
                }
                return self.input_response(true);
            }
        }
        if let UiEvent::TextInput(text) = &event
            && self.window_focused
            && let Some(focus) = self.focus
            && self.selects.contains_key(&focus)
        {
            let requested = self
                .selects
                .get_mut(&focus)
                .unwrap()
                .type_text(text, Instant::now());
            if let Some(value) = requested
                && let Err(error) = self.actor.set_widget_select(focus, value)
            {
                self.diagnostic = error;
            }
            self.paint_dirty.insert(focus);
            return self.input_response(true);
        }
        if let UiEvent::Pointer(pointer) = &event {
            if pointer.phase == PointerPhase::Down {
                self.menu_pointer_active = false;
                self.menu_hover.reset();
            }
            let hit = self.hit(pointer.position);
            if pointer.phase == PointerPhase::Down
                && pointer.button == Some(shell::PointerButton::Secondary)
                && let Some(trigger) = hit.and_then(|id| self.context_trigger_for(id))
            {
                self.open_context_menu(trigger, Some([pointer.position.x, pointer.position.y]));
                return self.input_response(true);
            }
            if pointer.phase == PointerPhase::Down
                && pointer.button == Some(shell::PointerButton::Primary)
                && let Some(panel) = self.active_popover()
            {
                let anchor = self.actor.view().bindings[&panel]
                    .node
                    .semantics
                    .labelled_by;
                let inside = [panel, anchor].iter().any(|key| {
                    self.widget_layout_rect(*key).is_some_and(|r| {
                        pointer.position.x >= f64::from(r[0])
                            && pointer.position.x <= f64::from(r[2])
                            && pointer.position.y >= f64::from(r[1])
                            && pointer.position.y <= f64::from(r[3])
                    })
                });
                let panel = if self.actor.view().bindings[&panel].node.semantics.role
                    == semantics::MENU_PANEL
                {
                    let root = self.menu_root(panel);
                    if hit.is_some_and(|hit| self.popover_contains(root, hit)) {
                        panel
                    } else {
                        root
                    }
                } else {
                    panel
                };
                if !inside && let Err(error) = self.actor.dismiss_popover(panel) {
                    self.diagnostic = error;
                }
            }
            if pointer.properties.primary {
                self.pointer_position =
                    if matches!(pointer.phase, PointerPhase::Leave | PointerPhase::Cancel)
                        || !matches!(
                            pointer.properties.pointer_type,
                            shell::PointerType::Mouse | shell::PointerType::Pen
                        )
                    {
                        None
                    } else {
                        Some(pointer.position)
                    };
                if pointer.phase == PointerPhase::Move {
                    self.menu_pointer_active = pointer.buttons == 0;
                    self.poll_menu_hover();
                }
                self.set_hover_from_hit(hit);
                if pointer.phase == PointerPhase::Down {
                    self.dismiss_tooltip_at(hit);
                }
            }
            if matches!(pointer.phase, PointerPhase::Leave | PointerPhase::Cancel) {
                self.set_pointer_pressed_visual(None);
            }
            if let Some((owner, layout, values)) = &self.select_popup
                && self
                    .selects
                    .get(owner)
                    .is_some_and(|control| control.is_open())
            {
                let owner = *owner;
                let value = layout
                    .hit([pointer.position.x as f32, pointer.position.y as f32])
                    .and_then(|index| values.get(index - layout.start))
                    .copied();
                if pointer.phase == PointerPhase::Move
                    && let Some(value) = value
                {
                    if self.selects.get_mut(&owner).unwrap().highlight(value) {
                        self.paint_dirty.insert(owner);
                    }
                    let mut response = self.input_response(true);
                    response.request_redraw = !self.paint_dirty.is_empty();
                    return response;
                }
                if pointer.phase == PointerPhase::Down
                    && pointer.button == Some(shell::PointerButton::Primary)
                {
                    self.select_pressed = value;
                    self.pressed = None;
                    if value.is_none() {
                        self.selects.get_mut(&owner).unwrap().close();
                        self.select_popup = None;
                        self.paint_dirty.insert(owner);
                    }
                    return self.input_response(true);
                }
                if pointer.phase == PointerPhase::Up
                    && pointer.button == Some(shell::PointerButton::Primary)
                {
                    if let Some(value) =
                        value.filter(|value| Some(*value) == self.select_pressed.take())
                    {
                        let request = self.selects.get_mut(&owner).unwrap().choose(value);
                        if let Some(value) = request
                            && let Err(error) = self.actor.set_widget_select(owner, value)
                        {
                            self.diagnostic = error;
                        }
                    }
                    self.select_pressed = None;
                    self.paint_dirty.insert(owner);
                    return self.input_response(true);
                }
                if pointer.phase == PointerPhase::Cancel {
                    self.select_pressed = None;
                }
                if value.is_some() {
                    return self.input_response(true);
                }
            }
            if pointer.buttons == 0
                && self.pressed.is_none()
                && matches!(
                    pointer.phase,
                    PointerPhase::Move | PointerPhase::Enter | PointerPhase::Leave
                )
            {
                let mut response = self.input_response(false);
                response.request_redraw = !self.paint_dirty.is_empty();
                return response;
            }
            let visual = match pointer.phase {
                PointerPhase::Down if pointer.button == Some(shell::PointerButton::Primary) => hit,
                PointerPhase::Move if self.pressed == hit => hit,
                PointerPhase::Up if pointer.button == Some(shell::PointerButton::Primary) => None,
                PointerPhase::Move | PointerPhase::Cancel => None,
                _ => self.pointer_pressed_visual,
            }
            .filter(|key| {
                self.window_focused
                    && self.actor.view().bindings.get(key).is_some_and(|binding| {
                        matches!(binding.node.kind, BUTTON | CHECKBOX) && binding.node.disabled == 0
                    })
            });
            self.set_pointer_pressed_visual(visual);
            // A retained slider owns its drag until release/cancel, even
            // outside the modal. Never admit a new background interaction.
            let captured_slider_move = pointer.phase == PointerPhase::Move
                && self
                    .pressed
                    .is_some_and(|key| self.focus == Some(key) && self.sliders.contains_key(&key));
            if self.modal.is_some()
                && hit.is_none()
                && !captured_slider_move
                && !matches!(pointer.phase, PointerPhase::Up | PointerPhase::Cancel)
            {
                return self.input_response(true);
            }
            if pointer.phase == PointerPhase::Down
                && pointer.button == Some(shell::PointerButton::Primary)
            {
                self.set_focus(hit.filter(|key| {
                    matches!(
                        self.actor.view().bindings[key].node.kind,
                        BUTTON | CHECKBOX | SELECT | SLIDER | ENTRY | TEXTAREA
                    ) && self.actor.view().bindings[key].node.disabled == 0
                }));
                self.pressed = hit;
            }
            if pointer.phase == PointerPhase::Down
                && matches!(
                    pointer.button,
                    Some(shell::PointerButton::Primary | shell::PointerButton::Secondary)
                )
                && let Some(key) = hit
                && self.actor.view().bindings[&key].node.kind == CANVAS
                && let Some(node) = self
                    .placed
                    .iter()
                    .find(|node| node.node_id == self.ids[&key])
            {
                let point = LocalPoint {
                    x: (pointer.position.x - f64::from(node.content_origin[0])) as f32,
                    y: (pointer.position.y - f64::from(node.content_origin[1])) as f32,
                };
                let secondary = pointer.button == Some(shell::PointerButton::Secondary);
                if point.x >= 0.0
                    && point.y >= 0.0
                    && point.x <= node.content_size[0]
                    && point.y <= node.content_size[1]
                {
                    if let Err(error) = self.actor.set_widget_canvas(key, point, secondary) {
                        self.diagnostic = error;
                    }
                    self.paint_dirty.insert(key);
                    self.pressed = None;
                    return self.input_response(true);
                }
            }
            if pointer.phase == PointerPhase::Cancel {
                self.pressed = None;
            }
            if pointer.phase == PointerPhase::Up
                && pointer.button == Some(shell::PointerButton::Primary)
                && hit == self.pressed.take()
            {
                if let Some(key) = hit {
                    let node = self.actor.view().bindings[&key].node;
                    if node.kind == SELECT {
                        let request = self
                            .selects
                            .get_mut(&key)
                            .unwrap()
                            .key(crate::select_model::Key::Activate, false);
                        if let Some(value) = request
                            && let Err(error) = self.actor.set_widget_select(key, value)
                        {
                            self.diagnostic = error;
                        }
                        self.paint_dirty.insert(key);
                    }
                    if matches!(node.kind, BUTTON | CHECKBOX)
                        && let Err(error) = self.actor.dispatch_widget(key)
                    {
                        self.diagnostic = error;
                    }
                }
                self.pressed = None;
            }
        }
        if let UiEvent::Wheel(wheel) = &event
            && self.scroll_tooltip(wheel)
        {
            return self.input_response(false);
        }
        if let UiEvent::Wheel(wheel) = &event
            && let Some((owner, layout, _)) = &self.select_popup
            && layout
                .hit([wheel.position.x as f32, wheel.position.y as f32])
                .is_some()
            && self
                .selects
                .get(owner)
                .is_some_and(|control| control.is_open())
        {
            let owner = *owner;
            if wheel.delta_y.is_finite() && wheel.delta_y != 0.0 {
                self.selects.get_mut(&owner).unwrap().key(
                    if wheel.delta_y > 0.0 {
                        crate::select_model::Key::Down
                    } else {
                        crate::select_model::Key::Up
                    },
                    false,
                );
                self.paint_dirty.insert(owner);
            }
            return self.input_response(false);
        }
        if let UiEvent::Wheel(wheel) = &event {
            self.scroll_wheel(wheel);
        }
        // Wheel belongs to the hovered editor, independently of keyboard focus.
        // It must not also reach a focused editor after scrolling another widget.
        if let UiEvent::Wheel(wheel) = &event {
            if let Some(key) = self.hit(wheel.position)
                && let Some(editor) = self.editors.get_mut(&key)
            {
                editor.handle_event(&event);
            }
            return self.input_response(false);
        }
        if self.window_focused
            && self.composing_editor.is_none()
            && let UiEvent::Key(event) = &event
            && event.phase == shell::KeyPhase::Down
            && !event.synthetic
            && let Some((grid, index, stride_key)) = self.grid_navigation_origin()
        {
            let primary = if cfg!(target_os = "macos") {
                shell::Modifiers::META
            } else {
                shell::Modifiers::CONTROL
            };
            let movement = match (
                event.key.as_str(),
                event.modifiers.is_empty(),
                event.modifiers == primary,
            ) {
                ("ArrowLeft", true, _) => Some(0),
                ("ArrowRight", true, _) => Some(1),
                ("ArrowUp", true, _) => Some(2),
                ("ArrowDown", true, _) => Some(3),
                ("Home", true, _) => Some(4),
                ("End", true, _) => Some(5),
                ("Home", _, true) => Some(6),
                ("End", _, true) => Some(7),
                ("PageUp", true, _) => Some(8),
                ("PageDown", true, _) => Some(9),
                _ => None,
            };
            if let Some(movement) = movement {
                let node = self.actor.view().bindings[&grid].node;
                let extent = blockui_abi::grid::GridExtent::new(
                    node.grid_rows as usize,
                    node.grid_columns as usize,
                )
                .unwrap();
                let stride = self
                    .ids
                    .get(&stride_key)
                    .and_then(|id| self.tree.layout(*id).ok())
                    .map_or(1.0, |layout| layout.size.height.max(1.0));
                let port = self.tree.layout(self.ids[&grid]).map_or(1.0, |layout| {
                    (layout.size.height
                        - layout.padding.top
                        - layout.padding.bottom
                        - layout.border.top
                        - layout.border.bottom)
                        .max(1.0)
                });
                let next = usize::try_from(index).ok().and_then(|index| {
                    extent.navigate(
                        index,
                        blockui_abi::grid::GridMove::try_from(movement).unwrap(),
                        (port / stride).floor().max(1.0) as usize,
                    )
                });
                if let Some(next) = next
                    && next as i64 != index
                {
                    match self.actor.navigate_grid(grid, next as i64) {
                        Err(error) => self.diagnostic = error,
                        Ok(()) => {
                            if let Err(error) = self.sync() {
                                self.diagnostic = error;
                            }
                            if let Some(target) = self.grid_target(grid, next as i64) {
                                self.set_focus(Some(target));
                                self.pending_reveal = Some(target);
                            } else {
                                self.suspended_grid_focus = Some((grid, next as i64));
                            }
                        }
                    }
                }
                return self.input_response(true);
            }
        }
        if self.window_focused
            && self.composing_editor.is_none()
            && let UiEvent::Key(event) = &event
            && event.phase == shell::KeyPhase::Down
            && !event.repeat
            && !event.synthetic
            && event.modifiers.is_empty()
            && matches!(event.key.as_str(), "Enter" | "Escape")
            && let Some(widget) = self.focus
            && let Some(binding) = self.actor.view().bindings.get(&widget)
            && binding.node.kind == ENTRY
            && binding.node.disabled == 0
            && binding.node.read_only == 0
            && (if event.key == "Escape" {
                binding.node.cancel_action
            } else {
                binding.node.submit_action
            }) != 0
        {
            if let Err(error) = self.actor.editor_command(widget, event.key == "Escape") {
                self.diagnostic = error;
            }
            if let Err(error) = self.sync() {
                self.diagnostic = error;
            }
            return self.input_response(true);
        }
        if self.window_focused
            && let Some(key) = self.focus
            && self.sliders.contains_key(&key)
        {
            let mut localized = event.clone();
            if let UiEvent::Pointer(pointer) = &mut localized
                && let Some(node) = self
                    .placed
                    .iter()
                    .find(|node| node.node_id == self.ids[&key])
            {
                pointer.position.x -= f64::from(node.content_origin[0]);
                pointer.position.y -= f64::from(node.content_origin[1]);
            }
            let slider = self.sliders.get_mut(&key).unwrap();
            let result = slider.handle_event(&localized);
            let value = slider.take_value();
            if let Some(value) = value {
                if let Err(error) = self.actor.set_widget_slider(key, value) {
                    self.diagnostic = error;
                }
                self.paint_dirty.insert(key);
            }
            return self.input_response(result.is_handled());
        }
        let mut response = self.input_response(false);
        if self.window_focused
            && let Some(key) = self.focus
            && let Some(editor) = self.editors.get_mut(&key)
        {
            let mut localized = event.clone();
            if let UiEvent::Pointer(pointer) = &mut localized
                && let Some(node) = self.placed.iter().find(|n| n.node_id == self.ids[&key])
            {
                pointer.position.x -= node.content_origin[0] as f64;
                pointer.position.y -= node.content_origin[1] as f64;
            }
            let result = editor.handle_event(&localized);
            match &localized {
                UiEvent::Ime(shell::ImeEvent::Preedit { text, .. }) => {
                    self.composing_editor = (!text.is_empty()
                        && self.actor.view().bindings[&key].node.read_only == 0)
                        .then_some(key);
                }
                UiEvent::Ime(
                    shell::ImeEvent::Commit(_)
                    | shell::ImeEvent::Disabled
                    | shell::ImeEvent::DeleteSurrounding { .. },
                ) => {
                    self.composing_editor = None;
                }
                _ => {}
            }
            let edit_shortcut = if let UiEvent::Key(key) = &localized {
                use shell::StandardShortcut;
                [
                    StandardShortcut::Copy,
                    StandardShortcut::Cut,
                    StandardShortcut::Paste,
                    StandardShortcut::SelectAll,
                ]
                .into_iter()
                .any(|shortcut| key.matches_standard_shortcut(shortcut))
            } else {
                false
            };
            response.consume_key_text = result.consumes_key_text()
                || result.clipboard_request().is_some()
                || (result.is_handled() && edit_shortcut);
            response.clipboard = result.clipboard_request().cloned();
            if matches!(response.clipboard, Some(shell::ClipboardRequest::Read)) {
                self.paste_target = Some(key);
            }
        }
        response
    }
    fn take_screenshot_request(&mut self) -> Option<shell::ScreenshotRequest> {
        if !self.capture || self.frames < 2 || (self.capture_wait_focus && !self.window_focused) {
            return None;
        }
        self.capture = false;
        let path = PathBuf::from("reports/native.png");
        std::fs::create_dir_all("reports").unwrap();
        Some(shell::ScreenshotRequest {
            file: std::fs::File::create(&path).unwrap(),
            path,
        })
    }
    fn complete_screenshot(&mut self, _: &Path, result: Result<PathBuf, String>) {
        result.unwrap();
        self.exit = true;
        eprintln!("PASS actual Winit window + Vello capture");
        if let Some(wake) = self.wake.lock().unwrap().as_ref() {
            wake();
        }
    }
    fn take_effect(&mut self) -> Option<shell::EffectRequest> {
        if !std::mem::take(&mut self.exit) {
            return None;
        }
        Some(shell::EffectRequest {
            id: shell::EffectId(1),
            scope: shell::effect::EffectScope::Runtime,
            payload: shell::EffectPayload::ApplicationExit,
        })
    }
}

pub fn proof(actor: Actor, replacement: &Path) -> Result<(), String> {
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("counter.notes");
    let scroll = key("counter.scroll");
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source
        .actor
        .set_text(key("notes"), "a😀bc unsaved")
        .unwrap();
    source.sync()?;
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .map_err(|e| e.to_string())?;
    source.actor.view.renderer.get_mut(&scroll).unwrap().scroll = 137.0;
    source.build_frame(&mut tcx, 720, 620);
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let selection = source.editors[&input].text_selection();
    let ime = source.ime_state();
    let node = source.ids[&input];
    let storage = source.actor.state_address();
    source.replace(replacement)?;
    let nodes = source.build_frame(&mut tcx, 720, 620);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.ime_state(), ime);
    assert_eq!(source.actor.state_address(), storage);
    assert_eq!(
        source.editors[&input].current_value(),
        Some("a😀bc unsaved")
    );
    assert_eq!(source.actor.view.renderer[&scroll].scroll, 137.0);
    let scroll_node = nodes
        .iter()
        .find(|n| n.node_id == source.ids[&scroll])
        .unwrap();
    assert_eq!(scroll_node.scroll.offset[1], 137.0);
    let mut scene = shell::Scene::new();
    shell::scene::build_scene_scaled(
        &mut scene,
        &nodes,
        &mut tcx,
        720,
        620,
        source.base_color(),
        1.0,
    );
    for width in [420, 720] {
        let nodes = source.build_frame(&mut tcx, width, 460);
        assert!(
            nodes
                .iter()
                .filter(|n| source.ids.values().any(|id| *id == n.node_id))
                .all(|n| n.rect[2] <= width as f32 + 0.1 && n.rect[3] <= 460.1)
        );
    }
    let button = source
        .placed
        .iter()
        .find(|n| n.node_id == source.ids[&key("counter.increment")])
        .unwrap();
    let position = shell::Point {
        x: (button.rect[0] + 10.0) as f64,
        y: (button.rect[1] + 10.0) as f64,
    };
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button: Some(shell::PointerButton::Primary),
            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    }
    assert_eq!(source.actor.integer("count"), 10);
    let keyboard = |name: &str, phase, modifiers, repeat, synthetic| {
        UiEvent::Key(shell::KeyEvent {
            phase,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers,
            repeat,
            synthetic,
        })
    };
    use shell::{KeyPhase, Modifiers};
    source.set_focus(None);
    let response = source.handle_event(keyboard(
        "Tab",
        KeyPhase::Down,
        Modifiers::empty(),
        false,
        false,
    ));
    assert!(response.consume_key_text);
    assert_eq!(response.text_input, Some(false));
    let increment = key("counter.increment");
    assert_eq!(source.focus, Some(increment));
    source.build_frame(&mut tcx, 720, 620);
    let paint = source
        .tree
        .get_node_context(source.ids[&increment])
        .unwrap();
    assert_eq!(paint.outline_width, 2.0);
    assert_eq!(paint.outline_color, Some(Color::from_rgb8(31, 88, 155)));
    source.handle_event(keyboard(
        "Enter",
        KeyPhase::Down,
        Modifiers::empty(),
        false,
        false,
    ));
    source.handle_event(keyboard(
        "Enter",
        KeyPhase::Down,
        Modifiers::empty(),
        true,
        false,
    ));
    source.handle_event(keyboard(
        "Enter",
        KeyPhase::Up,
        Modifiers::empty(),
        false,
        false,
    ));
    assert_eq!(source.actor.integer("count"), 20);
    source.handle_event(keyboard(
        " ",
        KeyPhase::Down,
        Modifiers::empty(),
        false,
        false,
    ));
    assert_eq!(source.actor.integer("count"), 20);
    source.replace(replacement)?;
    assert_eq!(source.focus, Some(increment));
    assert_eq!(source.keyboard_pressed, Some(increment));
    source.handle_event(keyboard(
        " ",
        KeyPhase::Down,
        Modifiers::empty(),
        true,
        false,
    ));
    source.handle_event(keyboard(
        " ",
        KeyPhase::Up,
        Modifiers::empty(),
        false,
        false,
    ));
    assert_eq!(source.actor.integer("count"), 30);
    source.handle_event(keyboard(
        "Enter",
        KeyPhase::Down,
        Modifiers::CONTROL,
        false,
        false,
    ));
    source.handle_event(keyboard(
        "Enter",
        KeyPhase::Down,
        Modifiers::empty(),
        false,
        true,
    ));
    assert_eq!(source.actor.integer("count"), 30);
    let response = source.handle_event(keyboard(
        "Tab",
        KeyPhase::Down,
        Modifiers::empty(),
        false,
        false,
    ));
    assert_eq!(source.focus, Some(input));
    assert_eq!(response.text_input, Some(true));
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    source.handle_event(keyboard(
        "Tab",
        KeyPhase::Down,
        Modifiers::SHIFT,
        false,
        false,
    ));
    assert_eq!(source.focus, Some(increment));
    source.handle_event(keyboard(
        " ",
        KeyPhase::Down,
        Modifiers::empty(),
        false,
        false,
    ));
    source.handle_event(UiEvent::Focus(false));
    assert!(source.keyboard_pressed.is_none());
    assert!(source.ime_state().is_none());
    source.handle_event(UiEvent::Focus(true));
    source.handle_event(keyboard(
        " ",
        KeyPhase::Up,
        Modifiers::empty(),
        false,
        false,
    ));
    assert_eq!(source.actor.integer("count"), 30);
    source.handle_event(keyboard(
        "Tab",
        KeyPhase::Down,
        Modifiers::SHIFT,
        false,
        false,
    ));
    assert_eq!(source.focus, Some(input));
    source.handle_event(keyboard(
        "Tab",
        KeyPhase::Down,
        Modifiers::empty(),
        false,
        false,
    ));
    assert_eq!(source.focus, Some(increment));
    assert_eq!(source.actor.state_address(), storage);
    assert_eq!(source.actor.text("notes"), "a😀bc unsaved");
    for sequence in [
        vec![
            (PointerPhase::Down, position),
            (PointerPhase::Up, shell::Point { x: 0.0, y: 0.0 }),
            (PointerPhase::Up, position),
        ],
        vec![
            (PointerPhase::Down, position),
            (PointerPhase::Cancel, position),
            (PointerPhase::Up, position),
        ],
    ] {
        for (phase, position) in sequence {
            source.handle_event(UiEvent::Pointer(shell::PointerEvent {
                phase,
                position,
                button: Some(shell::PointerButton::Primary),
                buttons: if phase == PointerPhase::Down { 1 } else { 0 },
                modifiers: Default::default(),
                properties: Default::default(),
            }));
        }
        assert_eq!(
            source.actor.integer("count"),
            30,
            "release outside or cancellation must not leave a stale press for another release"
        );
    }
    // The widget already implements clipboard semantics; verify that the source
    // preserves its request and routes the resulting paste to the same editor.
    let primary = if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL
    };
    source.actor.set_text(key("notes"), "a😀bc")?;
    source.sync()?;
    source.set_focus(Some(input));
    let select_all = source.handle_event(keyboard("a", KeyPhase::Down, primary, false, false));
    assert!(select_all.consume_key_text);
    source.prepare_for_event(&mut tcx);
    let copy = source.handle_event(keyboard("c", KeyPhase::Down, primary, false, false));
    assert_eq!(
        copy.clipboard,
        Some(shell::ClipboardRequest::Write("a😀bc".into()))
    );
    assert!(copy.consume_key_text);
    let cut = source.handle_event(keyboard("x", KeyPhase::Down, primary, false, false));
    assert_eq!(cut.clipboard, copy.clipboard);
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.actor.text("notes"), "");
    source.sync()?;
    let paste = source.handle_event(keyboard("v", KeyPhase::Down, primary, false, false));
    assert_eq!(paste.clipboard, Some(shell::ClipboardRequest::Read));
    assert!(paste.consume_key_text);
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    source.replace(replacement)?;
    source.handle_event(UiEvent::Paste("clipboard 😀".into()));
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.actor.text("notes"), "clipboard 😀");
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    source.sync()?;
    source.handle_event(keyboard("v", KeyPhase::Down, primary, false, false));
    source.set_focus(Some(increment));
    source.set_focus(Some(input));
    let stale = source.handle_event(UiEvent::Paste("wrong target".into()));
    assert!(!stale.handled);
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.actor.text("notes"), "clipboard 😀");
    source.handle_event(keyboard("v", KeyPhase::Down, primary, false, false));
    source.handle_event(UiEvent::Focus(false));
    source.handle_event(UiEvent::Focus(true));
    source.handle_event(UiEvent::Paste("stale after blur".into()));
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.actor.text("notes"), "clipboard 😀");
    source.handle_event(keyboard("a", KeyPhase::Down, primary, false, false));
    source.prepare_for_event(&mut tcx);
    let recovered_key = source.handle_event(keyboard("x", KeyPhase::Down, primary, false, true));
    assert!(recovered_key.consume_key_text);
    assert!(recovered_key.clipboard.is_none());
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.actor.text("notes"), "clipboard 😀");
    println!(
        "PASS native copy/cut/paste requests reach shell, Unicode edits synchronize to durable state, paste/editor survive replacement and focus/blur changes reject stale paste deliveries"
    );
    println!(
        "PASS native Tab/Shift+Tab traversal, visible focus, IME gating, Enter/Space activation, repeat/shortcut/synthetic guards, focus/press retention across reload and window-blur cancellation"
    );

    println!(
        "PASS reused Taffy node and real TextInput instance, text, UTF-16 selection, focus, scroll, stable state, native scene construction, narrow layout"
    );
    Ok(())
}

/// Prove retained widgets and keyed nodes through runtime view topology changes.
pub fn view_proof(actor: Actor, dynamic: &Path) -> Result<(), String> {
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("counter.notes");
    let scroll = key("counter.scroll");
    source
        .actor
        .set_text(key("notes"), "Native keyed topology")
        .unwrap();
    source.sync()?;
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source.actor.view.renderer.get_mut(&scroll).unwrap().scroll = 137.0;
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let input_node = source.ids[&input];
    source.build_frame(&mut tcx, 720, 900);
    let root = source.ids[&key("counter.root")];
    assert!(!source.tree.dirty(root).unwrap());
    source.sync()?;
    assert!(
        !source.tree.dirty(root).unwrap(),
        "unchanged frame must not invalidate layout"
    );
    source.replace(dynamic)?;
    source.build_frame(&mut tcx, 720, 900);
    let row = |id: u64| {
        id.to_le_bytes()
            .into_iter()
            .fold(key("counter.rows"), |hash, byte| {
                (hash ^ byte as u64).wrapping_mul(0x100000001b3)
            })
    };
    let first = source.ids[&row(1)];
    let mut second = None;
    for count in 1..=3 {
        source.actor.dispatch(Message {
            action: key("increment"),
            value: 0,
        })?;
        source.build_frame(&mut tcx, 720, 900);
        assert_eq!(source.ids[&row(1)], first);
        assert_eq!(source.ids[&input], input_node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            widget
        );
        assert_eq!(source.focus, Some(input));
        assert_eq!(
            source.editors[&input].current_value(),
            Some("Native keyed topology")
        );
        assert_eq!(source.actor.view.renderer[&scroll].scroll, 137.0);
        assert_eq!(
            source.ids.contains_key(&key("counter.conditional")),
            count % 2 == 1
        );
        if count <= 2 {
            let second_node = source.ids[&row(2)];
            if let Some(previous) = second {
                assert_eq!(second_node, previous, "reordering must retain the row node");
            }
            second = Some(second_node);
            let y = |id| {
                source
                    .placed
                    .iter()
                    .find(|node| node.node_id == id)
                    .unwrap()
                    .rect[1]
            };
            if count == 1 {
                assert!(y(first) < y(second_node));
            } else {
                assert!(
                    y(second_node) < y(first),
                    "business-keyed rows must follow their new order"
                );
            }
        }
    }
    assert!(!source.ids.contains_key(&row(2)));
    assert!(!source.applied.contains_key(&row(2)));
    assert!(!source.ids.contains_key(&row(3)));
    assert!(source.ids.contains_key(&key("counter.conditional")));
    println!(
        "PASS native nested/keyed view insertion, reordering and removal retain TextInput, focus, scroll and existing Taffy nodes; disposed nodes leave native caches; unchanged frame keeps clean layout"
    );
    Ok(())
}

/// Exercise computed updates and implementation replacement through actual native widgets.
pub fn graph_proof(actor: Actor, replacement: &Path) -> Result<(), String> {
    graph_proof_inner(actor, replacement, false)
}

/// Require direct float/boolean outputs in addition to the shared native graph contract.
pub fn scalar_graph_proof(actor: Actor, replacement: &Path) -> Result<(), String> {
    graph_proof_inner(actor, replacement, true)
}

fn graph_proof_inner(actor: Actor, replacement: &Path, scalars: bool) -> Result<(), String> {
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("graph.notes");
    source
        .actor
        .set_text(key("notes"), "a😀bc computed")
        .unwrap();
    source.sync()?;
    if scalars {
        assert_eq!(source.actor.view.bindings[&key("graph.fraction")].text, "0");
        assert_eq!(
            source.actor.view.bindings[&key("graph.positive")].text,
            "false"
        );
    }
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .unwrap();
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    let slot = source.actor.slot_address("count");
    source.actor.dispatch(Message {
        action: key("step"),
        value: 1,
    })?;
    source.build_frame(&mut tcx, 720, 620);
    let selection = source.editors[&input].text_selection();
    assert_eq!(selection.as_ref().map(|s| (s.anchor, s.head)), Some((1, 4)));
    assert_eq!(source.actor.computed_evaluations("double"), 2);
    assert_eq!(source.actor.computed_evaluations("caption"), 2);
    source.replace(replacement)?;
    let nodes = source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.actor.integer("count"), 1);
    assert_eq!(source.actor.slot_address("count"), slot);
    assert_eq!(
        source.actor.computed("double"),
        Some(&crate::Value::Integer(3))
    );
    assert_eq!(source.actor.computed_evaluations("double"), 1);
    assert_eq!(source.actor.computed_evaluations("caption"), 1);
    for key in [key("graph.caption.one"), key("graph.caption.two")] {
        assert_eq!(
            nodes
                .iter()
                .find(|n| n.node_id == source.ids[&key])
                .unwrap()
                .paint
                .text
                .as_deref(),
            Some("Derived: 3")
        );
    }
    if scalars {
        for (name, expected) in [("graph.fraction", "1.5"), ("graph.positive", "true")] {
            let id = source.ids[&key(name)];
            assert_eq!(
                nodes
                    .iter()
                    .find(|n| n.node_id == id)
                    .unwrap()
                    .paint
                    .text
                    .as_deref(),
                Some(expected)
            );
        }
        assert_eq!(source.actor.computed_evaluations("derived.fraction"), 1);
        assert_eq!(source.actor.computed_evaluations("derived.positive"), 1);
        println!("PASS native direct float/boolean computed bindings publish replacement values");
    }
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(
        source.editors[&input].current_value(),
        Some("a😀bc computed")
    );
    println!(
        "PASS native computed DAG updates both texts once, reload changes derived output without state recreation, real editor/selection/focus retained"
    );
    Ok(())
}

/// Native widget/event proof for the independent SDK editor; no desktop input.
pub fn editor_proof(actor: Actor, replacement: &Path, file: &Path) -> Result<(), String> {
    editor_smoke(actor, replacement, file).map(drop)
}

/// Return the exercised retained source for actual window presentation.
pub fn editor_smoke(
    actor: Actor,
    replacement: &Path,
    file: &Path,
) -> Result<NativeSource<Actor>, String> {
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    source
        .actor
        .set_text(key("editor.path"), file.to_str().ok_or("non UTF8 path")?)?;
    source.sync()?;
    let input = source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.node.slot == key("editor.draft"))
        .unwrap()
        .0
        .to_owned();
    let path_input = source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.node.slot == key("editor.path"))
        .unwrap()
        .0
        .to_owned();
    let button = source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| {
            binding.node.kind == BUTTON && binding.node.action == key("editor.save")
        })
        .unwrap()
        .0
        .to_owned();
    let load_button = source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, b)| b.node.action == key("editor.load") && b.node.kind == BUTTON)
        .unwrap()
        .0
        .to_owned();
    let old_shortcut = source.actor.view.bindings[&button].node.shortcut;
    let keyboard = |name: &str| {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Default::default(),
            repeat: false,
            synthetic: false,
        })
    };
    source.handle_event(UiEvent::Focus(true));
    for width in [720, 420, 320] {
        let nodes = source.build_frame(&mut tcx, width, 480);
        let save_rect = nodes
            .iter()
            .find(|n| n.node_id == source.ids[&button])
            .unwrap()
            .rect;
        let load_rect = nodes
            .iter()
            .find(|n| n.node_id == source.ids[&load_button])
            .unwrap()
            .rect;
        assert_eq!(save_rect[1], load_rect[1]);
        assert!(save_rect[2] + 11.9 <= load_rect[0]);
        assert!(load_rect[2] <= width as f32 - 23.9);
        assert!((save_rect[2] - save_rect[0] - (load_rect[2] - load_rect[0])).abs() < 0.2);
        let mut bottom = 0.0;
        for widget in [path_input, input, button] {
            let node = nodes
                .iter()
                .find(|node| node.node_id == source.ids[&widget])
                .unwrap();
            let [x, y, right, lower] = node.rect;
            assert!(x >= 0.0 && y >= bottom && right > x && lower > y);
            assert!(right <= width as f32 + 0.1 && lower <= 480.1);
            bottom = lower;
        }
    }
    source.handle_event(keyboard("Tab"));
    assert_eq!(source.focus, Some(path_input));
    source.handle_event(keyboard("Tab"));
    assert_eq!(source.focus, Some(input));
    source.handle_event(UiEvent::TextInput("a😀bc draft".into()));
    source.prepare_for_event(&mut tcx); // Flush deferred widget edits before the next event.
    assert_eq!(source.actor.text("editor.draft"), "a😀bc draft");
    source.build_frame(&mut tcx, 720, 480);
    source.handle_event(keyboard("Tab"));
    assert_eq!(source.focus, Some(button));
    source.handle_event(keyboard("Enter"));
    assert!(source.actor.boolean("editor.busy"));
    assert_eq!(source.actor.text("editor.snapshot"), "a😀bc draft");
    assert!(
        source
            .actor
            .view
            .bindings
            .values()
            .filter(|b| b.node.kind == BUTTON)
            .all(|b| b.node.disabled == 1)
    );
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 720, 480);
    let selection = source.editors[&input].text_selection();
    assert_eq!(selection.as_ref().map(|s| (s.anchor, s.head)), Some((1, 3)));
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let slot = source.actor.slot_address("editor.draft");
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 420, 480);
    assert!(
        source
            .actor
            .view
            .bindings
            .values()
            .filter(|b| b.node.kind == BUTTON)
            .all(|b| b.node.disabled == 1)
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.actor.slot_address("editor.draft"), slot);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    source.handle_event(UiEvent::TextInput("X".into()));
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.actor.text("editor.draft"), "aXbc draft");
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.actor.boolean("editor.busy") && Instant::now() < deadline {
        source.poll_async();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!source.actor.boolean("editor.busy"));
    assert_eq!(std::fs::read(file).unwrap(), "a😀bc draft".as_bytes());
    assert_eq!(source.actor.text("editor.draft"), "aXbc draft");
    let nodes = source.build_frame(&mut tcx, 420, 480);
    assert!(
        nodes
            .iter()
            .any(|node| node.paint.text.as_deref() == Some("Unsaved changes."))
    );
    assert_eq!(source.editors[&input].current_value(), Some("aXbc draft"));
    let button_node = nodes
        .iter()
        .find(|node| node.node_id == source.ids[&button])
        .unwrap();
    let position = shell::Point {
        x: ((button_node.rect[0] + button_node.rect[2]) / 2.0) as f64,
        y: ((button_node.rect[1] + button_node.rect[3]) / 2.0) as f64,
    };
    assert_eq!(source.hit(position), Some(button));
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button: Some(shell::PointerButton::Primary),
            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    }
    assert_eq!(source.focus, Some(button));
    assert!(source.actor.boolean("editor.busy"));
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.actor.boolean("editor.busy") && Instant::now() < deadline {
        source.poll_async();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!source.actor.boolean("editor.busy"));
    assert_eq!(source.actor.text("editor.status"), "Saved.");
    assert_eq!(std::fs::read(file).unwrap(), b"aXbc draft");
    let load_button = *source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| {
            binding.node.kind == BUTTON && binding.node.action == key("editor.load")
        })
        .unwrap()
        .0;
    std::fs::write(file, "Loaded native 😀").map_err(|e| e.to_string())?;
    source.handle_event(keyboard("Tab")); // Save → Load in mounted focus order.
    assert_eq!(source.focus, Some(load_button));
    source.handle_event(keyboard("Enter"));
    assert!(source.actor.boolean("editor.busy"));
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":10,"head":10}}"#)
        .unwrap();
    source.prepare_for_event(&mut tcx);
    source.handle_event(UiEvent::TextInput("!".into()));
    source.prepare_for_event(&mut tcx);
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.actor.boolean("editor.busy") && Instant::now() < deadline {
        source.poll_async();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!source.actor.boolean("editor.busy"));
    assert_eq!(source.actor.text("editor.draft"), "aXbc draft!");
    assert!(
        source
            .actor
            .text("editor.status")
            .contains("Kept your edits")
    );
    let nodes = source.build_frame(&mut tcx, 420, 480);
    assert_eq!(source.editors[&input].current_value(), Some("aXbc draft!"));
    let node = nodes
        .iter()
        .find(|node| node.node_id == source.ids[&load_button])
        .unwrap();
    assert!(node.rect[2] <= 420.1 && node.rect[3] <= 480.1);
    let position = shell::Point {
        x: ((node.rect[0] + node.rect[2]) / 2.0) as f64,
        y: ((node.rect[1] + node.rect[3]) / 2.0) as f64,
    };
    assert_eq!(source.hit(position), Some(load_button));
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button: Some(shell::PointerButton::Primary),
            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.actor.boolean("editor.busy") && Instant::now() < deadline {
        source.poll_async();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!source.actor.boolean("editor.busy"));
    source.build_frame(&mut tcx, 420, 480);
    assert_eq!(source.actor.text("editor.draft"), "Loaded native 😀");
    assert_eq!(
        source.editors[&input].current_value(),
        Some("Loaded native 😀")
    );
    assert_eq!(source.actor.text("editor.status"), "Loaded.");
    assert_eq!(source.actor.slot_address("editor.draft"), slot);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.actor.io_result_count(), 0);
    source.set_focus(Some(input));
    let end = source.actor.text("editor.draft").encode_utf16().count();
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(&format!(
            r#"{{"selection":{{"anchor":{end},"head":{end}}}}}"#
        ))
        .unwrap();
    source.prepare_for_event(&mut tcx);
    source.handle_event(keyboard("Enter"));
    source.prepare_for_event(&mut tcx);
    source.handle_event(UiEvent::TextInput("Second line".into()));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source.actor.text("editor.draft"),
        "Loaded native 😀\nSecond line"
    );
    assert_eq!(
        source.actor.view.bindings[&button].node.shortcut,
        's' as u32
    );
    if old_shortcut != 's' as u32 {
        let mut old_key = keyboard(&char::from_u32(old_shortcut).unwrap().to_string());
        if let UiEvent::Key(event) = &mut old_key {
            event.modifiers = if cfg!(target_os = "macos") {
                shell::Modifiers::META
            } else {
                shell::Modifiers::CONTROL
            };
        }
        source.handle_event(old_key);
        assert!(
            !source.actor.boolean("editor.busy"),
            "old shortcut survived replacement"
        );
    }
    let mut save_key = keyboard("s");
    if let UiEvent::Key(event) = &mut save_key {
        event.modifiers = if cfg!(target_os = "macos") {
            shell::Modifiers::META
        } else {
            shell::Modifiers::CONTROL
        };
    }
    // Blurred, synthetic and modified chords must not start a save.
    source.handle_event(UiEvent::Focus(false));
    source.handle_event(save_key.clone());
    assert!(!source.actor.boolean("editor.busy"));
    source.handle_event(UiEvent::Focus(true));
    let mut synthetic = save_key.clone();
    if let UiEvent::Key(event) = &mut synthetic {
        event.synthetic = true;
    }
    source.handle_event(synthetic);
    assert!(!source.actor.boolean("editor.busy"));
    let mut shifted = save_key.clone();
    if let UiEvent::Key(event) = &mut shifted {
        event.modifiers |= shell::Modifiers::SHIFT;
    }
    source.handle_event(shifted);
    assert!(!source.actor.boolean("editor.busy"));
    source.handle_event(UiEvent::Ime(shell::ImeEvent::Preedit {
        text: "仮".into(),
        cursor: None,
    }));
    assert_eq!(source.composing_editor, Some(input));
    source.replace(replacement)?;
    assert_eq!(source.composing_editor, Some(input));
    source.handle_event(save_key.clone());
    assert!(
        !source.actor.boolean("editor.busy"),
        "save must not execute during IME preedit"
    );
    source.handle_event(UiEvent::Ime(shell::ImeEvent::Disabled));
    assert!(source.handle_event(save_key.clone()).consume_key_text);
    assert!(source.actor.boolean("editor.busy"));
    assert_eq!(source.focus, Some(input));
    if let UiEvent::Key(event) = &mut save_key {
        event.repeat = true;
    }
    assert!(source.handle_event(save_key).consume_key_text);
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.actor.boolean("editor.busy") && Instant::now() < deadline {
        source.poll_async();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!source.actor.boolean("editor.busy"));
    assert_eq!(
        std::fs::read(file).unwrap(),
        "Loaded native 😀\nSecond line".as_bytes()
    );
    source.set_semantics_enabled(true);
    let semantic_text = |source: &NativeSource<Actor>| {
        let snapshot = source.semantic_snapshot().unwrap();
        assert_eq!(snapshot.focus, Some(input));
        let node = snapshot.nodes.iter().find(|node| node.id == input).unwrap();
        assert_eq!(node.role, shell::SemanticRole::TextInput);
        node.value.clone().unwrap()
    };
    let before_ime = source.actor.text("editor.draft").to_owned();
    let end = before_ime.encode_utf16().count();
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(&format!(
            r#"{{"selection":{{"anchor":{end},"head":{end}}}}}"#
        ))
        .unwrap();
    source.prepare_for_event(&mut tcx);
    source.handle_event(UiEvent::Ime(shell::ImeEvent::Preedit {
        text: "にほん".into(),
        cursor: None,
    }));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 420, 480);
    assert_eq!(source.actor.text("editor.draft"), before_ime);
    assert_eq!(
        source.editors[&input].current_value(),
        Some(before_ime.as_str())
    );
    assert_eq!(semantic_text(&source), before_ime);
    let composing_snapshot = source.semantic_snapshot().unwrap();
    let composing_geometry = source.ime_state();
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 420, 480);
    assert_eq!(source.composing_editor, Some(input));
    assert_eq!(source.ime_state(), composing_geometry);
    assert_eq!(semantic_text(&source), before_ime);
    assert!(Arc::ptr_eq(
        &composing_snapshot,
        &source.semantic_snapshot().unwrap()
    ));
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.actor.slot_address("editor.draft"), slot);
    assert_eq!(source.actor.text("editor.draft"), before_ime);
    source.handle_event(UiEvent::Ime(shell::ImeEvent::Commit("日本".into())));
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.composing_editor, None);
    let committed = format!("{before_ime}日本");
    assert_eq!(source.actor.text("editor.draft"), committed);
    source.build_frame(&mut tcx, 420, 480);
    assert_eq!(semantic_text(&source), committed);
    assert!(source.semantic_snapshot().unwrap().revision > composing_snapshot.revision);
    // The trailing empty preedit sent by input methods must not duplicate/remove the commit.
    source.handle_event(UiEvent::Ime(shell::ImeEvent::Preedit {
        text: String::new(),
        cursor: None,
    }));
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.actor.text("editor.draft"), committed);
    let mut save_committed = keyboard("s");
    if let UiEvent::Key(event) = &mut save_committed {
        event.modifiers = if cfg!(target_os = "macos") {
            shell::Modifiers::META
        } else {
            shell::Modifiers::CONTROL
        };
    }
    source.handle_event(save_committed);
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.actor.boolean("editor.busy") && Instant::now() < deadline {
        source.poll_async();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!source.actor.boolean("editor.busy"));
    assert_eq!(std::fs::read(file).unwrap(), committed.as_bytes());
    // IME byte deletion and UTF16 selection are different coordinate systems.
    source.actor.set_text(key("editor.draft"), "A😀日本Z")?;
    source.sync()?;
    source.build_frame(&mut tcx, 420, 480);
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":3,"head":3}}"#)
        .unwrap();
    source.handle_event(UiEvent::Ime(shell::ImeEvent::Preedit {
        text: "仮".into(),
        cursor: None,
    }));
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.composing_editor, Some(input));
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 420, 480);
    source.handle_event(UiEvent::Ime(shell::ImeEvent::DeleteSurrounding {
        before_bytes: 4,
        after_bytes: 3,
    }));
    source.prepare_for_event(&mut tcx);
    assert_eq!(source.actor.text("editor.draft"), "A本Z");
    source.build_frame(&mut tcx, 420, 480);
    assert_eq!(semantic_text(&source), "A本Z");
    assert_eq!(source.composing_editor, None);
    assert_eq!(source.actor.slot_address("editor.draft"), slot);
    assert_eq!(source.focus, Some(input));
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    let mut save_deleted = keyboard("s");
    if let UiEvent::Key(event) = &mut save_deleted {
        event.modifiers = if cfg!(target_os = "macos") {
            shell::Modifiers::META
        } else {
            shell::Modifiers::CONTROL
        };
    }
    source.handle_event(save_deleted);
    assert!(source.actor.boolean("editor.busy"));
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.actor.boolean("editor.busy") && Instant::now() < deadline {
        source.poll_async();
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!source.actor.boolean("editor.busy"));
    assert_eq!(std::fs::read(file).unwrap(), "A本Z".as_bytes());
    println!(
        "PASS native semantic Unicode values track IME commit/delete, exclude preedit and retain snapshot identity across compatible DLL replacement; routed IME byte deletion after DLL replacement clears composition and preserves valid Unicode durable state"
    );
    println!(
        "PASS native routed IME preedit survives real DLL replacement without mutating durable draft; Unicode commit applies once and new implementation saves exact contents"
    );
    println!(
        "PASS native TextArea Enter inserts newline and primary save shortcut after reload writes exact Unicode contents; blurred/synthetic/extra-modifier chords do not save"
    );
    println!(
        "PASS native editor keyboard/pointer loads preserve edits during pending read, retry displays Unicode contents in retained input and releases results"
    );
    println!(
        "PASS native editor Tab/Enter, pointer save, bounded 320/420/720 layouts and Unicode text events save through replacement, retain widget/slot/focus/UTF16 selection and preserve edits made during saving"
    );
    Ok(source)
}

/// Long multiline document viewport identity across actual implementation replacement.
pub fn textarea_proof(actor: Actor, replacement: &Path) -> Result<(), String> {
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let text: String = (0..100)
        .map(|index| format!("Row {index} — 😀\n"))
        .collect();
    source.actor.set_text(key("editor.draft"), &text)?;
    source.sync()?;
    let input = *source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, b)| b.node.kind == TEXTAREA)
        .unwrap()
        .0;
    source.handle_event(UiEvent::Focus(true));
    source.set_focus(Some(input));
    source.build_frame(&mut tcx, 420, 480);
    let offset = text[..text.find("Row 20").unwrap()].encode_utf16().count();
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(&format!(
            r#"{{"selection":{{"anchor":{offset},"head":{offset}}}}}"#
        ))
        .unwrap();
    source.build_frame(&mut tcx, 420, 480);
    let before = source.ime_state().unwrap();
    assert!(before.multiline);
    let node = source
        .placed
        .iter()
        .find(|node| node.node_id == source.ids[&input])
        .unwrap();
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: shell::Point {
            x: (node.rect[0] + 20.0) as f64,
            y: (node.rect[1] + 20.0) as f64,
        },
        delta_x: 0.0,
        delta_y: 20.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    source.build_frame(&mut tcx, 420, 480);
    let scrolled = source.ime_state().unwrap();
    assert!(
        scrolled.cursor_area[1] < before.cursor_area[1] - 5.0,
        "wheel must change multiline viewport: {:?} -> {:?}",
        before.cursor_area,
        scrolled.cursor_area
    );
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: shell::Point {
            x: 5000.0,
            y: 5000.0,
        },
        delta_x: 0.0,
        delta_y: 20.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    source.build_frame(&mut tcx, 420, 480);
    assert_eq!(
        source.ime_state(),
        Some(scrolled.clone()),
        "wheel outside editor must not scroll focused textarea"
    );
    let selection = source.editors[&input].text_selection();
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let slot = source.actor.slot_address("editor.draft");
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 420, 480);
    assert_eq!(source.ime_state(), Some(scrolled.clone()));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.actor.slot_address("editor.draft"), slot);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.actor.text("editor.draft"), text);
    source.set_focus(None);
    let node = source
        .placed
        .iter()
        .find(|node| node.node_id == source.ids[&input])
        .unwrap();
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: shell::Point {
            x: (node.rect[0] + 20.0) as f64,
            y: (node.rect[1] + 20.0) as f64,
        },
        delta_x: 0.0,
        delta_y: 10.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    assert_eq!(source.focus, None); // Scrolling does not steal keyboard focus.
    source.set_focus(Some(input));
    source.build_frame(&mut tcx, 420, 480);
    assert!(source.ime_state().unwrap().cursor_area[1] < scrolled.cursor_area[1] - 5.0);
    println!(
        "PASS long TextArea wheel changes viewport and real DLL replacement retains IME geometry/selection/focus/widget/slot and full Unicode document"
    );
    Ok(())
}

pub fn migration_proof(actor: Actor, replacement: &Path) -> Result<(), String> {
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("document.editor");
    source.actor.set_text(key("notes"), "a😀bc document")?;
    source.actor.dispatch(Message {
        action: key("increment"),
        value: 17,
    })?;
    source.sync()?;
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 720, 620);
    let selection = source.editors[&input].text_selection();
    assert_eq!(
        selection.as_ref().map(|value| (value.anchor, value.head)),
        Some((1, 4))
    );
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    let touch = source.actor.slot_address("touch");
    let storage = source.actor.state_address();
    source.replace(replacement)?;
    let nodes = source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.actor.state_address(), storage);
    assert_eq!(source.actor.slot_address("touch"), touch);
    assert_eq!(source.actor.text("count"), "17");
    assert_eq!(source.actor.text("document.notes"), "a😀bc document");
    assert_eq!(source.actor.slot_address("notes"), 0);
    assert_eq!(
        source.actor.view.bindings[&input].node.slot,
        key("document.notes")
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(
        source.editors[&input].current_value(),
        Some("a😀bc document")
    );
    assert_eq!(
        nodes
            .iter()
            .find(|node| node.node_id == source.ids[&key("document.count")])
            .unwrap()
            .paint
            .text
            .as_deref(),
        Some("17")
    );
    // Real native edits after rebinding must target the new durable slot.
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .attribute_changed("value", "edited after migration");
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(
        source.actor.text("document.notes"),
        "edited after migration"
    );
    assert_eq!(source.actor.slot_address("notes"), 0);
    println!(
        "PASS native schema migration retains editor/Taffy node/focus/UTF16 selection, preserves unchanged slot allocation, rebinds renamed state and routes subsequent edits to the new slot"
    );
    Ok(())
}

/// Test protocol for the real watcher without a desktop/window driver.
pub fn watch_probe(mut actor: Actor, config: ReloadConfig) -> Result<(), String> {
    use std::io::{BufRead, Write};
    actor.dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    actor.set_text(key("notes"), "unsaved watcher state")?;
    let storage = actor.state_address();
    let slot = actor.slot_address("count");
    let (watcher, builds) = watch(config, Arc::new(Mutex::new(None)));
    while !watcher.ready.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(10));
    }
    let (commands, inputs) = mpsc::channel();
    thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            if commands.send(line.unwrap_or_default()).is_err() {
                break;
            }
        }
    });
    println!("READY");
    std::io::stdout().flush().unwrap();
    let mut sequence = 0;
    loop {
        actor.tick()?;
        if let Ok(command) = inputs.try_recv() {
            if command == "quit" {
                drop(watcher);
                println!("STOPPED");
                return Ok(());
            }
            if command == "step" {
                actor.dispatch(Message {
                    action: key("increment"),
                    value: 0,
                })?;
            }
            println!(
                "STATE {}",
                serde_json::json!({"count":actor.integer("count"),"label":actor.projected_view().bindings.get(&key("label")).map(|binding| binding.text.as_str()),"label_width":actor.projected_view().bindings.get(&key("label")).map(|binding| binding.node.presentation.width),"async_text":actor.projected_view().bindings.get(&key("async.result")).map(|binding| binding.text.as_str()),"async_generation":actor.projected_view().bindings.get(&key("async.generation")).map(|binding| binding.text.as_str()),"async_boundary":actor.projected_view().bindings.values().find(|binding| binding.text.starts_with("Async boundary ") && binding.node.semantics.hidden == 0).map(|binding| binding.text.as_str()),"label_tone":actor.projected_view().bindings.get(&key("label")).map(|binding| binding.node.presentation.text_tone),"label_max_width":actor.projected_view().bindings.get(&key("label")).map(|binding| binding.node.presentation.max_width),"details_visible":actor.projected_view().bindings.get(&key("details")).map(|binding| binding.node.semantics.hidden == 0),"details_evaluations":actor.projected_view().scope_dependencies.get(&key("details")).map(|scope| scope.evaluations),"fallback_visible":actor.projected_view().bindings.values().any(|binding| binding.text == "Fallback" && binding.node.semantics.hidden == 0),"row_order":actor.projected_view().order.iter().filter_map(|id| actor.projected_view().bindings.get(id)).filter_map(|binding| (binding.text == "a" || binding.text == "b").then_some(binding.text.as_str())).collect::<Vec<_>>(),"sequence":sequence,"implementation":actor.implementation_version()})
            );
            std::io::stdout().flush().unwrap();
        }
        match builds.recv_timeout(Duration::from_millis(20)) {
            Ok(result) => {
                let mut error = Some(result.message);
                if let Some(path) = result.artifact {
                    error = actor.reload(&path).err();
                    let _ = std::fs::remove_file(path);
                }
                sequence += 1;
                assert_eq!(actor.state_address(), storage);
                assert_eq!(actor.slot_address("count"), slot);
                assert_eq!(actor.text("notes"), "unsaved watcher state");
                println!(
                    "EVENT {}",
                    serde_json::json!({"sequence":sequence,"error":error,"count":actor.integer("count"),"implementation":actor.implementation_version()})
                );
                std::io::stdout().flush().unwrap();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err("watch worker exited".into()),
        }
    }
}

/// Real retained renderer/event proof for two actors sharing all local widget keys.
pub fn family_proof(
    base: &Path,
    replacement: &Path,
    missing_mount: Option<&Path>,
) -> Result<(), String> {
    use crate::{ActorId, Runtime, native_model::NativeFamily};
    let parent = ActorId(100);
    let child = ActorId(101);
    let mut runtime = Runtime::new(".runtime");
    runtime.spawn(parent, base)?;
    runtime.spawn_child(parent, child, base)?;
    let snapshot = runtime.compose(parent)?;
    let input = snapshot.widget(child, key("counter.notes")).unwrap();
    snapshot.set_text(&mut runtime, input, "a😀bc child document")?;
    let model = NativeFamily::new(runtime, parent)?;
    let mut source = NativeSource::without_watch(model)?;
    let mut tcx = TextContext::new();
    let parent_input = source.actor.widget(parent, key("counter.notes")).unwrap();
    let child_input = source.actor.widget(child, key("counter.notes")).unwrap();
    assert_ne!(parent_input, child_input);
    source.focus = Some(child_input);
    source
        .editors
        .get_mut(&child_input)
        .unwrap()
        .focus_changed(true);
    source
        .editors
        .get_mut(&child_input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .map_err(|e| e.to_string())?;
    let scroll = source.actor.widget(child, key("counter.scroll")).unwrap();
    source
        .actor
        .view_mut()
        .renderer
        .get_mut(&scroll)
        .unwrap()
        .scroll = 137.0;
    source.build_frame(&mut tcx, 720, 1400);
    let widget = (&**source.editors.get(&child_input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&child_input];
    let selection = source.editors[&child_input].text_selection();
    let slot = source
        .actor
        .runtime()
        .actor(child)
        .unwrap()
        .slot_address("notes");
    let old_y = source
        .placed
        .iter()
        .find(|placed| placed.node_id == node)
        .unwrap()
        .rect[1];
    let child_root = source.actor.widget(child, key("counter.root")).unwrap();
    let original_parent = source.actor.view().bindings[&child_root].node.parent;
    assert!(
        source
            .actor
            .mount_child(child, key("counter.increment"))
            .is_err()
    );
    assert!(
        source
            .actor
            .mount_child(child, key("missing.column"))
            .is_err()
    );
    assert_eq!(
        source.actor.view().bindings[&child_root].node.parent,
        original_parent
    );
    source.actor.mount_child(child, key("counter.details"))?;
    assert!(source.poll_async());
    source.build_frame(&mut tcx, 720, 1400);
    assert!(
        source
            .placed
            .iter()
            .find(|placed| placed.node_id == node)
            .unwrap()
            .rect[1]
            < old_y
    );
    assert_eq!(
        source.actor.view().bindings[&child_root].node.parent,
        source.actor.widget(parent, key("counter.details")).unwrap()
    );
    assert_eq!(
        (&**source.editors.get(&child_input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.ids[&child_input], node);
    assert_eq!(source.focus, Some(child_input));
    assert_eq!(source.editors[&child_input].text_selection(), selection);

    source.replace(replacement)?; // parent-only replacement
    source.actor.reload_actor(child, replacement)?;
    source.build_frame(&mut tcx, 720, 1400);
    assert_eq!(
        source.actor.view().bindings[&child_root].node.parent,
        source.actor.widget(parent, key("counter.details")).unwrap()
    );
    assert_eq!(
        source.actor.widget(child, key("counter.notes")),
        Some(child_input)
    );
    assert_eq!(source.ids[&child_input], node);
    assert_eq!(
        (&**source.editors.get(&child_input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.focus, Some(child_input));
    assert_eq!(source.editors[&child_input].text_selection(), selection);
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(child)
            .unwrap()
            .slot_address("notes"),
        slot
    );
    assert_eq!(source.actor.view().renderer[&scroll].scroll, 137.0);
    if let Some(path) = missing_mount {
        source.replace(path)?;
        source.build_frame(&mut tcx, 720, 1400);
        assert_eq!(
            source.actor.view().bindings[&child_root].node.parent,
            source.actor.widget(parent, key("counter.root")).unwrap()
        );
        assert_eq!(
            (&**source.editors.get(&child_input).unwrap() as *const TextInput) as usize,
            widget
        );
        assert_eq!(source.focus, Some(child_input));
        assert_eq!(source.editors[&child_input].text_selection(), selection);
        source.replace(replacement)?;
        source.build_frame(&mut tcx, 720, 1400);
        assert_eq!(
            source.actor.view().bindings[&child_root].node.parent,
            source.actor.widget(parent, key("counter.details")).unwrap()
        );
        assert_eq!(
            (&**source.editors.get(&child_input).unwrap() as *const TextInput) as usize,
            widget
        );
    }
    let button_key = source
        .actor
        .widget(child, key("counter.increment"))
        .unwrap();
    let button = source
        .placed
        .iter()
        .find(|node| node.node_id == source.ids[&button_key])
        .unwrap();
    let position = shell::Point {
        x: (button.rect[0] + 10.0) as f64,
        y: (button.rect[1] + 10.0) as f64,
    };
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button: Some(shell::PointerButton::Primary),
            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    }
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(child)
            .unwrap()
            .integer("count"),
        10
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(parent)
            .unwrap()
            .integer("count"),
        0
    );
    source
        .editors
        .get_mut(&child_input)
        .unwrap()
        .attribute_changed("value", "Edited child only");
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source.actor.runtime().actor(child).unwrap().text("notes"),
        "Edited child only"
    );
    assert_eq!(
        source.actor.runtime().actor(parent).unwrap().text("notes"),
        ""
    );
    source.build_frame(&mut tcx, 720, 1400);
    let parent_widget =
        (&**source.editors.get(&parent_input).unwrap() as *const TextInput) as usize;
    source.actor.reconcile_children(parent, &[])?;
    source.build_frame(&mut tcx, 720, 1400);
    assert!(!source.editors.contains_key(&child_input));
    assert_eq!(
        (&**source.editors.get(&parent_input).unwrap() as *const TextInput) as usize,
        parent_widget
    );
    source.actor.reconcile_children(parent, &[(child, base)])?;
    source.build_frame(&mut tcx, 720, 1400);
    assert_ne!(
        source.actor.widget(child, key("counter.notes")),
        Some(child_input)
    );
    let recreated_root = source.actor.widget(child, key("counter.root")).unwrap();
    assert_eq!(
        source.actor.view().bindings[&recreated_root].node.parent,
        source.actor.widget(parent, key("counter.root")).unwrap()
    );
    assert!(source.actor.set_widget_text(input, "stale edit").is_err());
    let root_id = ActorId(200);
    let old_child = ActorId(201);
    let new_child = ActorId(202);
    let mut runtime = Runtime::new(".runtime");
    runtime.spawn(root_id, base)?;
    runtime.spawn_child(root_id, old_child, base)?;
    let mut redraw = NativeSource::without_watch(NativeFamily::new(runtime, root_id)?)?;
    redraw.build_frame(&mut tcx, 720, 1400);
    let (wake_tx, wake_rx) = mpsc::channel();
    redraw.set_wake_callback(Arc::new(move || {
        let _ = wake_tx.send(());
    }));
    let evaluations = redraw
        .actor
        .view()
        .bindings
        .values()
        .map(|binding| binding.evaluations)
        .sum::<u64>();
    redraw
        .actor
        .reconcile_children(root_id, &[(new_child, base)])?;
    wake_rx
        .try_recv()
        .expect("family reconciliation wakes the native loop");
    assert_eq!(
        redraw
            .actor
            .view()
            .bindings
            .values()
            .map(|binding| binding.evaluations)
            .sum::<u64>(),
        evaluations
    );
    assert!(
        redraw.poll_async(),
        "equal-size actor replacement must request a native redraw"
    );
    redraw.build_frame(&mut tcx, 720, 1400);
    assert!(
        !redraw.poll_async(),
        "unchanged native projection must remain idle"
    );
    let scroll_key = redraw
        .actor
        .widget(new_child, key("counter.scroll"))
        .unwrap();
    redraw
        .actor
        .view_mut()
        .renderer
        .get_mut(&scroll_key)
        .unwrap()
        .scroll = 91.0;
    assert!(redraw.poll_async(), "scroll-only changes must repaint");
    redraw.build_frame(&mut tcx, 720, 1400);
    assert!(!redraw.poll_async());

    println!(
        "PASS composed native actor views, unique widget identities, retained child editor/selection/focus/scroll/state, per-actor reload, explicit mounts and fallback, pointer/edit routing, child disposal and stale event rejection"
    );
    Ok(())
}

/// Controlled disposable-fixture proof of independent live compilation targets.
pub fn family_watch_proof(
    parent_config: ReloadConfig,
    child_config: ReloadConfig,
) -> Result<(), String> {
    use crate::{Runtime, native_model::NativeFamily};
    let parent = ActorId(300);
    let child = ActorId(301);
    let parent_source = parent_config.manifest.parent().unwrap().join("src/lib.rs");
    let child_source = child_config.manifest.parent().unwrap().join("src/lib.rs");
    let parent_original = std::fs::read_to_string(&parent_source).map_err(|e| e.to_string())?;
    let child_original = std::fs::read_to_string(&child_source).map_err(|e| e.to_string())?;
    let child_base = child_config
        .manifest
        .parent()
        .unwrap()
        .join("initial-child.so");
    std::fs::copy(&child_config.artifact, &child_base).map_err(|e| e.to_string())?;
    let mut runtime = Runtime::new(".runtime");
    runtime.spawn(parent, &parent_config.artifact)?;
    runtime.spawn_child(parent, child, &child_config.artifact)?;
    let parent_version = runtime.actor(parent).unwrap().implementation_version();
    let child_version = runtime.actor(child).unwrap().implementation_version();
    let parent_lifecycle = runtime.actor(parent).unwrap().lifecycle();
    let child_lifecycle = runtime.actor(child).unwrap().lifecycle();
    let parent_slot = runtime.actor(parent).unwrap().slot_address("count");
    let child_slot = runtime.actor(child).unwrap().slot_address("count");
    let mut source = NativeSource::without_watch(NativeFamily::new(runtime, parent)?)?;
    let mut tcx = TextContext::new();
    let input = source.actor.widget(child, key("input")).unwrap();
    source.actor.set_widget_text(input, "unsaved child text")?;
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source.build_frame(&mut tcx, 720, 1000);
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    source.watch_actor(parent, parent_config.clone())?;
    source.watch_actor(child, child_config.clone())?;
    assert!(source.watch_actor(child, child_config).is_err());
    let deadline = Instant::now() + Duration::from_secs(30);
    while source.actor_watches.values().any(|target| {
        !target
            .watcher
            .as_ref()
            .unwrap()
            .ready
            .load(Ordering::Acquire)
    }) {
        if Instant::now() > deadline {
            return Err("watch startup timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    std::fs::write(
        &parent_source,
        format!("{parent_original}\ncompile_error!(\"intentional parent failure\");\n"),
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        &child_source,
        child_original.replace("checked_add(1)", "checked_add(3)"),
    )
    .map_err(|e| e.to_string())?;
    while source
        .actor
        .runtime()
        .actor(child)
        .unwrap()
        .implementation_version()
        == child_version
        || !source
            .actor_diagnostic(parent)
            .is_some_and(|message| message.contains("intentional parent failure"))
    {
        source.poll_async();
        if Instant::now() > deadline {
            return Err("independent watch reload timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(parent)
            .unwrap()
            .implementation_version(),
        parent_version
    );
    let child_updated = source
        .actor
        .runtime()
        .actor(child)
        .unwrap()
        .implementation_version();
    for actor in [parent, child] {
        let button = source.actor.widget(actor, key("button")).unwrap();
        source.actor.dispatch_widget(button)?;
    }
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(parent)
            .unwrap()
            .integer("count"),
        1
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(child)
            .unwrap()
            .integer("count"),
        3
    );
    std::fs::write(
        &parent_source,
        parent_original.replace("checked_add(1)", "checked_add(2)"),
    )
    .map_err(|e| e.to_string())?;
    while source
        .actor
        .runtime()
        .actor(parent)
        .unwrap()
        .implementation_version()
        == parent_version
    {
        source.poll_async();
        if Instant::now() > deadline {
            return Err("parent recovery timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(child)
            .unwrap()
            .implementation_version(),
        child_updated
    );
    assert_eq!(
        source.actor.runtime().actor(parent).unwrap().lifecycle(),
        parent_lifecycle
    );
    assert_eq!(
        source.actor.runtime().actor(child).unwrap().lifecycle(),
        child_lifecycle
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(parent)
            .unwrap()
            .slot_address("count"),
        parent_slot
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(child)
            .unwrap()
            .slot_address("count"),
        child_slot
    );
    source.build_frame(&mut tcx, 720, 1000);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(
        source.actor.runtime().actor(child).unwrap().text("notes"),
        "unsaved child text"
    );
    // A real oversized cdylib must be rejected before creating a pending snapshot.
    let healthy_parent = source
        .actor
        .runtime()
        .actor(parent)
        .unwrap()
        .implementation_version();
    let oversized = format!(
        "{}\n#[used]\n#[unsafe(no_mangle)]\npub static OVERSIZE: [u8; 33 * 1024 * 1024] = [7; 33 * 1024 * 1024];\n",
        parent_original.replace("checked_add(1)", "checked_add(2)")
    );
    std::fs::write(&parent_source, oversized).map_err(|e| e.to_string())?;
    while !source
        .actor_diagnostic(parent)
        .is_some_and(|message| message.contains("32 MiB"))
    {
        source.poll_async();
        if Instant::now() > deadline {
            return Err("oversized artifact rejection timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(std::fs::metadata(&parent_config.artifact).unwrap().len() > crate::artifact::MAX_BYTES);
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(parent)
            .unwrap()
            .implementation_version(),
        healthy_parent
    );
    assert!(!std::fs::read_dir(".runtime").unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("pending-")
    }));
    let button = source.actor.widget(parent, key("button")).unwrap();
    let count = source
        .actor
        .runtime()
        .actor(parent)
        .unwrap()
        .integer("count");
    source.actor.dispatch_widget(button)?;
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(parent)
            .unwrap()
            .integer("count"),
        count + 2
    );
    std::fs::write(
        &parent_source,
        parent_original.replace("checked_add(1)", "checked_add(2)"),
    )
    .map_err(|e| e.to_string())?;
    while source
        .actor_diagnostic(parent)
        .is_some_and(|message| message.contains("32 MiB"))
    {
        source.poll_async();
        if Instant::now() > deadline {
            return Err("oversized artifact recovery timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        std::fs::metadata(&parent_config.artifact).unwrap().len() <= crate::artifact::MAX_BYTES
    );
    // Wait for a real new child build but deliberately don't deliver it yet.
    std::fs::write(
        &child_source,
        child_original.replace("checked_add(1)", "checked_add(4)"),
    )
    .map_err(|e| e.to_string())?;
    let result = loop {
        if let Ok(result) = source.actor_watches[&child].builds.try_recv() {
            break result;
        }
        if Instant::now() > deadline {
            return Err("stale child build timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    };
    let pending_path = result.artifact.clone().ok_or("child compilation failed")?;
    assert!(pending_path.exists());
    let (sender, receiver) = mpsc::channel();
    sender.send(result).unwrap();
    source.actor_watches.get_mut(&child).unwrap().builds = receiver.into();
    source.actor.reconcile_children(parent, &[])?;
    source
        .actor
        .reconcile_children(parent, &[(child, &child_base)])?;
    let recreated_version = source
        .actor
        .runtime()
        .actor(child)
        .unwrap()
        .implementation_version();
    assert_ne!(
        source.actor.runtime().actor(child).unwrap().lifecycle(),
        child_lifecycle
    );
    source.poll_async();
    assert!(!source.actor_watches.contains_key(&child));
    assert!(!pending_path.exists());
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(child)
            .unwrap()
            .implementation_version(),
        recreated_version
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(child)
            .unwrap()
            .integer("count"),
        0
    );
    println!(
        "PASS independent parent/child source watches, compiler failure isolation/recovery, retained lifecycle/state/editor/focus, oversized artifact rejection, stale queued build rejection and cleanup"
    );
    Ok(())
}

/// Two feature selections share one Cargo output filename but retain separate snapshots.
pub fn shared_watch_proof(first: ReloadConfig, second: ReloadConfig) -> Result<(), String> {
    assert_eq!(first.manifest, second.manifest);
    assert_eq!(first.artifact, second.artifact);
    let source = first.manifest.parent().unwrap().join("src/lib.rs");
    let original = std::fs::read_to_string(&source).map_err(|e| e.to_string())?;
    let (first_watch, first_results) = watch(first, Arc::new(Mutex::new(None)));
    let (second_watch, second_results) = watch(second, Arc::new(Mutex::new(None)));
    let deadline = Instant::now() + Duration::from_secs(30);
    while !first_watch.ready.load(Ordering::Acquire) || !second_watch.ready.load(Ordering::Acquire)
    {
        if Instant::now() > deadline {
            return Err("shared watcher startup timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    std::fs::write(
        source,
        original.replace("const BIAS: i64 = 0;", "const BIAS: i64 = 1;"),
    )
    .map_err(|e| e.to_string())?;
    let first = first_results
        .recv_timeout(Duration::from_secs(30))
        .map_err(|e| e.to_string())?;
    let second = second_results
        .recv_timeout(Duration::from_secs(30))
        .map_err(|e| e.to_string())?;
    let first = first.artifact.ok_or(first.message)?;
    let second = second.artifact.ok_or(second.message)?;
    assert_ne!(first, second);
    let mut a = Actor::new(400, Path::new(".runtime"))?;
    let mut b = Actor::new(401, Path::new(".runtime"))?;
    a.reload(&first)?;
    b.reload(&second)?;
    std::fs::remove_file(first).map_err(|e| e.to_string())?;
    std::fs::remove_file(second).map_err(|e| e.to_string())?;
    a.dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    b.dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    assert_eq!(a.integer("count"), 2);
    assert_eq!(b.integer("count"), 11);
    drop(first_watch);
    drop(second_watch);
    println!(
        "PASS simultaneous shared-crate feature builds retain distinct immutable snapshots and correct behavior"
    );
    Ok(())
}

/// End-to-end keyed declaration, admission rollback, retained native widgets and disposal.
/// Parent orientation replacement must preserve independently owned child actors.
pub fn declared_row_proof(base: &Path, horizontal: &Path, child: &Path) -> Result<(), String> {
    use crate::{Runtime, native_model::NativeFamily};
    let parent = ActorId(733);
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("counter", child)?;
    runtime.spawn(parent, base)?;
    let slots: Vec<_> = runtime
        .actor(parent)
        .unwrap()
        .view
        .order
        .iter()
        .copied()
        .filter(|key| runtime.actor(parent).unwrap().view.bindings[key].node.kind == CHILD)
        .collect();
    assert_eq!(slots.len(), 2);
    let children = [
        runtime.child(parent, slots[0]).unwrap(),
        runtime.child(parent, slots[1]).unwrap(),
    ];
    let state = runtime.actor(children[0]).unwrap().state_address();
    let slot = runtime.actor(children[0]).unwrap().slot_address("notes");
    let lifecycle = runtime.actor(children[0]).unwrap().lifecycle();
    let mut source = NativeSource::without_watch(NativeFamily::new(runtime, parent)?)?;
    let input = source
        .actor
        .widget(children[0], key("counter.notes"))
        .unwrap();
    let button = source
        .actor
        .widget(children[0], key("counter.increment"))
        .unwrap();
    let mounts = [
        source.actor.widget(parent, slots[0]).unwrap(),
        source.actor.widget(parent, slots[1]).unwrap(),
    ];
    source.actor.set_widget_text(input, "a😀bc durable child")?;
    source.sync()?;
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
        .unwrap();
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 720, 1600);
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let input_node = source.ids[&input];
    let mount_nodes = [source.ids[&mounts[0]], source.ids[&mounts[1]]];
    let selection = source.editors[&input].text_selection();
    assert_eq!(selection.as_ref().map(|s| (s.anchor, s.head)), Some((1, 3)));
    source.actor.dispatch_widget(button)?;
    for (path, is_row) in [(horizontal, true), (base, false)] {
        source.replace(path)?;
        source.build_frame(&mut tcx, 720, 1600);
        for index in 0..2 {
            assert_eq!(
                source.actor.runtime().child(parent, slots[index]),
                Some(children[index])
            );
            assert_eq!(source.ids[&mounts[index]], mount_nodes[index]);
        }
        let actor = source.actor.runtime().actor(children[0]).unwrap();
        assert_eq!(actor.state_address(), state);
        assert_eq!(actor.slot_address("notes"), slot);
        assert_eq!(actor.lifecycle(), lifecycle);
        assert_eq!(actor.text("notes"), "a😀bc durable child");
        assert_eq!(actor.integer("count"), 1);
        assert_eq!(
            source
                .actor
                .runtime()
                .actor(children[1])
                .unwrap()
                .integer("count"),
            0
        );
        assert_eq!(source.focus, Some(input));
        assert_eq!(source.editors[&input].text_selection(), selection);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            widget
        );
        assert_eq!(source.ids[&input], input_node);
        let first = source
            .placed
            .iter()
            .find(|node| node.node_id == mount_nodes[0])
            .unwrap()
            .rect;
        let second = source
            .placed
            .iter()
            .find(|node| node.node_id == mount_nodes[1])
            .unwrap()
            .rect;
        if is_row {
            assert!((first[1] - second[1]).abs() < 0.1);
            assert!(second[2] + 11.9 <= first[0]);
            assert!(
                second[0] >= 23.9 && first[2] <= 696.1,
                "mounts outside padded viewport: {first:?}, {second:?}"
            );
        } else {
            assert!((first[0] - second[0]).abs() < 0.1);
            assert!(first[3] + 11.9 <= second[1]);
        }
    }
    source.replace(horizontal)?;
    source.actor.mount_child(children[0], key("parent.root"))?;
    source.sync()?;
    source.build_frame(&mut tcx, 720, 1600);
    let root = source.actor.widget(parent, key("parent.root")).unwrap();
    let child_root = source
        .actor
        .widget(children[0], key("counter.root"))
        .unwrap();
    assert_eq!(source.actor.view().bindings[&child_root].node.parent, root);
    assert_eq!(
        source
            .tree
            .style(source.ids[&child_root])
            .unwrap()
            .flex_grow,
        1.0
    );
    assert_eq!(source.ids[&input], input_node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    source.actor.dispatch_widget(button)?;
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(children[0])
            .unwrap()
            .integer("count"),
        2
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(children[0])
            .unwrap()
            .state_address(),
        state
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(children[0])
            .unwrap()
            .slot_address("notes"),
        slot
    );
    println!(
        "PASS explicit child mount to Row retains native editor/focus/selection/state and routes action to its owning actor"
    );
    println!(
        "PASS declared child actors retain identity/lifecycle/state/slot/count/editor/focus/Unicode selection through parent Column/Row/Column reload and reordered nonoverlapping mount geometry"
    );
    Ok(())
}

pub fn declared_children_proof(
    base: &Path,
    replacement: &Path,
    rejected: &Path,
    child_library: &Path,
    recursive: &Path,
    nested: &Path,
    nested_replacement: &Path,
) -> Result<(), String> {
    use crate::{Runtime, native_model::NativeFamily};
    let parent = ActorId(42);
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("counter", child_library)?;
    runtime.register_implementation("bad", Path::new("/missing/child-implementation.so"))?;
    runtime.spawn(parent, base)?;
    assert_eq!(runtime.len(), 3);
    let slots: Vec<_> = runtime
        .actor(parent)
        .unwrap()
        .view
        .order
        .iter()
        .copied()
        .filter(|key| runtime.actor(parent).unwrap().view.bindings[key].node.kind == CHILD)
        .collect();
    assert_eq!(slots.len(), 2);
    let first = runtime.child(parent, slots[0]).unwrap();
    let second = runtime.child(parent, slots[1]).unwrap();
    assert_ne!(first, second);
    assert_eq!(runtime.parent(first), Some(parent));
    let child_state = runtime.actor(first).unwrap().state_address();
    let child_slot = runtime.actor(first).unwrap().slot_address("notes");
    let lifecycle = runtime.actor(first).unwrap().lifecycle();
    let mut source = NativeSource::without_watch(NativeFamily::new(runtime, parent)?)?;
    let watch_config = ReloadConfig::from_environment(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap(),
    );
    source.watch_implementation("counter", watch_config.clone())?;
    assert!(
        source
            .watch_implementation("counter", watch_config)
            .is_err()
    );
    assert!(source.actor_watches.is_empty());
    assert_eq!(source.implementation_watches.len(), 1);
    // Explicit lifecycle watches override group delivery for this recipient.
    source.watch_actor(
        first,
        ReloadConfig::from_environment(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap(),
        ),
    )?;
    assert_eq!(source.actor_watches[&first].lifecycle, lifecycle);

    let mut tcx = TextContext::new();
    let input = source.actor.widget(first, key("counter.notes")).unwrap();
    source
        .actor
        .set_widget_text(input, "durable declared child")?;
    source.sync()?;
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .map_err(|e| e.to_string())?;
    source.build_frame(&mut tcx, 720, 1600);
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    let selection = source.editors[&input].text_selection();
    let button = source
        .actor
        .widget(first, key("counter.increment"))
        .unwrap();
    source.actor.dispatch_widget(button)?;
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 1600);
    assert_eq!(source.actor.runtime().child(parent, slots[0]), Some(first));
    assert_eq!(source.actor.runtime().child(parent, slots[1]), Some(second));
    let order: Vec<_> = source
        .actor
        .root_actor()
        .view
        .order
        .iter()
        .copied()
        .filter(|key| source.actor.root_actor().view.bindings[key].node.kind == CHILD)
        .collect();
    assert_eq!(order, vec![slots[1], slots[0]]);
    assert_eq!(
        source.actor.runtime().actor(first).unwrap().state_address(),
        child_state
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(first)
            .unwrap()
            .slot_address("notes"),
        child_slot
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(first)
            .unwrap()
            .integer("count"),
        1
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(second)
            .unwrap()
            .integer("count"),
        0
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.ids[&input], node);
    let tab = || {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: "Tab".into(),
            key_without_modifiers: "Tab".into(),
            code: "Tab".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Default::default(),
            repeat: false,
            synthetic: false,
        })
    };
    source.set_focus(None);
    for _ in 0..3 {
        source.handle_event(tab());
    }
    assert_eq!(
        source.focus,
        source.actor.widget(second, key("counter.increment")),
        "Tab must follow mounted/reordered child placement rather than actor allocation order"
    );
    source.set_focus(Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    let version = source.actor.root_actor().implementation_version();
    let parent_slot = source.actor.root_actor().slot_address("show");
    let old_order = source.actor.view().order.clone();
    assert!(source.replace(rejected).is_err());
    assert_eq!(source.actor.root_actor().implementation_version(), version);
    assert!(
        source
            .actor
            .root_actor_mut()
            .dispatch(Message {
                action: key("bad"),
                value: 0
            })
            .is_err()
    );
    assert_eq!(source.actor.root_actor().integer("show"), 1);
    assert_eq!(source.actor.root_actor().slot_address("show"), parent_slot);
    assert_eq!(source.actor.view().order, old_order);
    assert_eq!(source.actor.runtime().len(), 3);
    assert_eq!(
        std::fs::read_dir(".runtime")
            .unwrap()
            .filter(|entry| entry
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("actor-"))
            .count(),
        3
    );
    // Queue a real DLL for the retiring lifecycle. Disposal must drain it,
    // never deliver it to a newly declared instance of the same local key.
    let stale_path = Path::new(".runtime/declared-stale.so");
    std::fs::copy(child_library, stale_path).map_err(|e| e.to_string())?;
    let (sender, receiver) = mpsc::sync_channel(1);
    let target = source.actor_watches.get_mut(&first).unwrap();
    target.watcher.take();
    target.builds = receiver.into();
    sender
        .send(BuildResult {
            artifact: Some(stale_path.into()),
            message: "retired result".into(),
        })
        .unwrap();
    let child_button = source
        .actor
        .widget(first, key("counter.increment"))
        .unwrap();
    source.set_focus(Some(child_button));
    let space = |phase| {
        UiEvent::Key(shell::KeyEvent {
            phase,
            key: " ".into(),
            key_without_modifiers: " ".into(),
            code: "Space".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Default::default(),
            repeat: false,
            synthetic: false,
        })
    };
    source.handle_event(space(shell::KeyPhase::Down));
    assert_eq!(source.keyboard_pressed, Some(child_button));
    let hide = source.actor.widget(parent, key("parent.hide")).unwrap();
    source.actor.dispatch_widget(hide)?;
    source.build_frame(&mut tcx, 720, 1600);
    assert_eq!(source.actor.runtime().len(), 1);
    assert!(source.keyboard_pressed.is_none());
    source.handle_event(space(shell::KeyPhase::Up));

    assert!(source.actor_watches.is_empty());
    assert!(!stale_path.exists());

    assert!(source.actor.runtime().child(parent, slots[0]).is_none());
    assert!(source.editors.is_empty());
    assert!(source.focus.is_none());
    let show = source.actor.widget(parent, key("parent.show")).unwrap();
    source.actor.dispatch_widget(show)?;
    source.build_frame(&mut tcx, 720, 1600);
    let recreated = source.actor.runtime().child(parent, slots[0]).unwrap();
    assert_ne!(recreated, first);
    assert!(source.actor_watches.is_empty());
    assert_eq!(source.implementation_watches.len(), 1);

    assert_ne!(
        source.actor.runtime().actor(recreated).unwrap().lifecycle(),
        lifecycle
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(recreated)
            .unwrap()
            .integer("count"),
        0
    );
    assert!(source.actor.set_widget_text(input, "stale edit").is_err());
    let fresh_input = source
        .actor
        .widget(recreated, key("counter.notes"))
        .unwrap();
    source.set_focus(Some(fresh_input));
    let primary = if cfg!(target_os = "macos") {
        shell::Modifiers::META
    } else {
        shell::Modifiers::CONTROL
    };
    let read = source.handle_event(UiEvent::Key(shell::KeyEvent {
        phase: shell::KeyPhase::Down,
        key: "v".into(),
        key_without_modifiers: "v".into(),
        code: "KeyV".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: primary,
        repeat: false,
        synthetic: false,
    }));
    assert_eq!(read.clipboard, Some(shell::ClipboardRequest::Read));
    assert_eq!(source.paste_target, Some(fresh_input));
    source.actor.dispatch_widget(hide)?;
    source.build_frame(&mut tcx, 720, 1600);
    assert!(source.paste_target.is_none());
    source.actor.dispatch_widget(show)?;
    source.build_frame(&mut tcx, 720, 1600);
    let latest_child = source.actor.runtime().child(parent, slots[0]).unwrap();
    let latest_input = source
        .actor
        .widget(latest_child, key("counter.notes"))
        .unwrap();
    source.set_focus(Some(latest_input));
    assert!(
        !source
            .handle_event(UiEvent::Paste("retired child paste".into()))
            .handled
    );
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(latest_child)
            .unwrap()
            .text("notes"),
        ""
    );
    drop(source);
    let mut nested_runtime = Runtime::new(".runtime");
    nested_runtime.register_implementation("counter", child_library)?;
    nested_runtime.register_implementation("branch", base)?;
    nested_runtime.spawn(parent, nested)?;
    assert_eq!(nested_runtime.len(), 7);
    let branch = nested_runtime.child(parent, slots[0]).unwrap();
    let leaf = nested_runtime.child(branch, slots[0]).unwrap();
    let leaf_state = nested_runtime.actor(leaf).unwrap().state_address();
    nested_runtime.actor_mut(leaf).unwrap().dispatch(Message {
        action: key("increment"),
        value: 0,
    })?;
    nested_runtime.reload(parent, nested_replacement)?;
    nested_runtime.reload(branch, replacement)?;
    assert_eq!(nested_runtime.child(parent, slots[0]), Some(branch));
    assert_eq!(nested_runtime.child(branch, slots[0]), Some(leaf));
    assert_eq!(
        nested_runtime.actor(leaf).unwrap().state_address(),
        leaf_state
    );
    assert_eq!(nested_runtime.actor(leaf).unwrap().integer("count"), 1);
    nested_runtime
        .actor_mut(branch)
        .unwrap()
        .dispatch(Message {
            action: key("hide"),
            value: 0,
        })?;
    assert!(nested_runtime.tick().is_empty());
    assert_eq!(nested_runtime.len(), 5);
    assert!(nested_runtime.actor(leaf).is_none());
    nested_runtime
        .actor_mut(branch)
        .unwrap()
        .dispatch(Message {
            action: key("show"),
            value: 0,
        })?;
    assert!(nested_runtime.tick().is_empty());
    assert_eq!(nested_runtime.len(), 7);
    assert_ne!(nested_runtime.child(branch, slots[0]), Some(leaf));
    assert!(nested_runtime.remove(parent));
    assert!(nested_runtime.is_empty());
    drop(nested_runtime);
    // Recursive declarations stop at the actor staging bound and roll back the whole tree.
    let mut recursive_runtime = Runtime::new(".runtime");
    recursive_runtime.register_implementation("recursive", recursive)?;
    assert!(
        recursive_runtime
            .spawn(parent, recursive)
            .unwrap_err()
            .contains("capacity")
    );
    assert!(recursive_runtime.is_empty());
    assert_eq!(std::fs::read_dir(".runtime").unwrap().count(), 0);
    println!(
        "PASS SDK child declarations create keyed actors, parent reload/reorder retains state/editor/focus/selection, child-load failure rolls back parent code/state/view, hide/show disposes/recreates lifecycles nested reload/disposal works and recursive staging remains bounded"
    );
    Ok(())
}

/// Accepted implementation bytes outlive all current instances of that type.
pub fn accepted_children_proof(
    parent_library: &Path,
    seed: &Path,
    replacement: &Path,
    rejected: &Path,
) -> Result<(), String> {
    use crate::Runtime;
    let root = ActorId(42);
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("counter", seed)?;
    runtime.spawn(root, parent_library)?;
    let slot = runtime
        .actor(root)
        .unwrap()
        .view
        .order
        .iter()
        .copied()
        .find(|key| runtime.actor(root).unwrap().view.bindings[key].node.kind == CHILD)
        .unwrap();
    let child = runtime.child(root, slot).unwrap();
    let address = runtime.actor(child).unwrap().slot_address("count");
    runtime.tell(
        child,
        Message {
            action: key("increment"),
            value: 0,
        },
    )?;
    assert!(runtime.tick().is_empty());
    let transient = Path::new(".runtime/accepted-input.so");
    crate::artifact::copy(replacement, transient)?;
    runtime.reload(child, transient)?;
    std::fs::remove_file(transient).map_err(|e| e.to_string())?;
    assert_eq!(runtime.actor(child).unwrap().slot_address("count"), address);
    assert_eq!(runtime.actor(child).unwrap().integer("count"), 1);
    let version = runtime.actor(child).unwrap().implementation_version();
    // Each successful publication replaces the prior owned cache entry.
    for _ in 0..4 {
        runtime.reload(child, replacement)?;
    }
    let snapshots = || {
        std::fs::read_dir(".runtime")
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("accepted-"))
            .count()
    };
    assert_eq!(snapshots(), 1);
    let invalid = Path::new(".runtime/rejected-input.so");
    crate::artifact::write(invalid, b"invalid shared library")?;
    assert!(runtime.reload(child, invalid).is_err());
    std::fs::remove_file(invalid).map_err(|e| e.to_string())?;
    assert!(runtime.reload(child, rejected).is_err());
    assert_eq!(snapshots(), 1);
    assert_eq!(
        runtime.actor(child).unwrap().implementation_version(),
        version
    );
    runtime.actor_mut(root).unwrap().dispatch(Message {
        action: key("hide"),
        value: 0,
    })?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.len(), 1);
    assert_eq!(snapshots(), 1);
    runtime.actor_mut(root).unwrap().dispatch(Message {
        action: key("show"),
        value: 0,
    })?;
    assert!(runtime.tick().is_empty());
    let fresh = runtime.child(root, slot).unwrap();
    assert_ne!(fresh, child);
    assert_eq!(
        runtime.actor(fresh).unwrap().implementation_version(),
        version
    );
    runtime.tell(
        fresh,
        Message {
            action: key("increment"),
            value: 0,
        },
    )?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(fresh).unwrap().integer("count"), 10);
    runtime.remove(root);
    assert_eq!(snapshots(), 1);
    drop(runtime);
    assert_eq!(snapshots(), 0);
    println!(
        "PASS new declared instances use retained accepted code after source deletion and last-instance disposal; failed replacement preserves cache and repeated reload keeps one owned snapshot"
    );
    Ok(())
}

pub fn implementation_watch_proof(parent: &Path, config: ReloadConfig) -> Result<(), String> {
    use crate::{Runtime, native_model::NativeFamily};
    let root = ActorId(42);
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("counter", &config.artifact)?;
    runtime.spawn(root, parent)?;
    let children = runtime.declared_children();
    assert_eq!(children.len(), 2);
    let ids: Vec<_> = children.iter().map(|(id, _)| *id).collect();
    let slots: Vec<_> = ids
        .iter()
        .map(|id| runtime.actor(*id).unwrap().slot_address("count"))
        .collect();
    for id in &ids {
        runtime.tell(
            *id,
            Message {
                action: key("increment"),
                value: 0,
            },
        )?;
    }
    assert!(runtime.tick().is_empty());
    let mut source = NativeSource::without_watch(NativeFamily::new(runtime, root)?)?;
    let input = source.actor.widget(ids[0], key("counter.notes")).unwrap();
    source.actor.set_widget_text(input, "group keeps notes")?;
    source.sync()?;
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .map_err(|e| e.to_string())?;
    let selection = source.editors[&input].text_selection();
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let versions: Vec<_> = ids
        .iter()
        .map(|id| {
            source
                .actor
                .runtime()
                .actor(*id)
                .unwrap()
                .implementation_version()
        })
        .collect();
    let source_path = config.manifest.parent().unwrap().join("src/lib.rs");
    let original = std::fs::read_to_string(&source_path).map_err(|e| e.to_string())?;
    source.watch_implementation("counter", config)?;
    let identity = key("counter");
    assert_eq!(source.implementation_watches.len(), 1);
    assert!(source.actor_watches.is_empty());
    let deadline = Instant::now() + Duration::from_secs(30);
    while !source.implementation_watches[&identity]
        .watcher
        .as_ref()
        .unwrap()
        .ready
        .load(Ordering::Acquire)
    {
        if Instant::now() >= deadline {
            return Err("group watcher did not become ready".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    std::fs::write(
        &source_path,
        format!("{original}\ncompile_error!(\"intentional group failure\");\n"),
    )
    .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while ids.iter().any(|id| {
        source
            .actor_diagnostic(*id)
            .is_none_or(|s| !s.contains("intentional group failure"))
    }) {
        source.poll_async();
        if Instant::now() >= deadline {
            return Err("group compiler failure was not delivered".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    for (index, id) in ids.iter().enumerate() {
        assert_eq!(
            source
                .actor
                .runtime()
                .actor(*id)
                .unwrap()
                .implementation_version(),
            versions[index]
        );
        assert_eq!(
            source.actor.runtime().actor(*id).unwrap().integer("count"),
            1
        );
    }
    // A well-compiled candidate accepts the first state, rejects the second.
    let second_button = source
        .actor
        .widget(ids[1], key("counter.increment"))
        .unwrap();
    source.actor.dispatch_widget(second_button)?;
    let rejected = original.replace("fn view(cx: &Cx<'_>) -> Result<View> {", "fn view(cx: &Cx<'_>) -> Result<View> { if COUNT.get(cx)? == 2 { return Err(Error::InvalidView(\"state-specific group rejection\")); }");
    assert_ne!(rejected, original);
    std::fs::write(&source_path, &rejected).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while ids.iter().any(|id| {
        source
            .actor_diagnostic(*id)
            .is_none_or(|message| !message.contains("state-specific group rejection"))
    }) {
        source.poll_async();
        if Instant::now() >= deadline {
            return Err("state-specific group rejection was not delivered".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    for (index, id) in ids.iter().enumerate() {
        assert_eq!(
            source
                .actor
                .runtime()
                .actor(*id)
                .unwrap()
                .implementation_version(),
            versions[index]
        );
        assert_eq!(
            source.actor.runtime().actor(*id).unwrap().integer("count"),
            (index + 1) as i64
        );
    }
    let replacement = original.replace("{ 1 }", "{ 2 }");
    assert_ne!(replacement, original);
    std::fs::write(&source_path, &replacement).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while ids.iter().enumerate().any(|(index, id)| {
        source
            .actor
            .runtime()
            .actor(*id)
            .unwrap()
            .implementation_version()
            == versions[index]
    }) {
        source.poll_async();
        if Instant::now() >= deadline {
            return Err("group successful build was not delivered".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    source.sync()?;
    for (index, id) in ids.iter().enumerate() {
        assert_eq!(
            source
                .actor
                .runtime()
                .actor(*id)
                .unwrap()
                .slot_address("count"),
            slots[index]
        );
        assert_eq!(
            source.actor.runtime().actor(*id).unwrap().integer("count"),
            (index + 1) as i64
        );
        let button = source.actor.widget(*id, key("counter.increment")).unwrap();
        source.actor.dispatch_widget(button)?;
        assert_eq!(
            source.actor.runtime().actor(*id).unwrap().integer("count"),
            (index + 3) as i64
        );
    }
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    assert_eq!(
        source.actor.runtime().actor(ids[0]).unwrap().text("notes"),
        "group keeps notes"
    );
    let hide = source.actor.widget(root, key("parent.hide")).unwrap();
    source.actor.dispatch_widget(hide)?;
    source.sync()?;
    assert_eq!(source.actor.runtime().len(), 1);
    // Compile while no members exist. A bounded result stays implementation-owned.
    std::fs::write(&source_path, original.replace("{ 1 }", "{ 3 }")).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let result = loop {
        source.poll_async();
        if let Ok(result) = source.implementation_watches[&identity].builds.try_recv()
            && result.artifact.is_some()
        {
            break result;
        }
        if Instant::now() >= deadline {
            return Err("memberless implementation build did not finish".into());
        }
        thread::sleep(Duration::from_millis(20));
    };
    // Return the observed result to a bounded queue to exercise normal delivery.
    let (sender, receiver) = mpsc::sync_channel(1);
    source
        .implementation_watches
        .get_mut(&identity)
        .unwrap()
        .watcher
        .take();
    source
        .implementation_watches
        .get_mut(&identity)
        .unwrap()
        .builds = receiver.into();
    sender.send(result).unwrap();
    let show = source.actor.widget(root, key("parent.show")).unwrap();
    source.actor.dispatch_widget(show)?;
    source.poll_async();
    source.sync()?;
    let fresh = source.actor.runtime().declared_children();
    assert_eq!(fresh.len(), 2);
    for (id, _) in fresh {
        assert!(!ids.contains(&id));
        let button = source.actor.widget(id, key("counter.increment")).unwrap();
        source.actor.dispatch_widget(button)?;
        assert_eq!(
            source.actor.runtime().actor(id).unwrap().integer("count"),
            3
        );
    }
    drop(source);
    println!(
        "PASS one implementation watcher atomically distributes real builds, rejects state-specific second-recipient failure for the whole group, retains native editor/focus/selection and delivers code compiled with zero members to recreated actors"
    );
    Ok(())
}

pub fn group_children_proof(base: &Path, expanded: &Path, counter: &Path) -> Result<(), String> {
    use crate::Runtime;
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("counter", counter)?;
    let a = ActorId(10);
    let b = ActorId(11);
    runtime.spawn(a, base)?;
    runtime.spawn(b, base)?;
    for id in 100..157 {
        runtime.spawn(ActorId(id), counter)?;
    }
    assert_eq!(runtime.len(), 63);
    let version = runtime.actor(a).unwrap().implementation_version();
    let slot = runtime.actor(a).unwrap().slot_address("show");
    let children = runtime.declared_children();
    assert!(
        runtime
            .reload_group(&[a, b], expanded)
            .unwrap_err()
            .contains("capacity")
    );
    assert_eq!(runtime.len(), 63);
    assert_eq!(runtime.actor(a).unwrap().implementation_version(), version);
    assert_eq!(runtime.actor(b).unwrap().implementation_version(), version);
    assert_eq!(runtime.actor(a).unwrap().slot_address("show"), slot);
    assert_eq!(runtime.declared_children(), children);
    assert_eq!(runtime.children.as_ref().unwrap().borrow().staged, 0);
    assert!(
        runtime
            .children
            .as_ref()
            .unwrap()
            .borrow()
            .pending
            .is_empty()
    );
    assert_eq!(
        std::fs::read_dir(".runtime")
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("actor-"))
            .count(),
        63
    );
    runtime.remove(ActorId(100));
    runtime.reload_group(&[a, b], expanded)?;
    assert_eq!(runtime.len(), 64);
    assert_eq!(runtime.declared_children().len(), 6);
    assert_eq!(runtime.actor(a).unwrap().slot_address("show"), slot);
    assert_eq!(runtime.children.as_ref().unwrap().borrow().staged, 0);
    drop(runtime);
    println!(
        "PASS group admissions count reservations across prepared child trees, reject aggregate capacity without publishing/leaking children, then commit both trees within the 64-actor bound"
    );
    Ok(())
}

pub fn supersession_proof(parent: &Path, config: ReloadConfig) -> Result<(), String> {
    use crate::{Runtime, native_model::NativeFamily};
    let root = ActorId(42);
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("counter", &config.artifact)?;
    runtime.spawn(root, parent)?;
    let ids: Vec<_> = runtime
        .declared_children()
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    let slots: Vec<_> = ids
        .iter()
        .map(|id| runtime.actor(*id).unwrap().slot_address("count"))
        .collect();
    let mut source = NativeSource::without_watch(NativeFamily::new(runtime, root)?)?;
    let directory = config.manifest.parent().unwrap().to_owned();
    let path = directory.join("src/lib.rs");
    let original = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    source.watch_implementation("counter", config)?;
    let identity = key("counter");
    let watcher = source.implementation_watches[&identity]
        .watcher
        .as_ref()
        .unwrap();
    let ready = watcher.ready.clone();
    let published = watcher.published.clone();
    let superseded = watcher.superseded.clone();
    let wait = |predicate: &dyn Fn() -> bool| -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(30);
        while !predicate() {
            if Instant::now() >= deadline {
                return Err("supersession proof timed out".into());
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    };
    wait(&|| ready.load(Ordering::Acquire))?;
    // The fixture build script blocks a real Cargo build until the next save.
    let gate = directory.join("hold-build");
    let marker = directory.join("build-started");
    std::fs::write(&gate, b"hold").map_err(|e| e.to_string())?;
    std::fs::write(&path, original.replace("{ 1 }", "{ 2 }")).map_err(|e| e.to_string())?;
    wait(&|| marker.exists())?;
    assert_eq!(published.load(Ordering::Acquire), 0);
    // Existing actors remain usable while a compiler process is blocked.
    for id in &ids {
        let button = source.actor.widget(*id, key("counter.increment")).unwrap();
        source.actor.dispatch_widget(button)?;
        assert_eq!(
            source.actor.runtime().actor(*id).unwrap().integer("count"),
            1
        );
    }
    std::fs::write(&path, original.replace("{ 1 }", "{ 5 }")).map_err(|e| e.to_string())?;
    // The blocked build must be killed before releasing its gate. Merely discarding
    // a completed result cannot pass this assertion.
    wait(&|| superseded.load(Ordering::Acquire) >= 1)?;
    assert!(gate.exists());
    assert_eq!(published.load(Ordering::Acquire), 0);
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(ids[0])
            .unwrap()
            .slot_address("count"),
        slots[0]
    );
    std::fs::remove_file(&gate).map_err(|e| e.to_string())?;
    wait(&|| published.load(Ordering::Acquire) >= 1)?;
    // Do not consume results: later saves must compile and replace the old slot.
    for (generation, delta) in [(2, 6), (3, 7), (4, 8)] {
        let mut code = original.replace("{ 1 }", &format!("{{ {delta} }}"));
        if delta == 7 {
            code.push_str("\ncompile_error!(\"superseded failure\");\n");
        }
        std::fs::write(&path, code).map_err(|e| e.to_string())?;
        wait(&|| published.load(Ordering::Acquire) >= generation)?;
        let pending = std::fs::read_dir(".runtime")
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("pending-"))
            .count();
        assert_eq!(pending, usize::from(delta != 7));
    }
    assert!(source.poll_async());
    for (index, id) in ids.iter().enumerate() {
        assert_eq!(
            source
                .actor
                .runtime()
                .actor(*id)
                .unwrap()
                .slot_address("count"),
            slots[index]
        );
        let button = source.actor.widget(*id, key("counter.increment")).unwrap();
        source.actor.dispatch_widget(button)?;
        assert_eq!(
            source.actor.runtime().actor(*id).unwrap().integer("count"),
            9
        );
    }
    drop(source);
    println!(
        "PASS save during gated Cargo build terminates obsolete owned process group before gate release; unconsumed successful/error results are superseded without blocking, retain one queued DLL and publish only latest behavior to both actors"
    );
    Ok(())
}

pub fn child_routing_proof(
    base: &Path,
    replacement: &Path,
    counter: &Path,
    readonly_lookup: &Path,
) -> Result<(), String> {
    use crate::Runtime;
    let mut runtime = Runtime::new(".runtime");
    runtime.register_implementation("counter", counter)?;
    let a = ActorId(10);
    let b = ActorId(11);
    runtime.spawn(a, base)?;
    runtime.spawn(b, base)?;
    let slot = runtime
        .actor(a)
        .unwrap()
        .view
        .order
        .iter()
        .copied()
        .find(|key| runtime.actor(a).unwrap().view.bindings[key].node.kind == CHILD)
        .unwrap();
    let a_child = runtime.child(a, slot).unwrap();
    let b_child = runtime.child(b, slot).unwrap();
    assert_ne!(a_child, b_child);
    let addresses = [
        runtime.actor(a_child).unwrap().slot_address("count"),
        runtime.actor(b_child).unwrap().slot_address("count"),
    ];
    let send = Message {
        action: key("increment_child"),
        value: 0,
    };
    runtime.actor_mut(a).unwrap().dispatch(send)?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(a_child).unwrap().integer("count"), 1);
    assert_eq!(runtime.actor(b_child).unwrap().integer("count"), 0);
    assert!(
        runtime
            .actor_mut(a)
            .unwrap()
            .dispatch(Message { value: 1, ..send })
            .is_err()
    );
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(a_child).unwrap().integer("count"), 1);
    runtime.actor_mut(b).unwrap().dispatch(send)?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(b_child).unwrap().integer("count"), 1);
    runtime.reload(a, replacement)?;
    assert_eq!(runtime.child(a, slot), Some(a_child));
    runtime.actor_mut(a).unwrap().dispatch(send)?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(a_child).unwrap().integer("count"), 2);
    assert_eq!(
        runtime.actor(a_child).unwrap().slot_address("count"),
        addresses[0]
    );
    assert_eq!(
        runtime.actor(b_child).unwrap().slot_address("count"),
        addresses[1]
    );
    runtime.actor_mut(a).unwrap().dispatch(Message {
        action: key("send_old_child"),
        value: a_child.0 as i64,
    })?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(a_child).unwrap().integer("count"), 3);
    let version = runtime.actor(a).unwrap().implementation_version();
    assert!(
        runtime
            .reload(a, readonly_lookup)
            .unwrap_err()
            .contains("view failed")
    );
    assert_eq!(runtime.actor(a).unwrap().implementation_version(), version);
    assert_eq!(runtime.child(a, slot), Some(a_child));
    runtime.actor_mut(a).unwrap().dispatch(Message {
        action: key("hide"),
        value: 0,
    })?;
    assert!(runtime.tick().is_empty());
    runtime
        .actor_mut(a)
        .unwrap()
        .dispatch(Message { value: 2, ..send })?;
    assert!(runtime.actor_mut(a).unwrap().dispatch(send).is_err());
    runtime.actor_mut(a).unwrap().dispatch(Message {
        action: key("show"),
        value: 0,
    })?;
    assert!(runtime.tick().is_empty());
    let fresh = runtime.child(a, slot).unwrap();
    assert_ne!(fresh, a_child);
    runtime.actor_mut(a).unwrap().dispatch(send)?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(fresh).unwrap().integer("count"), 1);
    assert!(
        runtime
            .actor_mut(a)
            .unwrap()
            .dispatch(Message {
                action: key("send_old_child"),
                value: a_child.0 as i64
            })
            .is_err()
    );
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(fresh).unwrap().integer("count"), 1);
    assert_eq!(runtime.actor(b_child).unwrap().integer("count"), 1);
    drop(runtime);
    println!(
        "PASS SDK child lookup is parent-scoped, routes through current code, survives parent reload/reorder, rolls back rejected sends, returns None after removal and resolves fresh identities after recreation while rejecting stale addresses"
    );
    Ok(())
}

#[cfg(test)]
mod source_watch_tests {
    use super::{source_change_summary, source_snapshot};
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    #[cfg(unix)]
    use std::path::PathBuf;

    #[test]
    fn source_edits_and_removal_change_snapshot() {
        let root = std::env::temp_dir().join(format!("blockui-watch-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("lib.rs");
        fs::write(&source, "one").unwrap();
        let before = source_snapshot(std::slice::from_ref(&root));
        fs::write(&source, "different length").unwrap();
        let changed = source_snapshot(std::slice::from_ref(&root));
        assert_ne!(before, changed);
        assert_eq!(
            source_change_summary(&before, &changed, &root),
            "1 changed file(s): lib.rs"
        );
        fs::remove_file(&source).unwrap();
        let removed = source_snapshot(std::slice::from_ref(&root));
        assert_ne!(changed, removed);
        assert!(!removed.contains_key(&source));
        assert_eq!(
            source_change_summary(&changed, &removed, &root),
            "1 changed file(s): lib.rs"
        );
        fs::write(root.join("new.rs"), "new").unwrap();
        let added = source_snapshot(std::slice::from_ref(&root));
        assert_eq!(
            source_change_summary(&before, &added, &root),
            "2 changed file(s): lib.rs"
        );
        fs::remove_file(root.join("new.rs")).unwrap();
        fs::remove_dir(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    #[ignore = "explicit watcher filesystem benchmark"]
    fn snapshot_alias_cost() {
        let root =
            std::env::temp_dir().join(format!("blockui-snapshot-bench-{}", std::process::id()));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let data = root.join("data");
        fs::create_dir_all(&data).unwrap();
        for i in 0..200 {
            fs::write(data.join(format!("input{i}.rs")), "source").unwrap();
        }
        for i in 0..16 {
            symlink(&data, root.join(format!("alias{i}"))).unwrap();
        }
        fn measure(roots: &[PathBuf]) -> serde_json::Value {
            let expected = source_snapshot(roots);
            for _ in 0..25 {
                assert_eq!(source_snapshot(roots), expected);
            }
            let mut samples = Vec::new();
            for _ in 0..5 {
                let start = std::time::Instant::now();
                for _ in 0..50 {
                    assert_eq!(source_snapshot(roots), expected);
                }
                samples.push(start.elapsed().as_secs_f64() * 1_000_000.0 / 50.0);
            }
            samples.sort_by(f64::total_cmp);
            serde_json::json!({"entries":expected.len(),"samples_microseconds":samples,"median_microseconds":samples[2]})
        }
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut config = super::ReloadConfig::from_environment(&repo);
        config
            .refresh(&std::sync::atomic::AtomicBool::new(false))
            .unwrap();
        let plain = measure(std::slice::from_ref(&data));
        let native_counter = measure(&config.sources);
        let aliased = measure(std::slice::from_ref(&root));
        let report = serde_json::json!({"fixture_version":3,"warmup_snapshots":25,"snapshots_per_batch":50,"files":200,"directory_aliases":16,
            "aliased":aliased,
            "plain":plain,
            "native_counter":native_counter,
            "native_package":config.package,
            "scope":"Warm filesystem snapshot only; dependency discovery, Cargo builds, reload and rendering excluded"});
        let reports = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../reports");
        fs::create_dir_all(&reports).unwrap();
        fs::write(
            reports.join("watch-snapshot-benchmark.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        eprintln!("{report}");
    }
    #[cfg(unix)]
    #[test]
    fn linked_roots_cycles_dangling_targets_and_excluded_build_trees() {
        let root = std::env::temp_dir().join(format!("identity-links-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let app = root.join("app");
        let external = root.join("external");
        fs::create_dir(&app).unwrap();
        fs::create_dir(&external).unwrap();
        let source = external.join("lib.rs");
        fs::write(&source, "one").unwrap();
        symlink(&external, app.join("src")).unwrap();
        symlink(&app, external.join("cycle")).unwrap();
        let linked_root = root.join("linked-app");
        symlink(&app, &linked_root).unwrap();
        let missing = external.join("missing.rs");
        symlink(&missing, app.join("pending.rs")).unwrap();
        let target = root.join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("large.rs"), "build cache").unwrap();
        symlink(&target, app.join("build-link")).unwrap();
        let before = source_snapshot(std::slice::from_ref(&linked_root));
        assert!(before.contains_key(&source));
        assert_eq!(
            source_snapshot(&[
                linked_root.clone(),
                app.clone(),
                external.clone(),
                app.clone()
            ]),
            before,
            "overlapping roots retain all canonical files and link metadata"
        );
        assert!(before.contains_key(&app.join("pending.rs")));
        assert!(!before.contains_key(&target.join("large.rs")));
        fs::write(&missing, "new source").unwrap();
        let restored = source_snapshot(std::slice::from_ref(&linked_root));
        assert!(restored.contains_key(&missing));
        assert_ne!(before, restored);
        let other = root.join("other");
        fs::create_dir(&other).unwrap();
        fs::write(other.join("lib.rs"), "one").unwrap();
        fs::remove_file(app.join("src")).unwrap();
        symlink(&other, app.join("src")).unwrap();
        let retargeted = source_snapshot(std::slice::from_ref(&linked_root));
        assert!(!retargeted.contains_key(&source));
        assert!(retargeted.contains_key(&other.join("lib.rs")));
        assert_ne!(restored, retargeted);
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod request_deadline_tests {
    use super::*;
    use crate::{ActorId, Runtime, native_model::NativeFamily, request::RequestError};
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };
    #[test]
    #[ignore = "requires Counter dylibs; scripts/verify_native_requests.py"]
    fn host_notification_panics_do_not_unwind_committed_native_family_operations() {
        use std::sync::atomic::AtomicUsize;
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let directory =
            std::env::temp_dir().join(format!("native-notification-fault-{}", std::process::id()));
        let root = ActorId(1);
        let peer = ActorId(2);
        let mut runtime = Runtime::new(&directory);
        runtime.spawn(root, Path::new(&base)).unwrap();
        runtime.spawn(peer, Path::new(&base)).unwrap();
        let mut family = NativeFamily::new(runtime, root).unwrap();
        let armed = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicUsize::new(0));
        let flag = armed.clone();
        let count = calls.clone();
        let callback: shell::WakeCallback = Arc::new(move || {
            if flag.load(Ordering::Acquire) {
                count.fetch_add(1, Ordering::Relaxed);
                panic!("injected host notification failure");
            }
        });
        family.set_wake(callback.clone());
        armed.store(true, Ordering::Release);
        let storage = family.root_actor().state_address();
        family
            .tell(
                root,
                Message {
                    action: key("increment"),
                    value: 0,
                },
            )
            .unwrap();
        let mut reply = family
            .ask(
                peer,
                Message {
                    action: key("request_count"),
                    value: 0,
                },
                Duration::from_secs(30),
            )
            .unwrap();
        family.tick().unwrap();
        assert_eq!(family.root_actor().integer("count"), 1);
        assert_eq!(
            Pin::new(&mut reply).poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Ok(1))
        );
        assert_eq!(calls.load(Ordering::Relaxed), 2);
        armed.store(false, Ordering::Release);
        family.set_wake(callback.clone());
        armed.store(true, Ordering::Release);
        family.reload_actor(root, Path::new(&next)).unwrap();
        assert_eq!(family.root_actor().state_address(), storage);
        assert_eq!(calls.load(Ordering::Relaxed), 3);
        armed.store(false, Ordering::Release);
        family.set_wake(callback);
        let mut cancelled = family
            .ask(
                root,
                Message {
                    action: key("request_count"),
                    value: 0,
                },
                Duration::from_secs(30),
            )
            .unwrap();
        armed.store(true, Ordering::Release);
        assert!(family.cancel_request(cancelled.id()));
        family.tick().unwrap();
        assert_eq!(
            Pin::new(&mut cancelled).poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Err(RequestError::Cancelled))
        );
        assert_eq!(family.root_actor().integer("count"), 1);
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        assert_eq!(family.async_deadline(), None);
        let recovered = Arc::new(AtomicUsize::new(0));
        let healthy = recovered.clone();
        family.set_wake(Arc::new(move || {
            healthy.fetch_add(1, Ordering::Relaxed);
        }));
        family
            .tell(
                root,
                Message {
                    action: key("increment"),
                    value: 0,
                },
            )
            .unwrap();
        family.tick().unwrap();
        assert_eq!(family.root_actor().integer("count"), 11);
        assert!(recovered.load(Ordering::Relaxed) >= 3);
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        drop(family);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
        std::fs::remove_dir(directory).unwrap();
    }
    #[test]
    #[ignore = "requires Counter dylibs; scripts/verify_native_requests.py"]
    fn sdk_request_native_binding_patch_retains_editor_through_both_actor_reloads() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let directory =
            std::env::temp_dir().join(format!("native-sdk-request-{}", std::process::id()));
        let root = ActorId(1);
        let peer = ActorId(2);
        let mut runtime = Runtime::new(&directory);
        runtime.spawn(root, Path::new(&base)).unwrap();
        runtime.spawn(peer, Path::new(&base)).unwrap();
        let model = NativeFamily::new(runtime, root).unwrap();
        let mut source = NativeSource::without_watch(model).unwrap();
        let (wake_tx, wake_rx) = mpsc::channel();
        source.set_wake_callback(Arc::new(move || {
            let _ = wake_tx.send(());
        }));
        while wake_rx.try_recv().is_ok() {}
        let input = source.actor.widget(root, key("counter.notes")).unwrap();
        let caption = source.actor.widget(root, key("counter.task")).unwrap();
        source
            .actor
            .set_widget_text(input, "a😀bc unsaved draft")
            .unwrap();
        source.sync().unwrap();
        source.set_focus(Some(input));
        source
            .editors
            .get_mut(&input)
            .unwrap()
            .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
            .unwrap();
        let mut text = TextContext::new();
        source.build_frame(&mut text, 640, 620);
        let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
        let selection = source.editors[&input].text_selection();
        let input_node = source.ids[&input];
        let caption_node = source.ids[&caption];
        let slot = source.actor.root_actor().slot_address("notes");
        while wake_rx.try_recv().is_ok() {}
        source
            .actor
            .root_actor_mut()
            .dispatch(Message {
                action: key("ask"),
                value: 2,
            })
            .unwrap();
        wake_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        source.poll_async(); // The request enters the peer mailbox.
        source.actor.reload_actor(root, Path::new(&next)).unwrap();
        source.actor.reload_actor(peer, Path::new(&next)).unwrap();
        for _ in 0..3 {
            source.poll_async();
        }
        source.build_frame(&mut text, 640, 620);
        assert_eq!(source.actor.root_actor().integer("task_result"), 1010);
        assert_eq!(
            source.actor.view().bindings[&caption].text,
            "Background result: 1010"
        );
        assert_eq!(
            source
                .tree
                .get_node_context(caption_node)
                .unwrap()
                .text
                .as_deref(),
            Some("Background result: 1010")
        );
        assert_eq!(source.ids[&caption], caption_node);
        assert_eq!(source.ids[&input], input_node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            editor
        );
        assert_eq!(source.editors[&input].text_selection(), selection);
        assert_eq!(source.focus, Some(input));
        assert_eq!(
            source.editors[&input].current_value(),
            Some("a😀bc unsaved draft")
        );
        assert_eq!(source.actor.root_actor().slot_address("notes"), slot);
        // Public host admission is asynchronous and resolves the implementation at delivery.
        assert!(
            source
                .actor
                .tell(
                    ActorId(99),
                    Message {
                        action: key("increment"),
                        value: 0
                    }
                )
                .is_err()
        );
        assert!(
            source
                .actor
                .tell(
                    root,
                    Message {
                        action: 0,
                        value: 0
                    }
                )
                .is_err()
        );
        source
            .actor
            .tell(
                root,
                Message {
                    action: key("increment"),
                    value: 0,
                },
            )
            .unwrap();
        assert_eq!(source.actor.root_actor().integer("count"), 0);
        source.actor.reload_actor(root, Path::new(&base)).unwrap();
        source.poll_async();
        source.build_frame(&mut text, 640, 620);
        assert_eq!(source.actor.root_actor().integer("count"), 1);
        assert_eq!(source.ids[&input], input_node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            editor
        );
        assert_eq!(source.editors[&input].text_selection(), selection);
        assert_eq!(source.actor.root_actor().slot_address("notes"), slot);
        assert_eq!(source.focus, Some(input));
        let mut abandoned = source
            .actor
            .ask(
                peer,
                Message {
                    action: key("request_count"),
                    value: 0,
                },
                Duration::from_secs(30),
            )
            .unwrap();
        assert!(source.actor.cancel_request(abandoned.id()));
        assert!(!source.actor.cancel_request(abandoned.id()));
        assert_eq!(
            Pin::new(&mut abandoned).poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Err(RequestError::Cancelled))
        );
        source.poll_async();
        assert_eq!(
            source.actor.runtime().actor(peer).unwrap().integer("count"),
            10,
            "cancelled unprocessed query must not execute the peer handler"
        );
        assert_eq!(source.actor.async_deadline(), None);
        assert!(source.diagnostic.is_empty());
        drop(source);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
        std::fs::remove_dir(directory).unwrap();
    }
    #[test]
    #[ignore = "requires a built Counter library; scripts/verify_native_requests.py"]
    fn idle_native_source_exposes_request_deadline_and_drains_timeout() {
        let library = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let directory =
            std::env::temp_dir().join(format!("native-request-deadline-{}", std::process::id()));
        let id = ActorId(1);
        let mut runtime = Runtime::new(&directory);
        runtime.spawn(id, Path::new(&library)).unwrap();
        let model = NativeFamily::new(runtime, id).unwrap();
        let mut source = NativeSource::without_watch(model).unwrap();
        // Ordinary increment intentionally has no reply, so it waits for expiry.
        let mut reply = source
            .actor
            .ask(
                id,
                Message {
                    action: key("increment"),
                    value: 0,
                },
                Duration::from_millis(30),
            )
            .unwrap();
        let deadline = source.actor.runtime().next_request_deadline().unwrap();
        assert!(!source.has_anim());
        assert_eq!(source.animation_deadline(), Some(deadline));
        source.poll_async();
        assert_eq!(
            Pin::new(&mut reply).poll(&mut Context::from_waker(Waker::noop())),
            Poll::Pending
        );
        std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
        // Deadline scheduling generates RedrawRequested rather than a proxy wake.
        source.build_frame(&mut TextContext::new(), 640, 480);
        assert_eq!(
            Pin::new(&mut reply).poll(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Err(RequestError::Timeout))
        );
        assert_eq!(source.animation_deadline(), None);
        assert!(!source.has_anim());
        drop(source);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

/// Real dylibs prove semantic automatic identity without relying on source positions.
pub fn auto_key_proof(
    base: &Path,
    replacement: &Path,
    ambiguous: &Path,
    static_ambiguous: &Path,
    disabled: &Path,
    row_root: &Path,
    string_ambiguous: &Path,
) -> Result<(), String> {
    let mut actor = Actor::new(701, Path::new(".runtime"))?;
    actor.reload(base)?;
    let root = key("identity.auto.actor.root");
    assert_eq!(actor.view.order[0], root);
    let input = actor
        .view
        .bindings
        .iter()
        .find(|(_, b)| b.node.kind == blockui_abi::ENTRY && b.node.parent != root)
        .unwrap()
        .0
        .to_owned();
    let button = actor
        .view
        .bindings
        .iter()
        .find(|(_, b)| b.node.kind == blockui_abi::BUTTON && b.text.starts_with("Increment"))
        .unwrap()
        .0
        .to_owned();
    let caption = *actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.text.starts_with("Count: 0"))
        .unwrap()
        .0;
    let scroll = *actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| {
            binding.node.kind == blockui_abi::SCROLL && binding.node.key != key("scroll.inner")
        })
        .unwrap()
        .0;
    assert_ne!(scroll, key("counter.scroll"));
    assert_ne!(caption, key("counter.count"));
    assert_ne!(input, key("counter.notes"));
    assert_ne!(button, key("counter.increment"));
    let mut source = NativeSource::without_watch(actor)?;
    assert!(source.semantic_snapshot().is_none());
    source.set_semantics_enabled(true);
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source
        .actor
        .set_text(key("notes"), "Automatic identity keeps a😀bc")?;
    source.sync()?;
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .map_err(|e| e.to_string())?;
    source.actor.view.renderer.get_mut(&scroll).unwrap().scroll = 137.0;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 720, 620);
    let shared_inputs: Vec<_> = source
        .actor
        .view
        .order
        .iter()
        .copied()
        .filter(|key| {
            *key != input && source.actor.view.bindings[key].node.kind == blockui_abi::ENTRY
        })
        .collect();
    assert_eq!(shared_inputs.len(), 2);
    assert_ne!(shared_inputs[0], shared_inputs[1]);
    let mut shared_editors = Vec::new();
    for (index, key) in shared_inputs.iter().enumerate() {
        assert_eq!(
            source.actor.view.bindings[key].node.slot,
            source.actor.view.bindings[&input].node.slot
        );
        let widget = source.editors.get_mut(key).unwrap();
        widget
            .config_changed(if index == 0 {
                r#"{"selection":{"anchor":5,"head":7}}"#
            } else {
                r#"{"selection":{"anchor":9,"head":12}}"#
            })
            .map_err(|e| e.to_string())?;
        assert_eq!(
            widget.current_value(),
            Some("Automatic identity keeps a😀bc")
        );
        source.build_frame(&mut tcx, 720, 620);
        let widget = &source.editors[key];
        shared_editors.push((
            *key,
            (&**widget as *const TextInput) as usize,
            source.ids[key],
            widget.text_selection(),
        ));
    }
    assert_ne!(shared_editors[0].1, shared_editors[1].1);
    assert_ne!(shared_editors[0].3, shared_editors[1].3);
    assert_eq!(shared_editors[0].3.as_ref().unwrap().anchor, 5);
    assert_eq!(shared_editors[0].3.as_ref().unwrap().head, 7);
    assert_eq!(shared_editors[1].3.as_ref().unwrap().anchor, 9);
    assert_eq!(shared_editors[1].3.as_ref().unwrap().head, 12);
    let inner_scroll = key("scroll.inner");
    let inner_scroll_node = source.ids[&inner_scroll];
    assert_eq!(
        source.actor.view.bindings[&inner_scroll].node.parent,
        scroll
    );
    let scrolled_action = key("scroll.action");
    let scrolled_node = source.ids[&scrolled_action];
    assert_eq!(
        source.actor.view.bindings[&scrolled_action].node.parent,
        inner_scroll
    );
    assert!(
        source
            .placed
            .iter()
            .all(|node| node.node_id != scrolled_node)
    );
    let offscreen = source.widget_layout_rect(scrolled_action).unwrap();
    let offscreen_point = shell::Point {
        x: f64::from(offscreen[0] + 10.0),
        y: f64::from(offscreen[1] + 10.0),
    };
    assert_ne!(source.hit(offscreen_point), Some(scrolled_action));
    let semantic_before = source.semantic_snapshot().unwrap();
    source.build_frame(&mut tcx, 720, 620);
    assert!(Arc::ptr_eq(
        &semantic_before,
        &source.semantic_snapshot().unwrap()
    ));
    assert_eq!(semantic_before.root_children, vec![root]);
    assert!(
        semantic_before
            .nodes
            .iter()
            .all(|node| node.bounds.iter().all(|value| value.is_finite()))
    );
    assert_eq!(semantic_before.focus, Some(input));
    assert_eq!(
        semantic_before
            .nodes
            .iter()
            .find(|node| node.id == input)
            .unwrap()
            .value
            .as_deref(),
        Some("Automatic identity keeps a😀bc")
    );
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let selection = source.editors[&input].text_selection();
    let mut ime = source.ime_state();
    let slot = source.actor.slot_address("notes");
    let scroll_node = source.ids[&scroll];
    let root_node = source.ids[&root];
    let static_key = source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, b)| b.text == "Automatic static caption")
        .unwrap()
        .0
        .to_owned();
    let local_keys: Vec<_> = [
        "Local title",
        "Local subtitle",
        "Named primary",
        "Named secondary",
        "Keyed row 1",
        "Keyed row 2",
        "Clone title",
        "Clone subtitle",
    ]
    .iter()
    .map(|text| {
        let key = *source
            .actor
            .view
            .bindings
            .iter()
            .find(|(_, b)| b.text == *text)
            .unwrap()
            .0;
        (key, source.ids[&key])
    })
    .collect();
    assert_ne!(local_keys[0].0, local_keys[1].0);
    assert_ne!(local_keys[2].0, local_keys[3].0);
    assert_ne!(local_keys[4].0, local_keys[5].0);
    assert!(
        !source
            .actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "Keyed row 3")
    );
    assert!(
        source
            .actor
            .view
            .order
            .iter()
            .position(|key| *key == local_keys[5].0)
            .unwrap()
            < source
                .actor
                .view
                .order
                .iter()
                .position(|key| *key == local_keys[4].0)
                .unwrap()
    );
    assert_eq!(
        source.actor.view.bindings[&local_keys[2].0].node.action,
        source.actor.view.bindings[&local_keys[3].0].node.action
    );
    let static_node = source.ids[&static_key];
    let container = source.actor.view.bindings[&input].node.parent;
    let container_node = source.ids[&container];
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    for (key, allocation, node, selection) in &shared_editors {
        assert_eq!(
            (&**source.editors.get(key).unwrap() as *const TextInput) as usize,
            *allocation
        );
        assert_eq!(source.ids[key], *node);
        assert_eq!(source.editors[key].text_selection(), *selection);
        assert_eq!(
            source.editors[key].current_value(),
            Some("Automatic identity keeps a😀bc")
        );
        assert_eq!(
            source.actor.view.bindings[key].node.slot,
            source.actor.view.bindings[&input].node.slot
        );
    }
    assert!(
        source
            .actor
            .view
            .order
            .iter()
            .position(|key| *key == shared_inputs[1])
            .unwrap()
            < source
                .actor
                .view
                .order
                .iter()
                .position(|key| *key == shared_inputs[0])
                .unwrap()
    );
    assert_eq!(source.actor.view.order[0], root);
    assert_eq!(source.ids[&root], root_node);
    assert_eq!(source.ids[&static_key], static_node);
    assert_eq!(source.ids[&scrolled_action], scrolled_node);
    assert_eq!(source.ids[&inner_scroll], inner_scroll_node);
    let semantic_after = source.semantic_snapshot().unwrap();
    assert!(semantic_after.revision > semantic_before.revision);
    assert_eq!(semantic_after.focus, semantic_before.focus);
    assert_eq!(
        semantic_after
            .nodes
            .iter()
            .find(|node| node.id == input)
            .unwrap()
            .role,
        shell::SemanticRole::TextInput
    );
    assert_eq!(
        semantic_after
            .nodes
            .iter()
            .find(|node| node.id == button)
            .unwrap()
            .role,
        shell::SemanticRole::Button
    );
    for (key, node) in &local_keys {
        assert_eq!(source.ids[key], *node);
    }
    assert_eq!(
        source.actor.view.bindings[&local_keys[0].0].text,
        "Edited local title"
    );
    assert_ne!(local_keys[6].0, local_keys[7].0);
    assert_eq!(
        source.actor.view.bindings[&local_keys[6].0].text,
        "Edited clone title"
    );
    let clone_order = &source.actor.view.order;
    assert!(
        clone_order
            .iter()
            .position(|k| *k == local_keys[7].0)
            .unwrap()
            < clone_order
                .iter()
                .position(|k| *k == local_keys[6].0)
                .unwrap()
    );
    println!(
        "PASS cloned custom components retain native Taffy nodes through real DLL content edits/source movement/sibling insertion/reorder"
    );
    let order = &source.actor.view.order;
    assert!(
        order.iter().position(|k| *k == local_keys[1].0).unwrap()
            < order.iter().position(|k| *k == local_keys[0].0).unwrap()
    );
    assert!(
        order.iter().position(|k| *k == local_keys[3].0).unwrap()
            < order.iter().position(|k| *k == local_keys[2].0).unwrap()
    );
    assert!(
        source
            .actor
            .view
            .order
            .iter()
            .position(|key| *key == local_keys[4].0)
            .unwrap()
            < source
                .actor
                .view
                .order
                .iter()
                .position(|key| *key == local_keys[5].0)
                .unwrap()
    );
    let inserted_key = *source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.text == "Keyed row 3")
        .unwrap()
        .0;
    let inserted_node = source.ids[&inserted_key];
    assert!(
        local_keys
            .iter()
            .all(|(key, node)| *key != inserted_key && *node != inserted_node)
    );
    assert_eq!(
        source.actor.view.bindings[&static_key].text,
        "Reloaded static caption"
    );
    assert_eq!(source.actor.view.bindings[&input].node.parent, container);
    assert_eq!(source.ids[&container], container_node);
    assert_eq!(source.ids[&scroll], scroll_node);
    assert_eq!(source.actor.view.renderer[&scroll].scroll, 137.0);
    let placed_scroll = source
        .placed
        .iter()
        .find(|node| node.node_id == scroll_node)
        .unwrap();
    assert_eq!(placed_scroll.scroll.offset[1], 137.0);
    assert!(
        source.actor.view.bindings[&scroll]
            .text
            .contains("Reloaded row")
    );
    assert_eq!(source.actor.slot_address("notes"), slot);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    let next_ime = source.ime_state();
    // Inserted siblings and padding move the candidate rectangle, while keeping
    // the same IME client and all editor selection/document state.
    assert_ne!(
        next_ime.as_ref().unwrap().cursor_area,
        ime.as_ref().unwrap().cursor_area
    );
    ime.as_mut().unwrap().cursor_area = next_ime.as_ref().unwrap().cursor_area;
    assert_eq!(next_ime, ime);
    assert_eq!(
        source.editors[&input].current_value(),
        Some("Automatic identity keeps a😀bc")
    );
    assert!(source.actor.view.bindings.contains_key(&button));
    assert!(
        source.actor.view.bindings[&caption]
            .text
            .contains("new code")
    );
    let version = source.actor.implementation_version();
    assert!(
        source
            .replace(ambiguous)
            .unwrap_err()
            .contains("duplicate view identity")
    );
    assert!(
        source
            .replace(static_ambiguous)
            .unwrap_err()
            .contains("duplicate view identity")
    );
    assert!(
        source
            .replace(string_ambiguous)
            .unwrap_err()
            .contains("duplicate view identity")
    );
    assert_eq!(source.actor.implementation_version(), version);
    assert_eq!(source.ids[&inserted_key], inserted_node);
    assert_eq!(
        source.actor.view.bindings[&inserted_key].text,
        "Keyed row 3"
    );
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    let node = source
        .placed
        .iter()
        .find(|n| n.node_id == source.ids[&button])
        .unwrap();
    let position = shell::Point {
        x: (node.rect[0] + 10.0) as f64,
        y: (node.rect[1] + 10.0) as f64,
    };
    for button in [
        shell::PointerButton::Secondary,
        shell::PointerButton::Auxiliary,
    ] {
        let focus = source.focus;
        for phase in [PointerPhase::Down, PointerPhase::Up] {
            source.handle_event(UiEvent::Pointer(shell::PointerEvent {
                phase,
                position,
                button: Some(button),
                buttons: 0,
                modifiers: Default::default(),
                properties: Default::default(),
            }));
        }
        assert_eq!(
            source.actor.integer("count"),
            0,
            "non-primary click must not activate button"
        );
        assert_eq!(source.focus, focus);
    }
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button: Some(shell::PointerButton::Primary),
            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    }
    assert_eq!(source.actor.integer("count"), 10);
    assert!(
        source.actor.view.bindings[&caption]
            .text
            .starts_with("Count: 10")
    );
    let retained_button = source.ids[&button];
    source.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase: PointerPhase::Down,
        position,
        button: Some(shell::PointerButton::Primary),
        buttons: 1,
        modifiers: Default::default(),
        properties: Default::default(),
    }));
    assert_eq!(source.pressed, Some(button));
    source.replace(disabled)?;
    let disabled_count = source.actor.integer("count");
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    assert!(!source.handle_semantic_action(shell::SemanticAction::Focus { target: button }));
    assert_eq!(source.actor.integer("count"), disabled_count);
    assert!(!source.ids.contains_key(&inserted_key));
    assert!(!source.actor.view.bindings.contains_key(&inserted_key));
    for (key, node) in &local_keys[4..] {
        assert_eq!(source.ids[key], *node);
    }
    assert_eq!(source.pressed, None);
    assert_ne!(source.focus, Some(button));
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(
        source.actor.view.renderer[&scroll].scroll,
        0.0,
        "lines={} range={:?}",
        source.actor.view.bindings[&scroll].text.lines().count(),
        source
            .placed
            .iter()
            .find(|node| node.node_id == scroll_node)
            .unwrap()
            .scroll
            .range
    );
    assert_eq!(
        source
            .placed
            .iter()
            .find(|node| node.node_id == scroll_node)
            .unwrap()
            .scroll
            .range[1],
        0.0
    );
    assert_eq!(source.ids[&button], retained_button);
    assert_eq!(source.actor.view.renderer[&scroll].scroll, 0.0);
    assert_eq!(source.actor.view.bindings[&button].node.disabled, 1);
    assert_eq!(
        source
            .tree
            .get_node_context(retained_button)
            .unwrap()
            .background,
        Some(Color::from_rgb8(240, 243, 247))
    );
    source.actor.dispatch_widget(button)?;
    assert_eq!(source.actor.integer("count"), 10);
    source.set_focus(None);
    source.focus_next(false);
    assert_eq!(source.focus, Some(input));
    source.set_focus(Some(button));
    let key_event = |name: &str, modifiers| {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers,
            repeat: false,
            synthetic: false,
        })
    };
    source.handle_event(key_event("Enter", shell::Modifiers::empty()));
    source = source.primary_shortcut("s", button)?;
    source.handle_event(key_event(
        "s",
        if cfg!(target_os = "macos") {
            shell::Modifiers::META
        } else {
            shell::Modifiers::CONTROL
        },
    ));
    assert_eq!(source.actor.integer("count"), 10);
    source.build_frame(&mut tcx, 720, 620);
    let placed = source
        .placed
        .iter()
        .find(|node| node.node_id == retained_button)
        .unwrap();
    let position = shell::Point {
        x: (placed.rect[0] + 10.0) as f64,
        y: (placed.rect[1] + 10.0) as f64,
    };
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button: Some(shell::PointerButton::Primary),
            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    }
    assert_eq!(source.actor.integer("count"), 10);
    assert_ne!(source.focus, Some(button));
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.ids[&button], retained_button);
    assert_eq!(source.actor.view.renderer[&scroll].scroll, 0.0);
    source.actor.dispatch_widget(button)?;
    assert_eq!(source.actor.integer("count"), 20);
    println!(
        "PASS disabled button survives replacement with retained layout identity, skips Tab and rejects direct widget/pointer/Enter/primary activation; reenabling executes new action"
    );
    let editor_before = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let button_before = source.ids[&button];
    source.replace(row_root)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.actor.view.bindings[&root].node.kind, ROW);
    assert_eq!(source.ids[&button], button_before);
    assert_eq!(source.tree.style(button_before).unwrap().flex_grow, 1.0);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor_before
    );
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.actor.view.bindings[&root].node.kind, COLUMN);
    assert_eq!(source.ids[&button], button_before);
    assert_eq!(source.tree.style(button_before).unwrap().flex_grow, 0.0);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor_before
    );
    println!(
        "PASS real Row/Column replacement rebuilds parent-dependent child styles while retaining child layout nodes and editor allocation"
    );
    assert!(
        source.handle_semantic_action(shell::SemanticAction::ScrollIntoView {
            target: scrolled_action
        })
    );
    source.build_frame(&mut tcx, 720, 620);
    let revealed = source
        .placed
        .iter()
        .find(|node| node.node_id == scrolled_node)
        .unwrap();
    let revealed_point = shell::Point {
        x: f64::from(revealed.rect[0] + 10.0),
        y: f64::from(revealed.rect[1] + 10.0),
    };
    assert_eq!(source.hit(revealed_point), Some(scrolled_action));
    let revealed_offset = source.actor.view.renderer[&scroll].scroll;
    assert!(revealed_offset > 1500.0);
    let inner_revealed_offset = source.actor.view.renderer[&inner_scroll].scroll;
    assert!(inner_revealed_offset > 1500.0);
    assert!(
        source.handle_semantic_action(shell::SemanticAction::ScrollIntoView {
            target: scrolled_action
        })
    );
    assert_eq!(source.actor.view.renderer[&scroll].scroll, revealed_offset);
    assert_eq!(
        source.actor.view.renderer[&inner_scroll].scroll,
        inner_revealed_offset
    );
    println!(
        "PASS nested ScrollIntoView reveals a retained button through both ancestor clips and repeated requests preserve offsets"
    );
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::ScrollIntoView { target: u64::MAX })
    );
    let scroll_layout = source
        .placed
        .iter()
        .find(|node| node.node_id == scroll_node)
        .unwrap();
    let range = f64::from(scroll_layout.scroll.range[1]);
    assert!(range > 1500.0, "fixture must exceed old hard-coded limit");
    let position = shell::Point {
        x: f64::from(scroll_layout.rect[0] + 10.0),
        y: f64::from(scroll_layout.rect[1] + 10.0),
    };
    let inner_range = f64::from(
        source
            .tree
            .layout(inner_scroll_node)
            .unwrap()
            .scroll_height(),
    );
    source.actor.view.renderer.get_mut(&scroll).unwrap().scroll = range - 60.0;
    source
        .actor
        .view
        .renderer
        .get_mut(&inner_scroll)
        .unwrap()
        .scroll = inner_range - 5.0;
    source.build_frame(&mut tcx, 720, 620);
    let inner_rect = source.widget_layout_rect(inner_scroll).unwrap();
    let outer_rect = source.widget_layout_rect(scroll).unwrap();
    let nested_position = shell::Point {
        x: f64::from(inner_rect[0] + 10.0),
        y: f64::from(inner_rect[1].max(outer_rect[1]) + 5.0),
    };
    let hit = source.hit(nested_position).unwrap();
    assert!(hit == inner_scroll || hit == scrolled_action);
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: nested_position,
        delta_x: 0.0,
        delta_y: 40.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    assert_eq!(
        source.actor.view.renderer[&inner_scroll].scroll,
        inner_range
    );
    assert_eq!(source.actor.view.renderer[&scroll].scroll, range - 25.0);
    source.build_frame(&mut tcx, 720, 620);
    source
        .actor
        .view
        .renderer
        .get_mut(&inner_scroll)
        .unwrap()
        .scroll = 5.0;
    source.build_frame(&mut tcx, 720, 620);
    let inner_placed = source
        .placed
        .iter()
        .find(|node| node.node_id == inner_scroll_node)
        .unwrap();
    let clip = inner_placed.clip.unwrap();
    let reverse_position = shell::Point {
        x: f64::from(inner_placed.rect[0].max(clip[0]) + 5.0),
        y: f64::from(inner_placed.rect[1].max(clip[1]) + 5.0),
    };
    assert_eq!(source.hit(reverse_position), Some(inner_scroll));
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: reverse_position,
        delta_x: 0.0,
        delta_y: -40.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    assert_eq!(source.actor.view.renderer[&inner_scroll].scroll, 0.0);
    assert_eq!(source.actor.view.renderer[&scroll].scroll, range - 60.0);
    // Keep the button revealed for the existing outer-wheel clipping checks.
    source
        .actor
        .view
        .renderer
        .get_mut(&inner_scroll)
        .unwrap()
        .scroll = inner_range;
    source.build_frame(&mut tcx, 720, 620);
    let tab_limit = source.actor.view.bindings.len();
    for reverse in [false, true] {
        source.actor.view.renderer.get_mut(&scroll).unwrap().scroll = 0.0;
        source
            .actor
            .view
            .renderer
            .get_mut(&inner_scroll)
            .unwrap()
            .scroll = 0.0;
        source.build_frame(&mut tcx, 720, 620);
        source.set_focus(None);
        for _ in 0..tab_limit {
            let response = source.handle_event(key_event(
                "Tab",
                if reverse {
                    shell::Modifiers::SHIFT
                } else {
                    shell::Modifiers::empty()
                },
            ));
            assert!(response.handled);
            assert!(response.request_redraw);
            if source.focus == Some(scrolled_action) {
                break;
            }
        }
        assert_eq!(source.focus, Some(scrolled_action));
        source.build_frame(&mut tcx, 720, 620);
        let rect = source.widget_layout_rect(scrolled_action).unwrap();
        assert_eq!(
            source.hit(shell::Point {
                x: f64::from(rect[0] + 10.0),
                y: f64::from(rect[1] + 10.0)
            }),
            Some(scrolled_action)
        );
        assert!(source.actor.view.renderer[&scroll].scroll > 1500.0);
        assert!(source.actor.view.renderer[&inner_scroll].scroll > 1500.0);
    }
    let focused_rect = source.widget_layout_rect(scrolled_action).unwrap();
    assert_eq!(
        source.hit(shell::Point {
            x: f64::from(focused_rect[0] + 10.0),
            y: f64::from(focused_rect[1] + 10.0)
        }),
        Some(scrolled_action)
    );
    assert!(source.actor.view.renderer[&scroll].scroll > 1500.0);
    assert!(source.actor.view.renderer[&inner_scroll].scroll > 1500.0);
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.focus, Some(scrolled_action));
    assert_eq!(
        source.widget_layout_rect(scrolled_action).unwrap(),
        focused_rect
    );
    println!(
        "PASS native Tab/Shift+Tab events reveal nested offscreen controls and replacement retains focused geometry"
    );
    source.set_focus(None);
    source.actor.view.renderer.get_mut(&scroll).unwrap().scroll = 0.0;
    source
        .actor
        .view
        .renderer
        .get_mut(&inner_scroll)
        .unwrap()
        .scroll = 0.0;
    source.build_frame(&mut tcx, 720, 620);
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: scrolled_action
    }));
    assert_eq!(
        source.semantic_snapshot().unwrap().focus,
        Some(scrolled_action)
    );
    source.build_frame(&mut tcx, 720, 620);
    let semantic_focus_rect = source.widget_layout_rect(scrolled_action).unwrap();
    assert_eq!(
        source.hit(shell::Point {
            x: f64::from(semantic_focus_rect[0] + 10.0),
            y: f64::from(semantic_focus_rect[1] + 10.0)
        }),
        Some(scrolled_action)
    );
    assert!(source.actor.view.renderer[&scroll].scroll > 1500.0);
    assert!(source.actor.view.renderer[&inner_scroll].scroll > 1500.0);
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(
        source.semantic_snapshot().unwrap().focus,
        Some(scrolled_action)
    );
    assert_eq!(
        source.widget_layout_rect(scrolled_action).unwrap(),
        semantic_focus_rect
    );
    println!(
        "PASS semantic Focus reveals nested clipped targets and preserves focus/geometry through replacement"
    );
    // Restore the offsets used by the independent repeated-reload proof.
    source.actor.view.renderer.get_mut(&scroll).unwrap().scroll = range - 60.0;
    source
        .actor
        .view
        .renderer
        .get_mut(&inner_scroll)
        .unwrap()
        .scroll = inner_range;
    source.build_frame(&mut tcx, 720, 620);
    let before_reload_rect = source.widget_layout_rect(inner_scroll).unwrap();
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.actor.view.bindings[&root].node.kind, COLUMN);
    assert_eq!(source.ids[&inner_scroll], inner_scroll_node);
    assert_eq!(
        source.widget_layout_rect(inner_scroll).unwrap(),
        before_reload_rect
    );
    assert_eq!(
        source.actor.view.renderer[&inner_scroll].scroll,
        inner_range
    );
    assert_eq!(source.actor.view.renderer[&scroll].scroll, range - 60.0);
    println!("PASS repeated compatible replacement retains nested scroll geometry and offsets");
    println!("PASS nested wheel passes unconsumed distance to the outer scroll in both directions");
    for (delta, expected) in [(1_000_000.0, range), (-1_000_000.0, 0.0)] {
        source.handle_event(UiEvent::Wheel(shell::WheelEvent {
            position,
            delta_x: 0.0,
            delta_y: delta,
            delta_mode: shell::WheelDeltaMode::Pixel,
            phase: shell::GesturePhase::Changed,
            modifiers: Default::default(),
        }));
        source.build_frame(&mut tcx, 720, 620);
        assert_eq!(source.actor.view.renderer[&scroll].scroll, expected);
        let action_rect = source.widget_layout_rect(scrolled_action).unwrap();
        let point = shell::Point {
            x: f64::from(action_rect[0] + 10.0),
            y: f64::from(action_rect[1] + 10.0),
        };
        if expected > 0.0 {
            assert_eq!(source.hit(point), Some(scrolled_action));
        } else {
            assert_ne!(source.hit(point), Some(scrolled_action));
        }

        assert_eq!(
            f64::from(
                source
                    .placed
                    .iter()
                    .find(|node| node.node_id == scroll_node)
                    .unwrap()
                    .scroll
                    .offset[1]
            ),
            expected
        );
    }
    source.set_reduced_motion(true); // Check normalization independently of animated presentation.
    for mode in [shell::WheelDeltaMode::Line, shell::WheelDeltaMode::Pixel] {
        source.handle_event(UiEvent::Wheel(shell::WheelEvent {
            position,
            delta_x: 0.0,
            delta_y: shell::WHEEL_LINE_DELTA,
            delta_mode: mode,
            phase: shell::GesturePhase::Changed,
            modifiers: Default::default(),
        }));
        source.build_frame(&mut tcx, 720, 620);
        assert_eq!(
            source.actor.view.renderer[&scroll].scroll,
            shell::WHEEL_LINE_DELTA
        );
        source.handle_event(UiEvent::Wheel(shell::WheelEvent {
            position,
            delta_x: 0.0,
            delta_y: -shell::WHEEL_LINE_DELTA,
            delta_mode: mode,
            phase: shell::GesturePhase::Changed,
            modifiers: Default::default(),
        }));
        source.build_frame(&mut tcx, 720, 620);
        assert_eq!(source.actor.view.renderer[&scroll].scroll, 0.0);
    }
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: input }));
    assert_eq!(source.semantic_snapshot().unwrap().focus, Some(input));
    assert!(source.handle_semantic_action(shell::SemanticAction::Blur { target: input }));
    assert_eq!(source.semantic_snapshot().unwrap().focus, None);
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: root }));
    assert!(!source.handle_semantic_action(shell::SemanticAction::Focus { target: u64::MAX }));
    let count = source.actor.integer("count");
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    assert_eq!(source.actor.integer("count"), count + 10);
    let published_revision = source.semantic_snapshot().unwrap().revision;
    source.set_semantics_enabled(false);
    assert!(source.semantic_snapshot().is_none());
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    assert_eq!(source.actor.integer("count"), count + 10);
    source.set_semantics_enabled(true);
    assert!(source.semantic_snapshot().unwrap().revision > published_revision);
    let reenabled = source.semantic_snapshot().unwrap();
    source.set_semantics_enabled(true);
    assert!(Arc::ptr_eq(
        &reenabled,
        &source.semantic_snapshot().unwrap()
    ));
    println!(
        "PASS semantic focus/blur and current-code click survive replacement; disabled, unknown, non-action and disabled-service targets reject"
    );
    source.replace(row_root)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.actor.view.bindings[&root].node.kind, ROW);
    assert!(
        source.handle_semantic_action(shell::SemanticAction::ScrollIntoView {
            target: scrolled_action
        })
    );
    source.build_frame(&mut tcx, 720, 620);
    let rect = source.widget_layout_rect(scrolled_action).unwrap();
    assert_eq!(
        source.hit(shell::Point {
            x: f64::from(rect[0] + 10.0),
            y: f64::from(rect[1] + 10.0)
        }),
        Some(scrolled_action)
    );
    let outer_offset = source.actor.view.renderer[&scroll].scroll;
    let inner_offset = source.actor.view.renderer[&inner_scroll].scroll;
    source.replace(row_root)?;
    source.build_frame(&mut tcx, 720, 620);
    assert_eq!(source.widget_layout_rect(scrolled_action).unwrap(), rect);
    assert_eq!(source.actor.view.renderer[&scroll].scroll, outer_offset);
    assert_eq!(
        source.actor.view.renderer[&inner_scroll].scroll,
        inner_offset
    );
    assert_eq!(source.ids[&inner_scroll], inner_scroll_node);
    println!(
        "PASS Column-to-Row reflow reveals the nested button and repeated Row replacement preserves geometry and offsets"
    );
    println!(
        "PASS semantic auto keys retain native editor allocation, state slot, text, selection, IME, focus and scroll through source movement/style/sibling changes; ambiguous identity rejects replacement and button invokes new code"
    );
    Ok(())
}

#[path = "native_interval.rs"]
mod interval_window;
pub use interval_window::proof as interval_window_proof;

#[path = "native_checkbox.rs"]
mod checkbox_fixture;
pub use checkbox_fixture::checkbox_proof;

#[path = "native_read_only.rs"]
mod read_only_fixture;
pub use read_only_fixture::proof as read_only_proof;

#[cfg(test)]
mod native_cancel_tests {
    use super::*;
    use crate::{ActorId, Runtime, native_model::NativeFamily};
    #[test]
    #[ignore = "requires cancellation-button Counter dylibs; scripts/verify_native_requests.py"]
    fn shortcut_cancels_retained_request_through_current_widget_action() {
        let base = std::env::var_os("ACTOR_TEST_LIBRARY").unwrap();
        let next = std::env::var_os("ACTOR_TEST_REPLACEMENT").unwrap();
        let directory =
            std::env::temp_dir().join(format!("native-cancel-shortcut-{}", std::process::id()));
        let root = ActorId(1);
        let peer = ActorId(2);
        let mut runtime = Runtime::new(&directory);
        runtime.spawn(root, Path::new(&base)).unwrap();
        runtime.spawn(peer, Path::new(&base)).unwrap();
        let model = NativeFamily::new(runtime, root).unwrap();
        let mut source = NativeSource::without_watch(model).unwrap();
        let input = source.actor.widget(root, key("counter.notes")).unwrap();
        let button = source.actor.widget(root, key("counter.increment")).unwrap();
        let caption = source.actor.widget(root, key("counter.task")).unwrap();
        source = source.primary_shortcut("k", button).unwrap();
        source
            .actor
            .set_widget_text(input, "a😀bc unsaved draft")
            .unwrap();
        source.sync().unwrap();
        source.set_focus(Some(input));
        source
            .editors
            .get_mut(&input)
            .unwrap()
            .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
            .unwrap();
        let mut text = TextContext::new();
        source.build_frame(&mut text, 640, 620);
        let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
        let selection = source.editors[&input].text_selection();
        let input_node = source.ids[&input];
        let caption_node = source.ids[&caption];
        let slot = source.actor.root_actor().slot_address("notes");
        let token = source
            .actor
            .root_actor_mut()
            .dispatch_request(Message {
                action: key("ask_token"),
                value: 2,
            })
            .unwrap()
            .unwrap();
        // Store only the integer identity in durable state; no old callback is retained.
        source
            .actor
            .root_actor_mut()
            .dispatch(Message {
                action: key("task_done"),
                value: token,
            })
            .unwrap();
        source.actor.reload_actor(root, Path::new(&next)).unwrap();
        source.actor.reload_actor(peer, Path::new(&next)).unwrap();
        source.sync().unwrap();
        source.handle_event(UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: "k".into(),
            key_without_modifiers: "k".into(),
            code: "KeyK".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: shell::Modifiers::CONTROL,
            repeat: false,
            synthetic: false,
        }));
        for _ in 0..4 {
            source.poll_async();
        }
        source.build_frame(&mut text, 640, 620);
        assert_eq!(
            source.actor.runtime().actor(peer).unwrap().integer("count"),
            0
        );
        assert_eq!(source.actor.root_actor().integer("task_result"), -4);
        assert_eq!(
            source
                .tree
                .get_node_context(caption_node)
                .unwrap()
                .text
                .as_deref(),
            Some("Background result: -4")
        );
        assert_eq!(source.ids[&caption], caption_node);
        assert_eq!(source.ids[&input], input_node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            editor
        );
        assert_eq!(source.editors[&input].text_selection(), selection);
        assert_eq!(source.focus, Some(input));
        assert_eq!(
            source.editors[&input].current_value(),
            Some("a😀bc unsaved draft")
        );
        assert_eq!(source.actor.root_actor().slot_address("notes"), slot);
        drop(source);
        assert!(std::fs::read_dir(&directory).unwrap().next().is_none());
        std::fs::remove_dir(directory).unwrap();
    }
}

#[path = "native_documents.rs"]
mod documents_fixture;
pub use documents_fixture::proof as documents_proof;

/// Verify editor names are semantic metadata independent of durable values and native identity.
pub fn editor_labels_proof(base: &Path, replacement: &Path, rejected: &Path) -> Result<(), String> {
    let mut actor = Actor::new(131, Path::new(".runtime"))?;
    actor.reload(base)?;
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    source.actor.set_text(key("notes"), "a😀bc draft")?;
    source.sync()?;
    let input = key("counter.notes");
    let area = key("counter.document");
    source.set_semantics_enabled(true);
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .map_err(|error| error.to_string())?;
    source.build_frame(&mut tcx, 720, 920);
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    let selection = source.editors[&input].text_selection();
    let state = source.actor.state_address();
    let slot = source.actor.slot_address("notes");
    let snapshot = source.semantic_snapshot().unwrap();
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.id == input)
            .unwrap()
            .label
            .as_deref(),
        Some("Notes")
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.id == area)
            .unwrap()
            .label
            .as_deref(),
        Some("Document")
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.id == input)
            .unwrap()
            .value
            .as_deref(),
        Some("a😀bc draft")
    );
    let revision = snapshot.revision;
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 720, 920);
    let snapshot = source.semantic_snapshot().unwrap();
    assert!(snapshot.revision > revision);
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.id == input)
            .unwrap()
            .label
            .as_deref(),
        Some("Edited notes")
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.id == area)
            .unwrap()
            .label
            .as_deref(),
        Some("Edited document")
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.id == area)
            .unwrap()
            .value
            .as_deref(),
        Some("a😀bc draft")
    );
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("notes"), slot);
    assert!(source.replace(rejected).is_err());
    source.build_frame(&mut tcx, 720, 920);
    assert_eq!(
        source.semantic_snapshot().unwrap().revision,
        snapshot.revision
    );
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|n| n.id == input)
            .unwrap()
            .label
            .as_deref(),
        Some("Edited notes")
    );
    println!(
        "PASS labeled TextInput/TextArea separate names from values, reload names with retained editor/node/state/focus/selection and reject invalid candidate without semantic publication"
    );
    Ok(())
}

/// Exercise a real string-list application without synthesizing OS input.
pub fn text_list_app_proof(base: &Path, replacement: &Path, rejected: &Path) -> Result<(), String> {
    let mut actor = Actor::new(133, Path::new(".runtime"))?;
    actor.reload(base)?;
    assert!(actor.view.root_dependencies.is_empty());
    assert_eq!(actor.scope_ids(), vec![key("labels.collection")]);
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("labels.draft");
    let add = key("labels.add");
    let scroll = key("labels.list");
    for navigation in [key("labels.previous"), key("labels.next")] {
        assert_eq!(source.actor.view.bindings[&navigation].node.disabled, 1);
    }
    // Empty and duplicate input retain the user's draft and explain the result.
    source.actor.dispatch_widget(add)?;
    assert_eq!(source.actor.text("labels.feedback"), "Enter a label.");
    source.actor.set_text(key("labels.draft"), "  ")?;
    source.actor.dispatch_widget(add)?;
    assert_eq!(source.actor.text("labels.feedback"), "Enter a label.");
    assert_eq!(source.actor.text("labels.draft"), "  ");
    source.actor.set_text(key("labels.draft"), " default ")?;
    source.actor.dispatch_widget(add)?;
    assert_eq!(
        source.actor.text("labels.feedback"),
        "This label already exists."
    );
    assert_eq!(source.actor.text("labels.draft"), " default ");
    source.actor.set_text(key("labels.draft"), " added 😀 ")?;
    source.actor.dispatch_widget(add)?;
    assert_eq!(
        source.actor.text_list("labels.items").last().unwrap(),
        "added 😀"
    );
    assert_eq!(source.actor.text("labels.draft"), "");
    assert_eq!(source.actor.text("labels.feedback"), "Label added.");
    // Grow the list while retaining a separate, unsaved editor draft.
    for value in 0..40 {
        source.actor.dispatch(Message {
            action: key("append"),
            value,
        })?;
    }
    source
        .actor
        .set_text(key("labels.draft"), "a😀bc pending")?;
    source.sync()?;
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .map_err(|error| error.to_string())?;
    source.set_semantics_enabled(true);
    source.build_frame(&mut tcx, 360, 520);
    let max_scroll = source
        .tree
        .layout(source.ids[&scroll])
        .unwrap()
        .scroll_height();
    assert!(max_scroll > 137.0);
    source.actor.view.renderer.get_mut(&scroll).unwrap().scroll = 137.0;
    source.build_frame(&mut tcx, 360, 520);
    let row = *source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.text == "标题😀")
        .unwrap()
        .0;
    let row_node = source.ids[&row];
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    let selection = source.editors[&input].text_selection();
    let state = source.actor.state_address();
    let slot = source.actor.slot_address("labels.items");
    let buffer = source.actor.text_list("labels.items").as_ptr();
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("labels.items"), slot);
    assert_eq!(source.actor.text_list("labels.items").as_ptr(), buffer);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(source.ids[&row], row_node);
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.actor.view.renderer[&scroll].scroll, 137.0);
    assert_eq!(source.actor.text("labels.draft"), "a😀bc pending");
    assert_eq!(source.actor.view.bindings[&add].text, "Add label");
    let snapshot = source.semantic_snapshot().unwrap();
    let editor_semantics = snapshot.nodes.iter().find(|node| node.id == input).unwrap();
    assert_eq!(editor_semantics.label.as_deref(), Some("New label"));
    assert_eq!(editor_semantics.value.as_deref(), Some("a😀bc pending"));
    let revision = snapshot.revision;
    assert!(source.replace(rejected).is_err());
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(source.semantic_snapshot().unwrap().revision, revision);
    source.actor.dispatch(Message {
        action: key("append"),
        value: 99,
    })?;
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(
        source.actor.text_list("labels.items").last().unwrap(),
        "new 99 😀"
    );
    assert_eq!(source.ids[&row], row_node);
    assert_eq!(source.ids[&input], node);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.actor.view.renderer[&scroll].scroll, 137.0);
    // Fixed controls must stay outside the scrolling region and within the viewport.
    let bounds = |key| {
        source
            .placed
            .iter()
            .find(|placed| placed.node_id == source.ids[&key])
            .unwrap()
            .rect
    };
    let input_bounds = bounds(input);
    let add_bounds = bounds(add);
    let list_bounds = bounds(scroll);
    assert!(input_bounds[0] >= 0.0 && input_bounds[2] <= 360.0);
    assert!(input_bounds[3] <= add_bounds[1] && add_bounds[3] <= list_bounds[1]);
    assert!(list_bounds[3] <= 520.0);
    for navigation in [key("labels.previous"), key("labels.next")] {
        let rect = bounds(navigation);
        assert!(rect[0] >= 0.0 && rect[2] <= 360.0);
        assert!(rect[1] >= list_bounds[3] && rect[3] <= 520.0);
    }
    let page_bounds = bounds(key("labels.page"));
    assert!(bounds(key("labels.previous"))[2] <= page_bounds[0]);
    assert!(page_bounds[2] <= bounds(key("labels.next"))[0]);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: add }));
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(
        source.actor.text_list("labels.items").last().unwrap(),
        "a😀bc pending"
    );
    assert_eq!(source.actor.text("labels.draft"), "");
    assert_eq!(source.actor.text("labels.feedback"), "Added label.");
    // A collection larger than the view-node budget remains usable through pages.
    for value in 100..400 {
        source.actor.dispatch(Message {
            action: key("append"),
            value,
        })?;
    }
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(source.actor.text_list("labels.items").len(), 345);
    source.actor.dispatch_widget(key("labels.next"))?;
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(source.actor.integer("labels.page"), 1);
    assert!(
        source
            .actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "new 119 😀")
    );
    source.actor.reload(base)?;
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(source.actor.integer("labels.page"), 1);
    assert_eq!(source.actor.text_list("labels.items").len(), 345);
    source.actor.dispatch_widget(key("labels.previous"))?;
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(source.actor.integer("labels.page"), 0);
    for _ in 0..5 {
        source.actor.dispatch_widget(key("labels.next"))?;
    }
    assert_eq!(source.actor.integer("labels.page"), 5);
    source.build_frame(&mut tcx, 360, 520);
    assert!(
        source
            .actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "new 399 😀")
    );
    assert_eq!(
        source.actor.view.bindings[&key("labels.next")]
            .node
            .disabled,
        1
    );
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::Click {
            target: key("labels.next")
        })
    );
    assert_eq!(source.actor.integer("labels.page"), 5);
    // A smaller collection projects a valid page, even if the stored page was larger.
    source.actor.dispatch(Message {
        action: key("seed"),
        value: 0,
    })?;
    source.build_frame(&mut tcx, 360, 520);
    assert_eq!(
        source.actor.view.bindings[&key("labels.page")].text,
        "Page 1 of 1"
    );
    assert!(
        source
            .actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "标题😀")
    );
    for navigation in [key("labels.previous"), key("labels.next")] {
        assert_eq!(source.actor.view.bindings[&navigation].node.disabled, 1);
    }
    source.actor.dispatch(Message {
        action: key("previous-page"),
        value: 0,
    })?;
    assert_eq!(source.actor.integer("labels.page"), 0);
    println!(
        "PASS text-list native app: empty/duplicate/add behavior, keyed rows, narrow layout, real editor allocation/node/Unicode selection/focus/scroll/state retention across rejected and successful DLL replacement; new-code append updates rows"
    );
    Ok(())
}

#[path = "native_button_arguments.rs"]
mod button_arguments_fixture;
pub use button_arguments_fixture::proof as button_arguments_proof;

#[path = "native_request_feedback.rs"]
mod request_feedback_fixture;
pub use request_feedback_fixture::proof as request_feedback_proof;

#[path = "native_seven_guis.rs"]
mod seven_guis_fixture;
pub use seven_guis_fixture::proof as seven_guis_proof;

#[path = "native_record_notepad.rs"]
mod record_notepad_fixture;
#[path = "native_router.rs"]
mod router_fixture;
pub use record_notepad_fixture::proof as record_notepad_proof;
pub use router_fixture::proof as router_proof;
pub use router_fixture::recovery_layout_proof as document_recovery_layout_proof;
pub use router_fixture::watch_proof as router_watch_proof;

#[path = "native_text_map.rs"]
mod text_map_fixture;
pub use text_map_fixture::migration_proof as text_map_migration_proof;
pub use text_map_fixture::proof as text_map_proof;

#[cfg(all(test, feature = "testing"))]
#[path = "native_select_tests.rs"]
mod select_tests;

#[path = "native_record_checkbox.rs"]
mod record_checkbox_fixture;
pub use record_checkbox_fixture::proof as record_checkbox_proof;

#[path = "native_record_slider.rs"]
mod record_slider_fixture;
pub use record_slider_fixture::proof as record_slider_proof;

#[path = "native_record_progress.rs"]
mod record_progress_fixture;
pub use record_progress_fixture::proof as record_progress_proof;

#[path = "native_record_select.rs"]
mod record_select_fixture;
pub use record_select_fixture::proof as record_select_proof;

#[path = "native_preferences.rs"]
mod preferences_fixture;
pub use preferences_fixture::proof as preferences_proof;

#[path = "native_record_map_text.rs"]
mod record_map_text_fixture;
pub use record_map_text_fixture::proof as record_map_text_proof;

#[path = "native_sdk_socket.rs"]
mod sdk_socket_fixture;
pub use sdk_socket_fixture::proof as sdk_socket_proof;

#[path = "native_tcp_client.rs"]
mod tcp_client_fixture;
pub use tcp_client_fixture::proof as tcp_client_proof;

#[cfg(all(test, feature = "testing"))]
#[path = "native_button_appearance_tests.rs"]
mod button_appearance_tests;

pub fn record_list_identity_proof(
    base: &Path,
    compatible: &Path,
    evolved: &Path,
    root: &Path,
) -> Result<(), String> {
    let mut actor = Actor::new(943, root)?;
    actor.reload(base)?;
    for id in [7, 8] {
        actor.dispatch(Message {
            action: key("list.append"),
            value: id,
        })?;
    }
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 320);
    let row = |id: u64| {
        id.to_le_bytes().iter().fold(key("list.row"), |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
    };
    let first = row(7);
    let second = row(8);
    let nodes = [source.ids[&first], source.ids[&second]];
    let first_before = source.widget_layout_rect(first).unwrap();
    source.set_focus(Some(first));
    source.actor.dispatch(Message {
        action: key("list.reverse"),
        value: 0,
    })?;
    source.sync()?;
    source.build_frame(&mut tcx, 480, 320);
    assert_eq!([source.ids[&first], source.ids[&second]], nodes);
    assert_eq!(source.focus, Some(first));
    assert_ne!(source.widget_layout_rect(first).unwrap(), first_before);
    source.replace(compatible)?;
    source.build_frame(&mut tcx, 480, 320);
    assert_eq!([source.ids[&first], source.ids[&second]], nodes);
    assert_eq!(source.focus, Some(first));
    source.replace(evolved)?;
    source.build_frame(&mut tcx, 480, 320);
    assert_eq!([source.ids[&first], source.ids[&second]], nodes);
    assert_eq!(source.focus, Some(first));
    source.actor.dispatch_widget(first)?;
    source.sync()?;
    source.build_frame(&mut tcx, 480, 320);
    assert!(source.ids.contains_key(&row(107)));
    assert_eq!([source.ids[&first], source.ids[&second]], nodes);
    assert_eq!(source.focus, Some(first));
    println!(
        "PASS direct record-list native keyed rows retain nodes/focus through reorder, DLL replacement, schema upgrade and current-code insertion"
    );
    Ok(())
}

#[path = "native_application.rs"]
mod application_fixture;
pub use application_fixture::editor_proof as application_editor_proof;
pub use application_fixture::watch_proof as application_watch_proof;

#[cfg(all(test, feature = "testing"))]
#[path = "native_reactive_style_tests.rs"]
mod reactive_style_tests;

#[path = "native_theme.rs"]
mod theme;

#[path = "native_hover.rs"]
mod hover;

#[cfg(all(test, feature = "testing"))]
#[path = "native_hover_tests.rs"]
mod hover_tests;

#[path = "native_choices.rs"]
mod choices;

#[cfg(all(test, feature = "testing"))]
#[path = "native_choices_tests.rs"]
mod choice_tests;

#[path = "native_motion.rs"]
mod motion;

#[path = "native_disclosure.rs"]
mod disclosure;
#[cfg(all(test, feature = "testing"))]
#[path = "native_disclosure_tests.rs"]
mod disclosure_tests;

#[cfg(all(test, feature = "testing"))]
#[path = "native_label_tests.rs"]
mod label_tests;
#[cfg(all(test, feature = "testing"))]
#[path = "native_popover_tests.rs"]
mod popover_tests;
#[cfg(all(test, feature = "testing"))]
#[path = "native_sync_bench.rs"]
mod sync_bench;
#[path = "native_toast.rs"]
mod toast;
#[cfg(all(test, feature = "testing"))]
#[path = "native_toast_tests.rs"]
mod toast_tests;

impl<M: NativeModel> NativeSource<M> {
    fn popover_contains(&self, panel: u64, mut key: u64) -> bool {
        while let Some(binding) = self.actor.view().bindings.get(&key) {
            if key == panel {
                return true;
            }
            key = binding.node.parent;
        }
        false
    }
    fn popover_owner(&self, mut key: u64) -> Option<u64> {
        while let Some(binding) = self.actor.view().bindings.get(&key) {
            if semantics::is_anchored_panel(binding.node.semantics.role) {
                return Some(key);
            }
            key = binding.node.parent;
        }
        None
    }
    fn active_popover(&self) -> Option<u64> {
        self.actor.view().order.iter().rev().copied().find(|key| {
            let n = self.actor.view().bindings[key].node;
            semantics::is_anchored_panel(n.semantics.role)
                && n.semantics.hidden == 0
                && self.in_modal(*key)
        })
    }
}

#[path = "native_tooltip.rs"]
mod tooltip;

#[cfg(all(test, feature = "testing"))]
#[path = "native_tooltip_tests.rs"]
mod tooltip_tests;

#[cfg(all(test, feature = "testing"))]
#[path = "native_table_tests.rs"]
mod table_tests;

#[path = "native_menu.rs"]
mod menu;
#[cfg(all(test, feature = "testing"))]
#[path = "native_menu_tests.rs"]
mod menu_tests;

#[cfg(all(test, feature = "testing"))]
#[path = "native_theme_tests.rs"]
mod theme_tests;

#[path = "native_icons.rs"]
mod icons;
pub(crate) use icons::validate as validate_icon;

#[path = "native_scroll.rs"]
mod scroll;
