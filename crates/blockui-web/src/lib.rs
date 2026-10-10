//! Statically linked release applications. Browser transport contains owned data only.
use blockui_abi::*;
use blockui_motion::{Easing, Key as MotionKey, Property, SWITCH_DURATION, Timeline};
use blockui_runtime::Actor;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

pub struct Application {
    actor: Actor,
    motion: Timeline,
    motion_dirty: BTreeSet<MotionKey>,
    menu_hover: blockui_runtime::interaction::menu_hover::Controller,
    menus: BTreeMap<u64, blockui_runtime::interaction::menu::Controller>,
    tooltip: blockui_runtime::interaction::tooltip::Controller,
}
impl Application {
    pub fn mount<A: blockui::Actor>() -> Result<Self, String> {
        Ok(Self::from_actor(Actor::mount::<A>(1)?))
    }
    fn from_actor(actor: Actor) -> Self {
        Self {
            actor,
            motion: Timeline::default(),
            motion_dirty: BTreeSet::new(),
            tooltip: Default::default(),
            menus: Default::default(),
            menu_hover: Default::default(),
        }
    }
    /// Lossless data-only checkpoint; IDs, integers and float bits use strings.
    pub fn checkpoint(&self) -> Result<String, String> {
        use blockui_runtime::Value as StateValue;
        let storage = self.actor.checkpoint()?;
        let slots: Vec<_> = storage
            .slots
            .iter()
            .map(|(key, value)| {
                let (kind, value) = match &**value {
                    StateValue::Integer(v) => ("integer", json!(v.to_string())),
                    StateValue::Float(v) => ("float", json!(v.to_bits().to_string())),
                    StateValue::Boolean(v) => ("boolean", json!(v)),
                    StateValue::Text(v) => ("text", json!(v)),
                    StateValue::IntegerList(v) => (
                        "integers",
                        json!(v.iter().map(i64::to_string).collect::<Vec<_>>()),
                    ),
                    StateValue::TextList(v) => ("texts", json!(v)),
                };
                json!({"key":key.to_string(),"kind":kind,"value":value})
            })
            .collect();
        let checkpoint = json!({"format":1,"slots":slots}).to_string();
        if checkpoint.len() > 16 * 1024 * 1024 {
            return Err("checkpoint exceeds 16 MiB".into());
        }
        Ok(checkpoint)
    }
    pub fn restore<A: blockui::Actor>(checkpoint: &str) -> Result<Self, String> {
        use blockui_runtime::{Storage, Value as StateValue};
        if checkpoint.len() > 16 * 1024 * 1024 {
            return Err("checkpoint exceeds 16 MiB".into());
        }
        let input: Value = serde_json::from_str(checkpoint).map_err(|e| e.to_string())?;
        if input["format"].as_u64() != Some(1) {
            return Err("unsupported checkpoint format".into());
        }
        let slots = input["slots"]
            .as_array()
            .filter(|slots| slots.len() <= 256)
            .ok_or("invalid checkpoint slots")?;
        let mut storage = Storage::default();
        for slot in slots {
            let key = slot["key"]
                .as_str()
                .ok_or("invalid checkpoint key")?
                .parse::<u64>()
                .map_err(|_| "invalid checkpoint key")?;
            let value = &slot["value"];
            let text = || value.as_str().ok_or("invalid checkpoint value");
            let value = match slot["kind"].as_str() {
                Some("integer") => {
                    StateValue::Integer(text()?.parse().map_err(|_| "invalid checkpoint integer")?)
                }
                Some("float") => StateValue::Float(f64::from_bits(
                    text()?
                        .parse()
                        .map_err(|_| "invalid checkpoint float bits")?,
                )),
                Some("boolean") => {
                    StateValue::Boolean(value.as_bool().ok_or("invalid checkpoint boolean")?)
                }
                Some("text") => StateValue::Text(text()?.to_owned()),
                Some("integers") => StateValue::IntegerList(
                    value
                        .as_array()
                        .filter(|v| v.len() <= MAX_LIST_ITEMS)
                        .ok_or("invalid checkpoint list")?
                        .iter()
                        .map(|v| {
                            v.as_str()
                                .ok_or("invalid checkpoint integer")?
                                .parse()
                                .map_err(|_| "invalid checkpoint integer")
                        })
                        .collect::<Result<_, _>>()?,
                ),
                Some("texts") => StateValue::TextList(
                    value
                        .as_array()
                        .filter(|v| v.len() <= MAX_LIST_ITEMS)
                        .ok_or("invalid checkpoint list")?
                        .iter()
                        .map(|v| {
                            v.as_str()
                                .map(str::to_owned)
                                .ok_or("invalid checkpoint text")
                        })
                        .collect::<Result<_, _>>()?,
                ),
                _ => return Err("invalid checkpoint value kind".into()),
            };
            if key == 0 || storage.slots.insert(key, Box::new(value)).is_some() {
                return Err("duplicate/zero checkpoint key".into());
            }
        }
        // Candidate remains private until migration and projection admission pass.
        Ok(Self::from_actor(Actor::mount_checkpoint::<A>(1, storage)?))
    }
    pub fn menu_hover(&mut self, input: &str) -> Result<String, String> {
        use blockui_runtime::interaction::menu_hover::{Branch, Target};
        let input: Value = serde_json::from_str(input).map_err(|_| "invalid menu hover input")?;
        if input["reset"].as_bool() == Some(true) {
            self.menu_hover.reset();
            return Ok(json!({"focus":null,"open":null,"deadline":null}).to_string());
        }
        let now = input["now"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.0 && *v < u64::MAX as f64)
            .ok_or("invalid menu hover time")? as u64;
        fn coordinates<const N: usize>(value: &Value) -> Result<[f64; N], String> {
            let values = value
                .as_array()
                .filter(|values| values.len() == N)
                .ok_or("invalid menu geometry")?;
            let mut result = [0.0; N];
            for (out, value) in result.iter_mut().zip(values) {
                *out = value
                    .as_f64()
                    .filter(|v| v.is_finite())
                    .ok_or("invalid menu coordinate")?;
            }
            Ok(result)
        }
        let point = coordinates(&input["point"])?;
        let view = self.actor.projected_view();
        let target = input["hovered"]
            .as_str()
            .and_then(|key| key.parse::<u64>().ok())
            .and_then(|key| {
                let node = view.bindings.get(&key)?.node;
                (semantics::is_menu_item(node.semantics.role) && node.disabled == 0).then_some(
                    Target {
                        key,
                        opens_submenu: node.semantics.role == semantics::SUBMENU_TRIGGER
                            && node.semantics.expanded != 2,
                    },
                )
            });
        let branch = if let Some(key) = input["branch"]["key"]
            .as_str()
            .and_then(|key| key.parse::<u64>().ok())
        {
            let node = view
                .bindings
                .get(&key)
                .ok_or("missing submenu trigger")?
                .node;
            if node.semantics.role != semantics::SUBMENU_TRIGGER || node.semantics.expanded != 2 {
                return Err("submenu is not open".into());
            }
            Some(Branch {
                trigger: key,
                anchor: coordinates(&input["branch"]["anchor"])?,
                panel: coordinates(&input["branch"]["panel"])?,
            })
        } else {
            None
        };
        let decision = self.menu_hover.observe(point, target, branch, now);
        Ok(json!({"focus":decision.focus.map(|key|key.to_string()),"open":decision.open.map(|key|key.to_string()),"deadline":self.menu_hover.deadline()}).to_string())
    }
    /// Navigate using the current actor projection; only host-owned IDs/search survive.
    pub fn context_origin(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Result<String, String> {
        let origin = blockui_runtime::interaction::context_menu::origin(
            [x, y],
            [width, height],
            [viewport_width, viewport_height],
        )
        .ok_or("invalid context geometry")?;
        Ok(json!(origin).to_string())
    }
    pub fn navigation_focus(&self, focused: &str) -> Result<Option<String>, String> {
        let key = focused
            .parse::<u64>()
            .map_err(|_| "invalid navigation focus key")?;
        Ok(blockui_runtime::interaction::navigation_focus::recover(
            self.actor.projected_view(),
            key,
        )
        .map(|key| key.to_string()))
    }
    pub fn menubar_input(&self, root: &str, command: &str) -> Result<String, String> {
        use blockui_runtime::interaction::menubar::{self, Input, Item};
        let root: u64 = root.parse().map_err(|_| "invalid menubar trigger")?;
        let view = self.actor.projected_view();
        let trigger = view
            .bindings
            .get(&root)
            .ok_or("missing menubar trigger")?
            .node;
        if trigger.semantics.role != semantics::MENU_TRIGGER {
            return Err("invalid menubar trigger role".into());
        }
        let bar = view
            .bindings
            .get(&trigger.parent)
            .ok_or("missing menu container")?
            .node
            .parent;
        if !view
            .bindings
            .get(&bar)
            .is_some_and(|binding| binding.node.semantics.role == semantics::MENUBAR)
        {
            return Err("trigger does not belong to a menubar".into());
        }
        let items: Vec<_> = view
            .order
            .iter()
            .filter_map(|id| {
                let node = view.bindings[id].node;
                (node.semantics.role == semantics::MENU_TRIGGER
                    && view
                        .bindings
                        .get(&node.parent)
                        .is_some_and(|parent| parent.node.parent == bar))
                .then_some(Item {
                    key: *id,
                    enabled: node.disabled == 0,
                })
            })
            .collect();
        let open = items
            .iter()
            .find(|item| view.bindings[&item.key].node.semantics.expanded == 2)
            .map(|item| item.key);
        let input = match command {
            "previous" => Input::Previous,
            "next" => Input::Next,
            "first" => Input::First,
            "last" => Input::Last,
            "hover" => Input::Hover(root),
            _ => return Err("invalid menubar command".into()),
        };
        let decision = menubar::resolve(&items, root, open, input);
        Ok(json!({"focus":decision.focus.map(|key|key.to_string()),"open":decision.open.map(|key|key.to_string())}).to_string())
    }
    pub fn menu_input(
        &mut self,
        panel: &str,
        focused: &str,
        command: &str,
        text: &str,
        now_ms: f64,
    ) -> Result<String, String> {
        use blockui_runtime::interaction::menu::{Item, Movement};
        if !now_ms.is_finite() || now_ms < 0.0 || now_ms >= u64::MAX as f64 {
            return Err("invalid menu timestamp".into());
        }
        let panel: u64 = panel.parse().map_err(|_| "invalid menu key")?;
        let view = self.actor.projected_view();
        self.menus.retain(|key, _| {
            view.bindings.get(key).is_some_and(|binding| {
                binding.node.semantics.role == semantics::MENU_PANEL
                    && binding.node.semantics.hidden == 0
            })
        });
        if !view.bindings.get(&panel).is_some_and(|binding| {
            binding.node.semantics.role == semantics::MENU_PANEL
                && binding.node.semantics.hidden == 0
        }) {
            return Err("menu is not visible".into());
        }
        let items: Vec<_> = view
            .order
            .iter()
            .filter_map(|key| {
                let binding = &view.bindings[key];
                let mut parent = binding.node.parent;
                let owner = loop {
                    let Some(node) = view.bindings.get(&parent).map(|binding| binding.node) else {
                        break None;
                    };
                    if semantics::is_anchored_panel(node.semantics.role) {
                        break Some(parent);
                    }
                    parent = node.parent;
                };
                (owner == Some(panel) && semantics::is_menu_item(binding.node.semantics.role)).then(
                    || Item {
                        key: *key,
                        label: binding.text.clone(),
                        enabled: binding.node.disabled == 0,
                    },
                )
            })
            .collect();
        let controller = self.menus.entry(panel).or_default();
        if let Ok(key) = focused.parse()
            && controller.active() != Some(key)
        {
            controller.focus(&items, key);
        }
        let target = match command {
            "open" => controller.open(&items, false),
            "open-last" => controller.open(&items, true),
            "next" => controller.navigate(&items, Movement::Next),
            "previous" => controller.navigate(&items, Movement::Previous),
            "first" => controller.navigate(&items, Movement::First),
            "last" => controller.navigate(&items, Movement::Last),
            "text" => controller.typeahead(&items, text, now_ms as u64),
            "reconcile" => controller.reconcile(&items),
            _ => return Err("invalid menu command".into()),
        };
        Ok(target.unwrap_or(panel).to_string())
    }
    /// Stable browser host interaction, independent of actor dispatch/bindings.
    pub fn tooltip_input(
        &mut self,
        hovered: &str,
        focused: &str,
        content: &str,
        now_ms: f64,
        command: u32,
    ) -> Result<String, String> {
        use blockui_runtime::interaction::tooltip::{Input, Trigger};
        if !now_ms.is_finite() || now_ms < 0.0 || command > 2 {
            return Err("invalid tooltip input".into());
        }
        let now = Duration::try_from_secs_f64(now_ms / 1000.0)
            .map_err(|_| "invalid tooltip timestamp")?;
        let view = self.actor.projected_view();
        let modal = view
            .order
            .iter()
            .rev()
            .find(|key| view.bindings[key].node.kind == DIALOG)
            .copied();
        let eligible = |hint| blockui_runtime::interaction::tooltip::eligible(view, hint, modal);
        self.tooltip.reconcile(eligible);
        let trigger = |key: &str| {
            key.parse::<u64>().ok().and_then(|key| {
                let hint = view.bindings.get(&key)?.node.semantics.described_by;
                if !eligible(hint) {
                    return None;
                }
                Some(Trigger {
                    key: hint,
                    delay: Duration::from_millis(u64::from(
                        view.bindings[&hint].node.tooltip_delay_ms,
                    )),
                })
            })
        };
        let input = Input {
            hovered: trigger(hovered),
            focused: trigger(focused),
            content: content.parse().ok(),
        };
        match command {
            0 => {
                self.tooltip.observe(input, now);
            }
            1 => {
                self.tooltip.dismiss(input, now);
            }
            _ => {
                self.tooltip.reset();
            }
        }
        Ok(json!({"visible":self.tooltip.visible().map(|key|key.to_string()), "deadline":self.tooltip.deadline().map(|d|d.as_secs_f64()*1000.0)}).to_string())
    }
    fn advance_motion(&mut self, now_ms: f64) -> Result<(), String> {
        if !now_ms.is_finite() || now_ms < 0.0 {
            return Err("invalid frame timestamp".into());
        }
        let now = Duration::try_from_secs_f64(now_ms / 1000.0)
            .map_err(|_| "frame timestamp out of range")?;
        // An event can retarget after the browser captured the current rAF timestamp.
        self.motion_dirty
            .extend(self.motion.advance_to(now.max(self.motion.now()))?);
        Ok(())
    }
    pub fn motion_target(
        &mut self,
        key: &str,
        checked: bool,
        now_ms: f64,
        immediate: bool,
    ) -> Result<f64, String> {
        let node = key.parse::<u64>().map_err(|_| "invalid motion key")?;
        let binding = self
            .actor
            .projected_view()
            .bindings
            .get(&node)
            .ok_or("removed motion widget")?;
        if binding.node.semantics.role != semantics::SWITCH {
            return Err("motion widget is not a switch".into());
        }
        self.advance_motion(now_ms)?;
        let key = MotionKey::new(node, Property::Switch);
        if self.motion.retarget(
            key,
            f64::from(checked),
            if immediate {
                Duration::ZERO
            } else {
                SWITCH_DURATION
            },
            Easing::default(),
        )? {
            self.motion_dirty.insert(key);
        }
        Ok(self.motion.value(key).unwrap())
    }
    pub fn motion_remove(&mut self, key: &str) -> Result<(), String> {
        let key = MotionKey::new(
            key.parse().map_err(|_| "invalid motion key")?,
            Property::Switch,
        );
        self.motion.remove(key);
        self.motion_dirty.remove(&key);
        Ok(())
    }
    pub fn motion_reduced(&mut self, reduced: bool) {
        self.motion_dirty
            .extend(self.motion.set_reduced_motion(reduced));
    }
    pub fn motion_active(&self) -> bool {
        self.motion.is_active() || !self.motion_dirty.is_empty()
    }
    pub fn motion_frame(&mut self, now_ms: f64) -> Result<String, String> {
        self.advance_motion(now_ms)?;
        // Unmounted/retyped widgets cannot keep tracks alive, even if an adapter
        // missed its explicit removal notification.
        self.motion.retain(|key| {
            self.actor
                .projected_view()
                .bindings
                .get(&key.node)
                .is_some_and(|b| b.node.semantics.role == semantics::SWITCH)
        });
        let values: Vec<_> = std::mem::take(&mut self.motion_dirty)
            .into_iter()
            .filter_map(|key| {
                self.motion
                    .value(key)
                    .map(|value| (key.node.to_string(), value))
            })
            .collect();
        serde_json::to_string(&values).map_err(|e| e.to_string())
    }
    pub fn snapshot(&self) -> Result<String, String> {
        let view = self.actor.projected_view();
        let mut nodes = Vec::with_capacity(view.order.len());
        for key in &view.order {
            let binding = &view.bindings[key];
            let n = binding.node;
            let options: Value = if n.kind == SELECT {
                let (_, options) = select::decode_control(&binding.text).map_err(str::to_owned)?;
                options.into_iter().map(|o| json!({"value": o.value.to_string(), "label": o.label, "enabled": o.enabled})).collect()
            } else {
                Value::Null
            };
            let canvas = if n.kind == CANVAS {
                let scene = canvas::CanvasScene::from_wire(&binding.text).map_err(str::to_owned)?;
                json!({"label": scene.label, "circles": scene.circles.iter().map(|c| json!({"x": c.center.x, "y": c.center.y, "radius": c.radius, "fill": c.fill, "stroke": c.stroke, "strokeWidth": c.stroke_width})).collect::<Vec<_>>()})
            } else {
                Value::Null
            };
            nodes.push(json!({
                "overlayPosition": n.overlay_position,
                "tooltipAnchor": n.tooltip_anchor.to_string(),
                "tooltipDelay": n.tooltip_delay_ms,
                "toastPosition": n.overlay_position,
                "toastPaused": n.action_payload != 0 && n.semantics.role == semantics::TOAST_VIEWPORT,
                "gridRows": n.grid_rows, "gridColumns": n.grid_columns, "index": n.action_payload,
                "key": key.to_string(), "parent": n.parent.to_string(), "kind": n.kind,
                "text": binding.text, "label": if n.semantics.role == semantics::CONTEXT_SURFACE { view.bindings.get(&n.semantics.labelled_by).map(|label|label.text.as_str()) } else { binding.semantic_label.as_deref() },
                "disabled": n.disabled != 0, "readonly": n.read_only != 0,
                "semantics": {"menubarTrigger": n.semantics.role == semantics::MENU_TRIGGER && view.bindings.get(&n.parent).and_then(|parent| view.bindings.get(&parent.node.parent)).is_some_and(|bar| bar.node.semantics.role == semantics::MENUBAR), "role": n.semantics.role, "sortDirection": n.semantics.sort_direction, "selected": n.semantics.selected != 0, "current": n.semantics.current, "tabStop": n.semantics.tab_stop != 0, "hidden": n.semantics.hidden != 0, "level": n.semantics.level, "expanded": n.semantics.expanded, "locked": n.semantics.locked != 0, "controls": n.semantics.controls.to_string(), "labelledBy": n.semantics.labelled_by.to_string(), "describedBy": n.semantics.described_by.to_string()},
                "variant": n.button_variant, "gap": n.gap, "padding": n.padding,
                "height": n.height, "contentWidth": n.content_width, "viewport": n.viewport_action != 0, "scroll": n.scroll_action != 0, "width": n.presentation.width,
                "maxWidth": n.presentation.max_width, "fontSize": n.presentation.font_size,
                "fillHeight": n.presentation.fill_height != 0,
                "tone": n.presentation.text_tone, "surface": n.presentation.surface,
                "radius": n.presentation.radius, "checked": self.actor.checkbox_value(*key),
                "value": self.actor.integer_control_value(*key).map(|v| v.to_string()),
                "range": [n.range.min, n.range.max, n.range.step], "options": options,
                "canvas": canvas,
                "evaluations": binding.evaluations, "styleEvaluations": binding.style_evaluations,
            }));
        }
        serde_json::to_string(&nodes).map_err(|e| e.to_string())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn poll(&mut self) -> Result<bool, String> {
        self.actor.poll_browser()
    }
    pub fn event(&mut self, key: &str, kind: &str, value: &str) -> Result<(), String> {
        let key = key.parse().map_err(|_| "invalid widget key")?;
        match kind {
            "toast-activity" => self.actor.observe_toast_activity(
                key,
                match value {
                    "1" => true,
                    "0" => false,
                    _ => return Err("invalid toast activity".into()),
                },
            ),
            "canvas" | "secondary" | "viewport" | "scroll" => {
                let [x, y]: [f32; 2] = serde_json::from_str(value).map_err(|_| "invalid point")?;
                let point = LocalPoint { x, y };
                match kind {
                    "viewport" => self.actor.observe_viewport(key, point),
                    "scroll" => self.actor.observe_scroll(key, point),
                    _ => self.actor.edit_canvas(key, point, kind == "secondary"),
                }
            }
            "click" => self.actor.dispatch_widget(key),
            "input" => self.actor.edit_text(key, value),
            "grid" => self
                .actor
                .navigate_grid(key, value.parse().map_err(|_| "invalid grid index")?),
            "slider" => self
                .actor
                .edit_slider(key, value.parse().map_err(|_| "invalid slider value")?),
            "select" => self
                .actor
                .edit_select(key, value.parse().map_err(|_| "invalid option value")?),
            "submit" => self.actor.editor_command(key, false),
            "cancel" => self.actor.editor_command(key, true),
            "dismiss" => self.actor.dismiss_dialog(key),
            "dismiss-popover" => self.actor.dismiss_popover(key),
            _ => Err("unsupported browser event".into()),
        }
    }
}

/// Export a statically linked application for the browser DOM backend.
/// The consuming Wasm crate depends on `wasm-bindgen`; no plugin loader is involved.
#[macro_export]
macro_rules! export_app {
    ($name:ident, $actor:ty) => {
        #[::wasm_bindgen::prelude::wasm_bindgen]
        pub struct $name($crate::Application);
        #[::wasm_bindgen::prelude::wasm_bindgen]
        impl $name {
            #[::wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new() -> ::std::result::Result<$name, ::wasm_bindgen::JsValue> {
                $crate::Application::mount::<$actor>()
                    .map(Self)
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
            pub fn checkpoint(&self) -> ::std::result::Result<String, ::wasm_bindgen::JsValue> {
                self.0
                    .checkpoint()
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
            pub fn restore(
                checkpoint: &str,
            ) -> ::std::result::Result<$name, ::wasm_bindgen::JsValue> {
                $crate::Application::restore::<$actor>(checkpoint)
                    .map(Self)
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
            pub fn snapshot(&self) -> ::std::result::Result<String, ::wasm_bindgen::JsValue> {
                self.0
                    .snapshot()
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
            pub fn event(
                &mut self,
                key: &str,
                kind: &str,
                value: &str,
            ) -> ::std::result::Result<(), ::wasm_bindgen::JsValue> {
                self.0
                    .event(key, kind, value)
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
            pub fn motion_target(
                &mut self,
                key: &str,
                checked: bool,
                now_ms: f64,
                immediate: bool,
            ) -> ::std::result::Result<f64, ::wasm_bindgen::JsValue> {
                self.0
                    .motion_target(key, checked, now_ms, immediate)
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
            pub fn motion_remove(
                &mut self,
                key: &str,
            ) -> ::std::result::Result<(), ::wasm_bindgen::JsValue> {
                self.0
                    .motion_remove(key)
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
            pub fn motion_reduced(&mut self, reduced: bool) {
                self.0.motion_reduced(reduced);
            }
            pub fn menu_hover(
                &mut self,
                input: &str,
            ) -> ::std::result::Result<String, ::wasm_bindgen::JsValue> {
                self.0
                    .menu_hover(input)
                    .map_err(|error| ::wasm_bindgen::JsValue::from_str(&error))
            }
            pub fn context_origin(
                &self,
                x: f64,
                y: f64,
                width: f64,
                height: f64,
                viewport_width: f64,
                viewport_height: f64,
            ) -> ::std::result::Result<String, ::wasm_bindgen::JsValue> {
                self.0
                    .context_origin(x, y, width, height, viewport_width, viewport_height)
                    .map_err(|error| ::wasm_bindgen::JsValue::from_str(&error))
            }
            pub fn navigation_focus(
                &self,
                focused: &str,
            ) -> ::std::result::Result<Option<String>, ::wasm_bindgen::JsValue> {
                self.0
                    .navigation_focus(focused)
                    .map_err(|error| ::wasm_bindgen::JsValue::from_str(&error))
            }
            pub fn menubar_input(
                &self,
                root: &str,
                command: &str,
            ) -> ::std::result::Result<String, ::wasm_bindgen::JsValue> {
                self.0
                    .menubar_input(root, command)
                    .map_err(|error| ::wasm_bindgen::JsValue::from_str(&error))
            }
            pub fn menu_input(
                &mut self,
                panel: &str,
                focused: &str,
                command: &str,
                text: &str,
                now_ms: f64,
            ) -> ::std::result::Result<String, ::wasm_bindgen::JsValue> {
                self.0
                    .menu_input(panel, focused, command, text, now_ms)
                    .map_err(|error| ::wasm_bindgen::JsValue::from_str(&error))
            }
            pub fn tooltip_input(
                &mut self,
                hovered: &str,
                focused: &str,
                content: &str,
                now_ms: f64,
                command: u32,
            ) -> ::std::result::Result<String, ::wasm_bindgen::JsValue> {
                self.0
                    .tooltip_input(hovered, focused, content, now_ms, command)
                    .map_err(|error| ::wasm_bindgen::JsValue::from_str(&error))
            }
            pub fn motion_active(&self) -> bool {
                self.0.motion_active()
            }
            pub fn motion_frame(
                &mut self,
                now_ms: f64,
            ) -> ::std::result::Result<String, ::wasm_bindgen::JsValue> {
                self.0
                    .motion_frame(now_ms)
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
            #[cfg(target_arch = "wasm32")]
            pub fn poll(&mut self) -> ::std::result::Result<bool, ::wasm_bindgen::JsValue> {
                self.0
                    .poll()
                    .map_err(|e| ::wasm_bindgen::JsValue::from_str(&e))
            }
        }
    };
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    use blockui::{Cx, IntoView, State, StateSpec, Switch, View};
    const ON: State<bool> = State::<bool>::new("motion.on", false);
    struct Controls;
    impl blockui::Actor for Controls {
        const STATE: &'static [StateSpec] = &[ON.spec()];
        const ACTIONS: &'static [blockui::Action] = &[];
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(blockui::Column::default().children(vec![
                Switch::new("Airplane mode", ON, |cx: &Cx<'_>, checked| {
                    ON.set(cx, checked)
                })
                .into_view()
                .key("switch"),
            ]))
        }
    }
    #[test]
    fn bridge_samples_without_actor_updates_and_finishes_exactly() -> Result<(), String> {
        let mut app = Application::mount::<Controls>()?;
        let key = blockui_abi::key("switch").to_string();
        assert_eq!(app.motion_target(&key, false, 0.0, false)?, 0.0);
        app.motion_frame(0.0)?;
        assert!(!app.motion_active());
        app.event(&key, "click", "")?;
        assert_eq!(app.motion_target(&key, true, 0.0, false)?, 0.0);
        let snapshot = app.snapshot()?;
        let sample: Vec<(String, f64)> = serde_json::from_str(&app.motion_frame(80.0)?).unwrap();
        assert_eq!(sample, [(key.clone(), 0.875)]);
        assert_eq!(app.snapshot()?, snapshot);
        // A second patch must not restart the same target.
        assert_eq!(app.motion_target(&key, true, 80.0, false)?, 0.875);
        app.motion_frame(160.0)?;
        assert!(!app.motion_active());
        app.event(&key, "click", "")?;
        app.motion_target(&key, false, 160.0, false)?;
        app.motion_frame(200.0)?;
        app.motion_reduced(true);
        let sample: Vec<(String, f64)> = serde_json::from_str(&app.motion_frame(200.0)?).unwrap();
        assert_eq!(sample, [(key.clone(), 0.0)]);
        assert!(!app.motion_active());
        app.motion_remove(&key)?;
        assert!(app.motion.is_empty());
        assert!(app.motion_frame(f64::NAN).is_err());
        Ok(())
    }
    #[test]
    fn delayed_frame_preserves_other_tracks_and_new_mounts_are_isolated() -> Result<(), String> {
        let mut a = Application::mount::<Controls>()?;
        let b = Application::mount::<Controls>()?;
        let key = blockui_abi::key("switch").to_string();
        a.motion_target(&key, false, 0.0, false)?;
        a.motion_target(&key, true, 0.0, false)?;
        // Finishing while retargeting must still publish the final sample.
        a.motion_target(&key, true, 500.0, false)?;
        let sample: Vec<(String, f64)> = serde_json::from_str(&a.motion_frame(490.0)?).unwrap();
        assert_eq!(sample, [(key, 1.0)]);
        assert!(!a.motion_active());
        assert!(!b.motion_active());
        assert!(a.motion_target("999", true, 600.0, false).is_err());
        Ok(())
    }
}

#[cfg(test)]
mod checkpoint_tests {
    use super::Application;
    use blockui::*;
    use blockui_abi::key;
    use std::time::Duration;
    const COUNT: State<i64> = State::<i64>::new("checkpoint.count", 9_007_199_254_740_993);
    const FLOAT: State<f64> = State::<f64>::new("checkpoint.float", -0.0);
    const BOOL: State<bool> = State::<bool>::new("checkpoint.bool", true);
    const TEXT: State<String> = State::<String>::new("checkpoint.text");
    const INTS: State<Vec<i64>> = State::<Vec<i64>>::new("checkpoint.ints");
    const TEXTS: State<Vec<String>> = State::<Vec<String>>::new("checkpoint.texts");
    const EXTRA: State<bool> = State::<bool>::new("checkpoint.extra", false);
    const CHANGE: Action = Action::new("checkpoint.change", |cx, _| {
        COUNT.update(cx, |n| Ok(n + 1))?;
        TEXT.set(cx, "中文 🦀")?;
        INTS.set(cx, &[i64::MIN, i64::MAX])?;
        TEXTS.set(cx, &["中文".into(), "🦀".into()])
    });
    struct Original;
    impl blockui::Actor for Original {
        const STATE: &'static [StateSpec] = &[
            COUNT.spec(),
            FLOAT.spec(),
            BOOL.spec(),
            TEXT.spec(),
            INTS.spec(),
            TEXTS.spec(),
        ];
        const ACTIONS: &'static [Action] = &[CHANGE];
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(Column::default().children(vec![
                Button::new("Change", CHANGE).key("change").into_view(),
            ]))
        }
    }
    struct Rejected;
    impl blockui::Actor for Rejected {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), EXTRA.spec()];
        const ACTIONS: &'static [Action] = Original::ACTIONS;
        fn view(cx: &Cx<'_>) -> blockui::Result<View> {
            Original::view(cx)
        }
        fn migrate(cx: &Cx<'_>, _: u64) -> blockui::Result<()> {
            COUNT.set(cx, 0)?;
            Err(Error::Application("rejected checkpoint migration".into()))
        }
    }
    #[test]
    fn lossless_roundtrip_and_rejected_migration_preserve_running_actor()
    -> std::result::Result<(), String> {
        let mut app = Application::mount::<Original>()?;
        app.event(&key("change").to_string(), "click", "")?;
        let saved = app.checkpoint()?;
        let restored = Application::restore::<Original>(&saved)?;
        assert_eq!(restored.checkpoint()?, saved);
        let state = restored.actor.checkpoint()?;
        assert!(
            matches!(&**state.slots.get(&key("checkpoint.float")).unwrap(), blockui_runtime::Value::Float(v) if v.to_bits() == (-0.0f64).to_bits())
        );
        assert!(
            Application::restore::<Rejected>(&saved)
                .err()
                .unwrap()
                .contains("rejected checkpoint migration")
        );
        assert_eq!(app.checkpoint()?, saved);
        app.event(&key("change").to_string(), "click", "")?;
        assert_ne!(app.checkpoint()?, saved);
        Ok(())
    }
    const TIMER: Task = Task::new("checkpoint.timer");
    const START: Action = Action::new("checkpoint.start", |cx, _| {
        TIMER.interval(cx, Duration::from_secs(60), CHANGE, 0)
    });
    const STOP: Action = Action::new("checkpoint.stop", |cx, _| TIMER.cancel(cx));
    struct Clock;
    impl blockui::Actor for Clock {
        const STATE: &'static [StateSpec] = Original::STATE;
        const ACTIONS: &'static [Action] = &[CHANGE, START, STOP];
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(Column::default().children(vec![
                Button::new("Start", START).key("start").into_view(),
                Button::new("Stop", STOP).key("stop").into_view(),
            ]))
        }
    }
    #[test]
    fn active_interval_blocks_checkpoint_until_canceled() -> std::result::Result<(), String> {
        let mut app = Application::mount::<Clock>()?;
        app.event(&key("start").to_string(), "click", "")?;
        assert!(
            app.checkpoint()
                .unwrap_err()
                .contains("Web hot reload waits")
        );
        app.event(&key("stop").to_string(), "click", "")?;
        Application::restore::<Clock>(&app.checkpoint()?)?;
        Ok(())
    }
    #[test]
    fn invalid_checkpoint_is_rejected() {
        for input in [
            "{}",
            r#"{"format":2,"slots":[]}"#,
            r#"{"format":1,"slots":[{"key":"0","kind":"integer","value":"1"}]}"#,
            r#"{"format":1,"slots":[{"key":"1","kind":"boolean","value":true},{"key":"1","kind":"boolean","value":false}]}"#,
            r#"{"format":1,"slots":[{"key":"1","kind":"integer","value":9007199254740993}]}"#,
        ] {
            assert!(Application::restore::<Original>(input).is_err(), "{input}");
        }
    }
}
