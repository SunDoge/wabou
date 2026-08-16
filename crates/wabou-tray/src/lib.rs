//! Optional system-tray integration for Wabou applications.
//!
//! This crate contains the GTK/AppIndicator dependency on Linux so applications
//! that do not use a tray icon do not pay for it.
//!
//! Linux builds require GTK 3 and an AppIndicator implementation at build and
//! runtime. Wabou hosts the GTK loop on a dedicated thread so it can coexist
//! with winit on both Wayland and X11.

#![warn(missing_docs)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use tray_icon::menu::{
    CheckMenuItem, ContextMenu, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use wabou_shell::{ExtensionContext, ShellExtension, WakeCallback};

type Action = Box<dyn FnMut(&mut ExtensionContext<'_>)>;

enum PendingEvent {
    Item(String),
    #[cfg(target_os = "linux")]
    Dismiss(wabou_shell::EffectId),
}

#[cfg(target_os = "linux")]
enum GtkCommand {
    Static,
    Effect {
        id: wabou_shell::EffectId,
        items: Vec<wabou_shell::ContextMenuItem>,
    },
}

struct Item {
    id: String,
    label: String,
    action: Action,
}

/// RGBA pixels used for a native tray icon.
#[derive(Clone, Debug)]
pub struct TrayImage {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl TrayImage {
    /// Validate and retain a tightly packed, non-empty RGBA8 image.
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, String> {
        let expected = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .map(|bytes| bytes as usize)
            .ok_or_else(|| "tray image dimensions overflow".to_string())?;
        if width == 0 || height == 0 || rgba.len() != expected {
            return Err(format!(
                "tray image needs {expected} RGBA bytes for {width}x{height}, got {}",
                rgba.len()
            ));
        }
        Icon::from_rgba(rgba.clone(), width, height).map_err(|error| error.to_string())?;
        Ok(Self {
            rgba,
            width,
            height,
        })
    }

    fn to_icon(&self) -> Result<Icon, String> {
        Icon::from_rgba(self.rgba.clone(), self.width, self.height)
            .map_err(|error| error.to_string())
    }
}

/// A native system tray icon and its right-click menu.
pub struct SystemTray {
    tooltip: Option<String>,
    icon: TrayImage,
    items: Vec<Item>,
    separators: Vec<usize>,
    context_items: Vec<Item>,
    context_separators: Vec<usize>,
    pending: Arc<Mutex<VecDeque<PendingEvent>>>,
    effect_routes: HashMap<String, (wabou_shell::EffectId, u64, wabou_shell::EffectOp, String)>,
    active_effects: HashSet<wabou_shell::EffectId>,
    #[cfg(not(target_os = "linux"))]
    native: Option<TrayIcon>,
    #[cfg(not(target_os = "linux"))]
    native_context_menu: Option<Menu>,
    #[cfg(target_os = "linux")]
    context_sender: Option<gtk::glib::Sender<GtkCommand>>,
    hide_window_on_close: Option<u64>,
}

impl SystemTray {
    /// Construct a tray extension with no tooltip or menu items.
    pub fn new(icon: TrayImage) -> Self {
        Self {
            tooltip: None,
            icon,
            items: Vec::new(),
            separators: Vec::new(),
            context_items: Vec::new(),
            context_separators: Vec::new(),
            pending: Arc::new(Mutex::new(VecDeque::new())),
            effect_routes: HashMap::new(),
            active_effects: HashSet::new(),
            #[cfg(not(target_os = "linux"))]
            native: None,
            #[cfg(not(target_os = "linux"))]
            native_context_menu: None,
            #[cfg(target_os = "linux")]
            context_sender: None,
            hide_window_on_close: None,
        }
    }

    /// Set the native tray tooltip.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Add an activatable item to the tray's primary menu.
    pub fn item(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        action: impl FnMut(&mut ExtensionContext<'_>) + 'static,
    ) -> Self {
        self.items.push(Item {
            id: id.into(),
            label: label.into(),
            action: Box::new(action),
        });
        self
    }

    /// Add a separator after the primary items currently registered.
    pub fn separator(mut self) -> Self {
        self.separators.push(self.items.len());
        self
    }

    /// Add an item to the native menu shown for a secondary pointer click.
    pub fn context_item(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        action: impl FnMut(&mut ExtensionContext<'_>) + 'static,
    ) -> Self {
        self.context_items.push(Item {
            id: id.into(),
            label: label.into(),
            action: Box::new(action),
        });
        self
    }

    /// Add a separator after the context items currently registered.
    pub fn context_separator(mut self) -> Self {
        self.context_separators.push(self.context_items.len());
        self
    }

    /// Keep the process alive and hide this window when its close button is used.
    pub fn hide_window_on_close(mut self, logical_window_id: u64) -> Self {
        self.hide_window_on_close = Some(logical_window_id);
        self
    }

    fn build_menu(items: Vec<(String, String)>, separators: Vec<usize>) -> Result<Menu, String> {
        let menu = Menu::new();
        for position in 0..=items.len() {
            if separators.contains(&position) {
                menu.append(&PredefinedMenuItem::separator())
                    .map_err(|error| error.to_string())?;
            }
            if let Some((id, label)) = items.get(position) {
                menu.append(&MenuItem::with_id(
                    MenuId::new(id.clone()),
                    label,
                    true,
                    None,
                ))
                .map_err(|error| error.to_string())?;
            }
        }
        Ok(menu)
    }

    fn build_effect_menu(items: &[wabou_shell::ContextMenuItem]) -> Result<Menu, String> {
        fn build_items(
            items: &[wabou_shell::ContextMenuItem],
        ) -> Result<Vec<Box<dyn tray_icon::menu::IsMenuItem>>, String> {
            let mut native_items: Vec<Box<dyn tray_icon::menu::IsMenuItem>> = Vec::new();
            for item in items {
                match item {
                    wabou_shell::ContextMenuItem::Item {
                        id,
                        label,
                        enabled,
                        checked,
                    } => {
                        let item: Box<dyn tray_icon::menu::IsMenuItem> = if *checked {
                            Box::new(CheckMenuItem::with_id(
                                MenuId::new(id.clone()),
                                label,
                                *enabled,
                                true,
                                None,
                            ))
                        } else {
                            Box::new(MenuItem::with_id(
                                MenuId::new(id.clone()),
                                label,
                                *enabled,
                                None,
                            ))
                        };
                        native_items.push(item);
                    }
                    wabou_shell::ContextMenuItem::Separator => {
                        native_items.push(Box::new(PredefinedMenuItem::separator()));
                    }
                    wabou_shell::ContextMenuItem::Submenu { label, items } => {
                        let children = build_items(items)?;
                        let refs = children.iter().map(Box::as_ref).collect::<Vec<_>>();
                        native_items.push(Box::new(
                            Submenu::with_items(label, true, &refs)
                                .map_err(|error| error.to_string())?,
                        ));
                    }
                }
            }
            Ok(native_items)
        }

        let items = build_items(items)?;
        let refs = items.iter().map(Box::as_ref).collect::<Vec<_>>();
        Menu::with_items(&refs).map_err(|error| error.to_string())
    }

    fn route_effect_items(
        effect_id: wabou_shell::EffectId,
        window_id: u64,
        op: wabou_shell::EffectOp,
        items: &[wabou_shell::ContextMenuItem],
        routes: &mut HashMap<String, (wabou_shell::EffectId, u64, wabou_shell::EffectOp, String)>,
        next: &mut usize,
    ) -> Vec<wabou_shell::ContextMenuItem> {
        items
            .iter()
            .map(|item| match item {
                wabou_shell::ContextMenuItem::Item {
                    id,
                    label,
                    enabled,
                    checked,
                } => {
                    let native_id = format!("wabou.effect.{}.{}", effect_id.0, *next);
                    *next += 1;
                    routes.insert(native_id.clone(), (effect_id, window_id, op, id.clone()));
                    wabou_shell::ContextMenuItem::Item {
                        id: native_id,
                        label: label.clone(),
                        enabled: *enabled,
                        checked: *checked,
                    }
                }
                wabou_shell::ContextMenuItem::Separator => wabou_shell::ContextMenuItem::Separator,
                wabou_shell::ContextMenuItem::Submenu { label, items } => {
                    wabou_shell::ContextMenuItem::Submenu {
                        label: label.clone(),
                        items: Self::route_effect_items(
                            effect_id, window_id, op, items, routes, next,
                        ),
                    }
                }
            })
            .collect()
    }

    fn validate_item_ids(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for item in self.items.iter().chain(&self.context_items) {
            if !ids.insert(&item.id) {
                return Err(format!("duplicate native menu item id `{}`", item.id));
            }
        }
        Ok(())
    }

    fn build_native(
        icon: TrayImage,
        tooltip: Option<String>,
        items: Vec<(String, String)>,
        separators: Vec<usize>,
        pending: Arc<Mutex<VecDeque<PendingEvent>>>,
        wake: WakeCallback,
    ) -> Result<TrayIcon, String> {
        let menu = Self::build_menu(items, separators)?;

        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            pending
                .lock()
                .expect("tray event queue mutex poisoned")
                .push_back(PendingEvent::Item(event.id().0.clone()));
            wake();
        }));

        let mut builder = TrayIconBuilder::new()
            .with_icon(icon.to_icon()?)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true);
        if let Some(tooltip) = tooltip {
            builder = builder.with_tooltip(tooltip);
        }
        builder.build().map_err(|error| error.to_string())
    }
}

impl ShellExtension for SystemTray {
    fn initialize(&mut self, wake: WakeCallback) -> Result<(), String> {
        self.validate_item_ids()?;
        let items = self
            .items
            .iter()
            .map(|item| (item.id.clone(), item.label.clone()))
            .collect::<Vec<_>>();
        let context_items = self
            .context_items
            .iter()
            .map(|item| (item.id.clone(), item.label.clone()))
            .collect::<Vec<_>>();

        #[cfg(target_os = "linux")]
        {
            let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
            let icon = self.icon.clone();
            let tooltip = self.tooltip.clone();
            let separators = self.separators.clone();
            let context_separators = self.context_separators.clone();
            let pending = self.pending.clone();
            let menu_pending = pending.clone();
            let menu_wake = wake.clone();
            #[allow(deprecated)] // glib 0.18's channel is the GTK-loop wake primitive.
            let (context_sender, context_receiver) =
                gtk::glib::MainContext::channel(gtk::glib::Priority::DEFAULT);
            self.context_sender = Some(context_sender);
            std::thread::Builder::new()
                .name("wabou-tray-gtk".into())
                .spawn(move || {
                    let result = gtk::init()
                        .map_err(|error| error.to_string())
                        .and_then(|()| {
                            let native = Self::build_native(
                                icon, tooltip, items, separators, pending, wake,
                            )?;
                            let context_menu = Self::build_menu(context_items, context_separators)?;
                            let anchor = gtk::Window::new(gtk::WindowType::Popup);
                            context_receiver.attach(None, move |command| {
                                match command {
                                    GtkCommand::Static => {
                                        context_menu
                                            .show_context_menu_for_gtk_window(&anchor, None);
                                    }
                                    GtkCommand::Effect { id, items } => {
                                        if let Ok(menu) = Self::build_effect_menu(&items) {
                                            menu.show_context_menu_for_gtk_window(&anchor, None);
                                        } else {
                                            menu_pending
                                                .lock()
                                                .expect("tray event queue mutex poisoned")
                                                .push_back(PendingEvent::Dismiss(id));
                                            menu_wake();
                                        }
                                    }
                                }
                                gtk::glib::ControlFlow::Continue
                            });
                            Ok(native)
                        });
                    match result {
                        Ok(_native) => {
                            let _ = ready_tx.send(Ok(()));
                            gtk::main();
                        }
                        Err(error) => {
                            let _ = ready_tx.send(Err(error));
                        }
                    }
                })
                .map_err(|error| error.to_string())?;
            ready_rx.recv().map_err(|error| error.to_string())?
        }

        #[cfg(not(target_os = "linux"))]
        {
            self.native = Some(Self::build_native(
                self.icon.clone(),
                self.tooltip.clone(),
                items,
                self.separators.clone(),
                self.pending.clone(),
                wake,
            )?);
            self.native_context_menu = Some(Self::build_menu(
                context_items,
                self.context_separators.clone(),
            )?);
            Ok(())
        }
    }

    fn poll(&mut self, context: &mut ExtensionContext<'_>) {
        loop {
            let id = self
                .pending
                .lock()
                .expect("tray event queue mutex poisoned")
                .pop_front();
            let Some(event) = id else { break };
            match event {
                PendingEvent::Item(id) => {
                    if let Some((effect_id, window_id, op, selection)) =
                        self.effect_routes.get(&id).cloned()
                    {
                        self.active_effects.remove(&effect_id);
                        self.effect_routes.retain(|_, route| route.0 != effect_id);
                        context.complete_effect(
                            window_id,
                            wabou_shell::EffectCompletion {
                                id: effect_id,
                                op,
                                result: wabou_shell::EffectResult::ContextMenuSelection(Some(
                                    selection,
                                )),
                            },
                        );
                    } else if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
                        (item.action)(context);
                    } else if let Some(item) =
                        self.context_items.iter_mut().find(|item| item.id == id)
                    {
                        (item.action)(context);
                    }
                }
                #[cfg(target_os = "linux")]
                PendingEvent::Dismiss(effect_id) => {
                    if self.active_effects.remove(&effect_id)
                        && let Some((_, window_id, op, _)) = self
                            .effect_routes
                            .values()
                            .find(|route| route.0 == effect_id)
                            .cloned()
                    {
                        context.complete_effect(
                            window_id,
                            wabou_shell::EffectCompletion {
                                id: effect_id,
                                op,
                                result: wabou_shell::EffectResult::Error {
                                    code: wabou_shell::EffectErrorCode::PlatformFailure,
                                    message: "failed to create native context menu".into(),
                                },
                            },
                        );
                    }
                    self.effect_routes.retain(|_, route| route.0 != effect_id);
                }
            }
        }
    }

    fn close_requested(
        &mut self,
        logical_window_id: u64,
        context: &mut ExtensionContext<'_>,
    ) -> bool {
        self.hide_window_on_close == Some(logical_window_id)
            && context.hide_window(logical_window_id)
    }

    fn pointer_button(
        &mut self,
        logical_window_id: u64,
        button: wabou_shell::PointerButton,
        phase: wabou_shell::PointerPhase,
        position: wabou_shell::Point,
        context: &mut ExtensionContext<'_>,
    ) -> bool {
        if self.context_items.is_empty()
            || button != wabou_shell::PointerButton::Secondary
            || phase != wabou_shell::PointerPhase::Down
        {
            return false;
        }

        #[cfg(target_os = "linux")]
        {
            let _ = (logical_window_id, position, context);
            return self
                .context_sender
                .as_ref()
                .is_some_and(|sender| sender.send(GtkCommand::Static).is_ok());
        }

        #[cfg(target_os = "windows")]
        if let (Some(menu), Some(wabou_shell::raw_window_handle::RawWindowHandle::Win32(handle))) = (
            &self.native_context_menu,
            context.window_handle(logical_window_id),
        ) {
            let scale = context
                .window_scale_factor(logical_window_id)
                .unwrap_or(1.0);
            let position =
                tray_icon::menu::dpi::PhysicalPosition::new(position.x * scale, position.y * scale);
            return unsafe {
                menu.show_context_menu_for_hwnd(handle.hwnd.get(), Some(position.into()))
            };
        }

        #[cfg(target_os = "macos")]
        if let (Some(menu), Some(wabou_shell::raw_window_handle::RawWindowHandle::AppKit(handle))) = (
            &self.native_context_menu,
            context.window_handle(logical_window_id),
        ) {
            let position = tray_icon::menu::dpi::LogicalPosition::new(position.x, position.y);
            return unsafe {
                menu.show_context_menu_for_nsview(handle.ns_view.as_ptr(), Some(position.into()))
            };
        }

        #[allow(unreachable_code)]
        false
    }

    fn submit_effect(
        &mut self,
        request: &wabou_shell::EffectRequest,
        context: &mut ExtensionContext<'_>,
    ) -> bool {
        let wabou_shell::EffectPayload::ContextMenuShow(menu_request) = &request.payload else {
            return false;
        };
        let window_id = match request.scope {
            wabou_shell::EffectScope::Window(id) => id,
            wabou_shell::EffectScope::Runtime(_) => menu_request.window_id,
        };
        if menu_request.items.is_empty() {
            context.complete_effect(
                window_id,
                wabou_shell::EffectCompletion {
                    id: request.id,
                    op: request.payload.op(),
                    result: wabou_shell::EffectResult::Error {
                        code: wabou_shell::EffectErrorCode::InvalidRequest,
                        message: "a native context menu needs at least one item".into(),
                    },
                },
            );
            return true;
        }
        let mut next = 0;
        let items = Self::route_effect_items(
            request.id,
            window_id,
            request.payload.op(),
            &menu_request.items,
            &mut self.effect_routes,
            &mut next,
        );
        self.active_effects.insert(request.id);

        #[cfg(target_os = "linux")]
        {
            let _ = context;
            let sent = self.context_sender.as_ref().is_some_and(|sender| {
                sender
                    .send(GtkCommand::Effect {
                        id: request.id,
                        items,
                    })
                    .is_ok()
            });
            if !sent {
                self.active_effects.remove(&request.id);
                self.effect_routes.retain(|_, route| route.0 != request.id);
            }
            sent
        }

        #[cfg(not(target_os = "linux"))]
        {
            let Ok(menu) = Self::build_effect_menu(&items) else {
                self.active_effects.remove(&request.id);
                self.effect_routes.retain(|_, route| route.0 != request.id);
                return false;
            };

            #[cfg(target_os = "windows")]
            if let Some(wabou_shell::raw_window_handle::RawWindowHandle::Win32(handle)) =
                context.window_handle(window_id)
            {
                let scale = context.window_scale_factor(window_id).unwrap_or(1.0);
                let position = menu_request.position.as_ref().map(|position| {
                    tray_icon::menu::dpi::PhysicalPosition::new(
                        f64::from(position.x) * scale,
                        f64::from(position.y) * scale,
                    )
                    .into()
                });
                let shown = unsafe { menu.show_context_menu_for_hwnd(handle.hwnd.get(), position) };
                if !shown {
                    self.active_effects.remove(&request.id);
                    self.effect_routes.retain(|_, route| route.0 != request.id);
                }
                return shown;
            }

            #[cfg(target_os = "macos")]
            if let Some(wabou_shell::raw_window_handle::RawWindowHandle::AppKit(handle)) =
                context.window_handle(window_id)
            {
                let position = menu_request.position.as_ref().map(|position| {
                    tray_icon::menu::dpi::LogicalPosition::new(position.x, position.y).into()
                });
                let shown =
                    unsafe { menu.show_context_menu_for_nsview(handle.ns_view.as_ptr(), position) };
                if !shown {
                    self.active_effects.remove(&request.id);
                    self.effect_routes.retain(|_, route| route.0 != request.id);
                }
                return shown;
            }

            self.active_effects.remove(&request.id);
            self.effect_routes.retain(|_, route| route.0 != request.id);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_invalid_rgba_buffer() {
        assert!(TrayImage::from_rgba(vec![0; 3], 1, 1).is_err());
    }

    #[test]
    fn rejects_ids_shared_by_tray_and_context_menus() {
        let icon = TrayImage::from_rgba(vec![255; 4], 1, 1).unwrap();
        let tray = SystemTray::new(icon)
            .item("duplicate", "Tray", |_| {})
            .context_item("duplicate", "Context", |_| {});
        assert_eq!(
            tray.validate_item_ids().unwrap_err(),
            "duplicate native menu item id `duplicate`"
        );
    }
}
