//! Synchronous component integration tests through the real actor runtime.
//! No window, layout, text shaping, GPU, browser or dylib compilation is required.
use blockui::abi::{self, semantics};
pub use blockui_runtime::Dependency;
pub use blockui_runtime::application::Services;
use blockui_runtime::{Actor, Binding};
use std::collections::BTreeSet;
mod debug;
mod style;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Group,
    Menu,
    Menubar,
    Menuitem,
    Menuitemcheckbox,
    Menuitemradio,
    Table,
    Row,
    Columnheader,
    Cell,
    Button,
    Textbox,
    Checkbox,
    Switch,
    Tab,
    Tablist,
    Tabpanel,
    Radio,
    Radiogroup,
    Dialog,
    Heading,
    Alert,
    Status,
    Slider,
    Progressbar,
    Combobox,
    Region,
}
fn role(binding: &Binding) -> Option<Role> {
    Some(match binding.node.semantics.role {
        semantics::MENU_GROUP | semantics::CONTEXT_SURFACE => Role::Group,
        semantics::MENU_PANEL => Role::Menu,
        semantics::MENUBAR => Role::Menubar,
        semantics::MENU_ITEM | semantics::SUBMENU_TRIGGER => Role::Menuitem,
        semantics::MENU_CHECKBOX => Role::Menuitemcheckbox,
        semantics::MENU_RADIO => Role::Menuitemradio,
        semantics::TABLE => Role::Table,
        semantics::TABLE_ROW => Role::Row,
        semantics::COLUMN_HEADER => Role::Columnheader,
        semantics::CELL => Role::Cell,
        semantics::SWITCH => Role::Switch,
        semantics::TAB => Role::Tab,
        semantics::TAB_LIST => Role::Tablist,
        semantics::TAB_PANEL => Role::Tabpanel,
        semantics::RADIO => Role::Radio,
        semantics::RADIO_GROUP => Role::Radiogroup,
        semantics::HEADING => Role::Heading,
        semantics::ALERT => Role::Alert,
        semantics::STATUS => Role::Status,
        semantics::DISCLOSURE_PANEL | semantics::TOAST_VIEWPORT => Role::Region,
        _ => match binding.node.kind {
            abi::BUTTON => Role::Button,
            abi::ENTRY | abi::TEXTAREA => Role::Textbox,
            abi::CHECKBOX => Role::Checkbox,
            abi::DIALOG => Role::Dialog,
            abi::SLIDER => Role::Slider,
            abi::PROGRESS => Role::Progressbar,
            abi::SELECT => Role::Combobox,
            _ => return None,
        },
    })
}
/// A host-owned key. Every operation resolves the current projection again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Locator(u64);

/// Queries restricted to a mounted subtree, including its root. Results remain
/// host-owned locators; no builder, callback or borrowed actor context is retained.
pub struct Within<'screen> {
    screen: &'screen Screen,
    root: Locator,
}
impl Within<'_> {
    /// Inspect this visible subtree without evaluating observers or rendering.
    pub fn debug_tree(&self) -> String {
        debug::tree(self.screen, Some(self.root))
    }
    pub fn query_all_by_role(&self, role: Role, name: &str) -> Vec<Locator> {
        self.screen
            .query_all_by_role_in(Some(self.root), role, name)
    }
    pub fn get_by_role(&self, role: Role, name: &str) -> Result<Locator> {
        self.screen.get_by_role_in(Some(self.root), role, name)
    }
    pub fn query_all_by_text(&self, text: &str) -> Vec<Locator> {
        self.screen.query_all_by_text_in(Some(self.root), text)
    }
    pub fn get_by_text(&self, text: &str) -> Result<Locator> {
        self.screen.get_by_text_in(Some(self.root), text)
    }
}

/// Evaluation counters for the current node's independent reactive observers.
/// These count computations, not native layout, painting or elapsed time.
/// Compare snapshots within the same mount and implementation; replacement or
/// structural remount may reset counters. This value retains no actor or code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvaluationCounts {
    pub text: u64,
    pub style: u64,
}

/// Owned direct dependency snapshots. No callback or implementation pointer is
/// retained; a later successful evaluation may replace either set.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObserverDependencies {
    pub text: BTreeSet<Dependency>,
    pub style: BTreeSet<Dependency>,
}

pub struct Screen {
    actor: Actor,
    manual_time: Option<std::time::Duration>,
}
impl Screen {
    /// Inspect the current visible projection without running callbacks or a renderer.
    /// Includes keys and text/style evaluation counts. Output is bounded to 128
    /// nodes, 16 indentation levels and escaped 80-character label previews.
    /// This diagnostic format is intended for humans, not snapshot assertions.
    pub fn debug_tree(&self) -> String {
        debug::tree(self, None)
    }
    /// Mount an actor without rendering; failures include its Rust type and original cause.
    pub fn render<A: blockui::Actor>() -> Result<Self> {
        Ok(Self {
            actor: Actor::mount::<A>(1).map_err(|reason| {
                format!(
                    "render failed for actor {}: {reason}",
                    std::any::type_name::<A>()
                )
            })?,
            manual_time: None,
        })
    }
    /// Render with executable-owned business services for interaction tests.
    /// Handlers remain in the test executable; do not register plugin callbacks.
    /// The initial read-only projection cannot invoke business services.
    pub fn with_services<A: blockui::Actor>(services: Services) -> Result<Self> {
        let mut screen = Self::render::<A>()?;
        screen.set_services(services);
        Ok(screen)
    }
    /// Set executable-owned service handlers between test interactions.
    /// This also supports screens created with `with_manual_time`.
    pub fn set_services(&mut self, services: Services) {
        self.actor.set_services(services);
    }
    /// Replace the implementation through the production migration/projection
    /// path, preserving host state. This does not compile or unload a dylib;
    /// keep real dylib proofs as a separate validation layer.
    /// Rejection errors include the candidate actor type and original cause.
    pub fn replace<A: blockui::Actor>(&mut self) -> Result<()> {
        self.actor.replace_for_test::<A>().map_err(|reason| {
            format!(
                "replace failed for actor {}: {reason}",
                std::any::type_name::<A>()
            )
        })
    }
    /// Resolve an exact published identity, for example a named scope root.
    /// Child keys inside captured scopes follow their mount's namespace; use
    /// `within(...).get_by_role(...)` to avoid depending on those derived keys.
    pub fn get_by_key(&self, key: impl Into<blockui::Key>) -> Result<Locator> {
        let locator = Locator(key.into().0);
        self.binding(locator)?;
        Ok(locator)
    }
    /// Restrict queries to a currently visible subtree. A locator can be kept
    /// across updates/replacement and used to create a fresh query each time.
    pub fn within(&self, root: Locator) -> Result<Within<'_>> {
        self.binding(root)?;
        Ok(Within { screen: self, root })
    }
    /// Find the nearest mounted structural scope that owns this node (possibly
    /// the node itself), without reconstructing its component instance key.
    pub fn scope_of(&self, locator: Locator) -> Result<Locator> {
        self.binding(locator)?;
        let view = self.actor.projected_view();
        let mut key = locator.0;
        while let Some(binding) = view.bindings.get(&key) {
            if view.scope_dependencies.contains_key(&key) {
                return Ok(Locator(key));
            }
            key = binding.node.parent;
        }
        Err(self.element_error(
            "scope_of",
            locator,
            "element has no structural scope; mount a Scope before inspecting its boundary",
        ))
    }
    /// Render with synchronized application/timer clocks, without starting a
    /// real timer worker. Configure before any interaction admits timers.
    pub fn with_manual_time<A: blockui::Actor>() -> Result<Self> {
        let mut screen = Self::render::<A>()?;
        screen.actor.use_manual_timers_for_test()?;
        screen.actor.set_monotonic_ms_for_test(0)?;
        screen.manual_time = Some(std::time::Duration::ZERO);
        Ok(screen)
    }
    /// Advance virtual time and deliver due actions. Intervals coalesce missed
    /// ticks, matching production behavior. No sleeping or rendering occurs.
    /// Delivery errors include virtual time and admission count. Time remains advanced
    /// even if an action fails; its actor writes follow ordinary transactional rollback.
    pub fn advance_time(&mut self, elapsed: std::time::Duration) -> Result<usize> {
        let now = self
            .manual_time
            .ok_or("use Screen::with_manual_time before advancing time")?
            .checked_add(elapsed)
            .ok_or("manual time overflow")?;
        let millis = u64::try_from(now.as_millis()).map_err(|_| "manual time overflow")?;
        self.actor.set_monotonic_ms_for_test(millis)?;
        self.manual_time = Some(now);
        let admitted = self.actor.advance_timer_time_for_test(elapsed)?;
        self.actor.tick().map_err(|reason| {
            format!("advance_time failed at {now:?} after admitting {admitted} timer action(s): {reason}")
        })?;
        Ok(admitted)
    }
    /// Committed root projections, including mount and accepted replacement.
    /// Independent bindings/scopes and rejected turns do not increment this count.
    pub fn root_evaluations(&self) -> u64 {
        self.actor.projected_view().root_evaluations()
    }

    /// Snapshot the real runtime's text/style evaluation counters for a visible
    /// current node. Resolve the locator before the interaction and compare after
    /// it to verify that unrelated observers did not run.
    pub fn evaluations(&self, locator: Locator) -> Result<EvaluationCounts> {
        let binding = self.binding(locator)?;
        Ok(EvaluationCounts {
            text: binding.evaluations,
            style: binding.style_evaluations,
        })
    }
    /// Snapshot direct subscriptions without evaluating callbacks. Text and style
    /// are separate observers; computed dependencies are not flattened to state.
    pub fn dependencies(&self, locator: Locator) -> Result<ObserverDependencies> {
        let binding = self.binding(locator)?;
        Ok(ObserverDependencies {
            text: binding.text_dependencies.clone(),
            style: binding.style_dependencies.clone(),
        })
    }
    /// Snapshot the root builder's direct subscriptions without evaluating it.
    /// Scope builders and text/style observers keep their own dependency sets.
    pub fn root_dependencies(&self) -> BTreeSet<Dependency> {
        self.actor.projected_view().root_dependencies.clone()
    }
    /// Snapshot evaluations of a registered typed memo without evaluating it.
    /// Admission evaluates registered memos for validation. Compare within the
    /// current implementation; replacement can reset counters.
    pub fn computed_evaluations<T>(&self, computed: blockui::Computed<T>) -> Result<u64> {
        let key = computed.key();
        if self.actor.computed_dependencies_by_key(key).is_none() {
            return Err(format!("unregistered computed {key:#018x}"));
        }
        Ok(self.actor.computed_evaluations_by_key(key))
    }
    /// Snapshot a record memo's evaluations without decoding or evaluating it.
    pub fn record_computed_evaluations<T: blockui::Record>(
        &self,
        computed: blockui::RecordComputed<T>,
    ) -> Result<u64> {
        let key = computed.key();
        if self.actor.computed_dependencies_by_key(key).is_none() {
            return Err(format!("unregistered computed {key:#018x}"));
        }
        Ok(self.actor.computed_evaluations_by_key(key))
    }
    /// Snapshot a registered memo's direct inputs without evaluating the memo.
    pub fn computed_dependencies<T>(
        &self,
        computed: blockui::Computed<T>,
    ) -> Result<BTreeSet<Dependency>> {
        let key = computed.key();
        self.actor
            .computed_dependencies_by_key(key)
            .ok_or_else(|| format!("unregistered computed {key:#018x}"))
    }
    /// Snapshot a mounted scope builder's direct inputs. Nested text/style
    /// observers keep their own sets; use `scope_of` to find a boundary first.
    pub fn scope_dependencies(&self, scope: Locator) -> Result<BTreeSet<Dependency>> {
        self.binding(scope)?;
        self.actor
            .projected_view()
            .scope_dependencies
            .get(&scope.0)
            .map(|scope| scope.dependencies.clone())
            .ok_or_else(|| {
                self.element_error("scope_dependencies", scope,
                "element is not a structural scope; use scope_of(element) to locate its boundary")
            })
    }
    /// Snapshot builder executions of a visible mounted scope. Use `scope_of`
    /// to find an element's nearest scope first. Cached reuse does not increment
    /// this count; failed turns do not publish it. Remount/replacement may reset it.
    pub fn scope_evaluations(&self, scope: Locator) -> Result<u64> {
        self.binding(scope)?;
        self.actor
            .projected_view()
            .scope_dependencies
            .get(&scope.0)
            .map(|scope| scope.evaluations)
            .ok_or_else(|| {
                self.element_error("scope_evaluations", scope,
                "element is not a structural scope; use scope_of(element) to locate its boundary")
            })
    }
    /// Inspect production dependency/evaluation counters or state when required.
    pub fn actor(&self) -> &Actor {
        &self.actor
    }
    fn visible(&self, mut key: u64) -> bool {
        let view = self.actor.projected_view();
        while let Some(binding) = view.bindings.get(&key) {
            if binding.node.semantics.hidden != 0 {
                return false;
            }
            key = binding.node.parent;
        }
        true
    }
    fn in_subtree(&self, root: Option<Locator>, mut key: u64) -> bool {
        let Some(root) = root else {
            return true;
        };
        let view = self.actor.projected_view();
        while let Some(binding) = view.bindings.get(&key) {
            if key == root.0 {
                return true;
            }
            key = binding.node.parent;
        }
        false
    }
    fn accessible_name<'a>(&'a self, binding: &'a Binding) -> &'a str {
        let view = self.actor.projected_view();
        let label = if binding.node.semantics.role == semantics::TOAST_VIEWPORT {
            "Notifications"
        } else if binding.node.semantics.labelled_by != 0 {
            view.bindings
                .get(&binding.node.semantics.labelled_by)
                .and_then(|label| {
                    if label.node.semantics.role == semantics::CONTEXT_SURFACE {
                        view.bindings.get(&label.node.semantics.labelled_by)
                    } else {
                        Some(label)
                    }
                })
                .map(|b| b.text.as_str())
                .unwrap_or("")
        } else {
            binding.semantic_label.as_deref().unwrap_or(&binding.text)
        };
        if binding.node.kind == abi::CHECKBOX {
            label
                .strip_prefix("☑ ")
                .or_else(|| label.strip_prefix("☐ "))
                .unwrap_or(label)
        } else {
            label
        }
    }
    fn query_context(&self, root: Option<Locator>) -> String {
        match root {
            None => "screen".into(),
            Some(root) => {
                let name = self
                    .actor
                    .projected_view()
                    .bindings
                    .get(&root.0)
                    .map(|binding| self.accessible_name(binding))
                    .unwrap_or("");
                format!(
                    "subtree {:#018x} named {:?}",
                    root.0,
                    Self::query_preview(name)
                )
            }
        }
    }
    // Build bounded diagnostics only on query failure, without evaluating UI code.
    fn query_candidates(
        &self,
        root: Option<Locator>,
        wanted: Option<Role>,
        matches: &[Locator],
    ) -> String {
        let view = self.actor.projected_view();
        let mut candidates = Vec::new();
        for key in &view.order {
            let binding = &view.bindings[key];
            if !self.visible(*key)
                || !self.in_subtree(root, *key)
                || wanted.is_some_and(|wanted| role(binding) != Some(wanted))
                || (!matches.is_empty() && !matches.contains(&Locator(*key)))
            {
                continue;
            }
            let label = if wanted.is_some() {
                self.accessible_name(binding)
            } else {
                &binding.text
            };
            if wanted.is_none() && label.is_empty() {
                continue;
            }
            if candidates.len() == 8 {
                candidates.push("…".into());
                break;
            }
            let preview = Self::query_preview(label);
            candidates.push(format!("{preview:?} (key {key:#018x})"));
        }
        if candidates.is_empty() {
            "none".into()
        } else {
            candidates.join(", ")
        }
    }
    fn query_preview(label: &str) -> String {
        let mut preview: String = label.chars().take(80).collect();
        if preview.len() < label.len() {
            preview.push('…');
        }
        preview
    }
    pub fn query_all_by_role(&self, wanted: Role, name: &str) -> Vec<Locator> {
        self.query_all_by_role_in(None, wanted, name)
    }
    fn query_all_by_role_in(
        &self,
        root: Option<Locator>,
        wanted: Role,
        name: &str,
    ) -> Vec<Locator> {
        let view = self.actor.projected_view();
        view.order
            .iter()
            .copied()
            .filter(|key| {
                let binding = &view.bindings[key];
                let label = self.accessible_name(binding);
                role(binding) == Some(wanted)
                    && label == name
                    && self.visible(*key)
                    && self.in_subtree(root, *key)
            })
            .map(Locator)
            .collect()
    }
    /// Missing and ambiguous queries are errors, rather than selecting the first match.
    /// Errors include up to eight visible candidates with bounded labels and keys.
    /// Requested names and candidate labels use escaped 80-character previews;
    /// matching always compares the full strings.
    /// Subtree queries restrict diagnostics to that subtree; callbacks are not run.
    pub fn get_by_role(&self, role: Role, name: &str) -> Result<Locator> {
        self.get_by_role_in(None, role, name)
    }
    fn get_by_role_in(&self, root: Option<Locator>, role: Role, name: &str) -> Result<Locator> {
        let matches = self.query_all_by_role_in(root, role, name);
        match matches.as_slice() {
            [one] => Ok(*one),
            _ => Err(format!(
                "in {}: expected one {role:?} named {:?}, found {}; visible candidates: {}",
                self.query_context(root),
                Self::query_preview(name),
                matches.len(),
                self.query_candidates(root, Some(role), &matches)
            )),
        }
    }
    /// Exact current projected text, independent of accessible names. Missing
    /// and ambiguous matches are errors, as with role queries.
    pub fn query_all_by_text(&self, text: &str) -> Vec<Locator> {
        self.query_all_by_text_in(None, text)
    }
    /// Find exact full text; failures show bounded previews as with role queries.
    pub fn get_by_text(&self, text: &str) -> Result<Locator> {
        self.get_by_text_in(None, text)
    }
    fn query_all_by_text_in(&self, root: Option<Locator>, text: &str) -> Vec<Locator> {
        let view = self.actor.projected_view();
        view.order
            .iter()
            .copied()
            .filter(|key| {
                view.bindings[key].text == text && self.visible(*key) && self.in_subtree(root, *key)
            })
            .map(Locator)
            .collect()
    }
    fn get_by_text_in(&self, root: Option<Locator>, text: &str) -> Result<Locator> {
        let matches = self.query_all_by_text_in(root, text);
        match matches.as_slice() {
            [one] => Ok(*one),
            _ => Err(format!(
                "in {}: expected one element with text {:?}, found {}; visible candidates: {}",
                self.query_context(root),
                Self::query_preview(text),
                matches.len(),
                self.query_candidates(root, None, &matches)
            )),
        }
    }
    fn binding(&self, locator: Locator) -> Result<&Binding> {
        if !self.visible(locator.0) {
            return Err(format!("element is hidden (key {:#018x})", locator.0));
        }
        self.actor
            .projected_view()
            .bindings
            .get(&locator.0)
            .ok_or_else(|| format!("element is no longer mounted (key {:#018x})", locator.0))
    }
    fn element_error(&self, operation: &str, locator: Locator, reason: &str) -> String {
        let identity = self
            .actor
            .projected_view()
            .bindings
            .get(&locator.0)
            .map(|binding| {
                format!(
                    "{} {:?}",
                    role(binding)
                        .map(|role| format!("{role:?}"))
                        .unwrap_or_else(|| format!("Node({})", binding.node.kind)),
                    Self::query_preview(self.accessible_name(binding))
                )
            })
            .unwrap_or_else(|| "unmounted element".into());
        format!(
            "{operation} failed for {identity} (key {:#018x}): {reason}",
            locator.0
        )
    }
    fn binding_for(&self, operation: &str, locator: Locator) -> Result<&Binding> {
        self.binding(locator)
            .map_err(|reason| self.element_error(operation, locator, &reason))
    }
    pub fn click(&mut self, locator: Locator) -> Result<()> {
        let b = self.binding_for("click", locator)?;
        if !matches!(
            role(b),
            Some(
                Role::Button
                    | Role::Menuitem
                    | Role::Menuitemcheckbox
                    | Role::Menuitemradio
                    | Role::Tab
                    | Role::Radio
                    | Role::Switch
                    | Role::Checkbox
            )
        ) {
            return Err(self.element_error("click", locator, "element is not activatable"));
        }
        self.actor
            .dispatch_widget(locator.0)
            .map_err(|reason| self.element_error("click", locator, &reason))
    }
    /// Activate the nearest context target without requiring layout or a window.
    pub fn context_menu(&mut self, locator: Locator) -> Result<()> {
        self.binding_for("context_menu", locator)?;
        let mut key = locator.0;
        while let Some(binding) = self.actor.projected_view().bindings.get(&key) {
            if semantics::is_anchored_panel(binding.node.semantics.role) {
                break;
            }
            if semantics::is_context_trigger(binding.node.semantics.role) {
                return self
                    .actor
                    .dispatch_widget(key)
                    .map_err(|reason| self.element_error("context_menu", locator, &reason));
            }
            key = binding.node.parent;
        }
        Err(self.element_error("context_menu", locator, "element has no context target"))
    }
    pub fn input(&mut self, locator: Locator, value: &str) -> Result<()> {
        if role(self.binding_for("input", locator)?) != Some(Role::Textbox) {
            return Err(self.element_error("input", locator, "element is not a textbox"));
        }
        self.actor
            .edit_text(locator.0, value)
            .map_err(|reason| self.element_error("input", locator, &reason))
    }
    pub fn select(&mut self, locator: Locator, value: i64) -> Result<()> {
        if role(self.binding_for("select", locator)?) != Some(Role::Combobox) {
            return Err(self.element_error("select", locator, "element is not a combobox"));
        }
        self.actor
            .edit_select(locator.0, value)
            .map_err(|reason| self.element_error("select", locator, &reason))
    }
    fn press_error(&self, locator: Locator, key: &str, reason: &str) -> String {
        self.element_error(
            "press",
            locator,
            &format!("key {:?}: {reason}", Self::query_preview(key)),
        )
    }
    /// Exercise the same choice navigation as native; returns the new choice.
    /// This does not simulate OS focus, physical key input or geometry.
    pub fn press(&mut self, locator: Locator, key: &str) -> Result<Locator> {
        self.binding(locator)
            .map_err(|reason| self.press_error(locator, key, &reason))?;
        if let Some(next) = blockui_runtime::interaction::disclosure_target(
            self.actor.projected_view(),
            locator.0,
            key,
        ) {
            return Ok(Locator(next));
        }
        if matches!(key, "Enter" | " " | "Space") {
            self.click(locator)
                .map_err(|reason| self.press_error(locator, key, &reason))?;
            return Ok(locator);
        }
        let next = blockui_runtime::interaction::choice_target(
            self.actor.projected_view(),
            locator.0,
            key,
        )
        .ok_or_else(|| self.press_error(locator, key, "unsupported component key"))?;
        self.actor
            .dispatch_widget(next)
            .map_err(|reason| self.press_error(Locator(next), key, &reason))?;
        Ok(Locator(next))
    }
    pub fn expanded(&self, locator: Locator) -> Result<bool> {
        match self.binding(locator)?.node.semantics.expanded {
            1 => Ok(false),
            2 => Ok(true),
            _ => Err("element is not expandable".into()),
        }
    }
    pub fn selected(&self, locator: Locator) -> Result<bool> {
        let b = self.binding(locator)?;
        if !matches!(role(b), Some(Role::Tab | Role::Radio)) {
            return Err("element is not selectable".into());
        }
        Ok(b.node.semantics.selected != 0)
    }
    pub fn checked(&self, locator: Locator) -> Result<bool> {
        self.binding(locator)?;
        self.actor
            .checkbox_value(locator.0)
            .ok_or_else(|| "element is not checkable".into())
    }
    pub fn disabled(&self, locator: Locator) -> Result<bool> {
        let node = self.binding(locator)?.node;
        Ok(node.disabled != 0 || node.semantics.locked != 0)
    }
    /// Copy the node's current committed presentation without evaluating builders
    /// or rendering. Zero dimensions and inherited theme/density remain unresolved.
    /// Resolves the locator again, so removed/hidden elements return an error.
    pub fn style(&self, locator: Locator) -> Result<blockui::ViewStyle> {
        style::snapshot(self.binding(locator)?.node.presentation)
    }
    pub fn text(&self, locator: Locator) -> Result<&str> {
        Ok(&self.binding(locator)?.text)
    }
    /// Compare exact committed text without evaluating observers or rendering.
    /// Mismatches include the current element identity and a bounded UI tree;
    /// text previews are bounded, but equality compares the complete strings.
    pub fn expect_text(&self, locator: Locator, expected: &str) -> Result<()> {
        let binding = self.binding_for("expect_text", locator)?;
        if binding.text == expected {
            return Ok(());
        }
        let difference = binding
            .text
            .chars()
            .zip(expected.chars())
            .position(|(actual, expected)| actual != expected)
            .unwrap_or_else(|| binding.text.chars().count().min(expected.chars().count()));
        let reason = format!(
            "expected text {:?}, received {:?}; first difference at character {}",
            Self::query_preview(expected),
            Self::query_preview(&binding.text),
            difference + 1
        );
        Err(format!(
            "{}; current UI:\n{}",
            self.element_error("expect_text", locator, &reason),
            self.debug_tree()
        ))
    }
}
