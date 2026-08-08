//! Optional system-tray integration for Wabou applications.
//!
//! This crate contains the GTK/AppIndicator dependency on Linux so applications
//! that do not use a tray icon do not pay for it.
//!
//! Linux builds require GTK 3 and an AppIndicator implementation at build and
//! runtime. Wabou hosts the GTK loop on a dedicated thread so it can coexist
//! with winit on both Wayland and X11.

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use tray_icon::menu::{ContextMenu, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use wabou_shell::{ExtensionContext, ShellExtension, WakeCallback};

type Action = Box<dyn FnMut(&mut ExtensionContext<'_>)>;

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
    pending: Arc<Mutex<VecDeque<String>>>,
    #[cfg(not(target_os = "linux"))]
    native: Option<TrayIcon>,
    #[cfg(not(target_os = "linux"))]
    native_context_menu: Option<Menu>,
    #[cfg(target_os = "linux")]
    context_sender: Option<gtk::glib::Sender<()>>,
    hide_window_on_close: Option<u64>,
}

impl SystemTray {
    pub fn new(icon: TrayImage) -> Self {
        Self {
            tooltip: None,
            icon,
            items: Vec::new(),
            separators: Vec::new(),
            context_items: Vec::new(),
            context_separators: Vec::new(),
            pending: Arc::new(Mutex::new(VecDeque::new())),
            #[cfg(not(target_os = "linux"))]
            native: None,
            #[cfg(not(target_os = "linux"))]
            native_context_menu: None,
            #[cfg(target_os = "linux")]
            context_sender: None,
            hide_window_on_close: None,
        }
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

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
        pending: Arc<Mutex<VecDeque<String>>>,
        wake: WakeCallback,
    ) -> Result<TrayIcon, String> {
        let menu = Self::build_menu(items, separators)?;

        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            pending
                .lock()
                .expect("tray event queue mutex poisoned")
                .push_back(event.id().0.clone());
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
                            context_receiver.attach(None, move |()| {
                                context_menu.show_context_menu_for_gtk_window(&anchor, None);
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
            let Some(id) = id else { break };
            if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
                (item.action)(context);
            } else if let Some(item) = self.context_items.iter_mut().find(|item| item.id == id) {
                (item.action)(context);
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
                .is_some_and(|sender| sender.send(()).is_ok());
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
