//! GPUI-native system tray integration for Wabou applications.

#![warn(missing_docs)]

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[cfg(not(target_os = "linux"))]
use tray_icon::TrayIcon;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use wabou_shell_gpui::gpui::{AnyWindowHandle, App, AsyncApp, Task};
use wabou_shell_gpui::{ApplicationExtension, ApplicationExtensionContext};

type Action = Box<dyn FnMut(&mut TrayContext<'_>)>;

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
            .ok_or_else(|| "tray image dimensions overflow".to_owned())?;
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

/// Operations exposed to one tray menu callback.
pub struct TrayContext<'a> {
    app: &'a mut AsyncApp,
    primary_window: Option<AnyWindowHandle>,
}

impl TrayContext<'_> {
    /// Activate the application and raise its primary window.
    pub fn show_primary_window(&mut self) {
        let primary_window = self.primary_window;
        self.app.update(move |app| {
            app.activate(true);
            if let Some(window) = primary_window {
                let _ = window.update(app, |_, window, _| window.activate_window());
            }
        });
    }

    /// Hide the application without destroying its windows.
    pub fn hide_application(&mut self) {
        self.app.update(|app| app.hide());
    }

    /// Quit through GPUI's platform lifecycle.
    pub fn quit(&mut self) {
        self.app.update(|app| app.quit());
    }
}

/// A native tray icon installed into the GPUI application lifecycle.
pub struct SystemTray {
    tooltip: Option<String>,
    icon: TrayImage,
    items: Rc<RefCell<Vec<Item>>>,
    separators: Vec<usize>,
    hide_on_close: bool,
    #[cfg(not(target_os = "linux"))]
    native: Option<TrayIcon>,
    task: Option<Task<()>>,
}

impl SystemTray {
    /// Construct a tray extension with no tooltip or menu items.
    pub fn new(icon: TrayImage) -> Self {
        Self {
            tooltip: None,
            icon,
            items: Rc::new(RefCell::new(Vec::new())),
            separators: Vec::new(),
            hide_on_close: false,
            #[cfg(not(target_os = "linux"))]
            native: None,
            task: None,
        }
    }

    /// Set the native tray tooltip.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Add an activatable item to the tray menu.
    pub fn item(
        self,
        id: impl Into<String>,
        label: impl Into<String>,
        action: impl FnMut(&mut TrayContext<'_>) + 'static,
    ) -> Self {
        self.items.borrow_mut().push(Item {
            id: id.into(),
            label: label.into(),
            action: Box::new(action),
        });
        self
    }

    /// Add a separator after the items currently registered.
    pub fn separator(mut self) -> Self {
        self.separators.push(self.items.borrow().len());
        self
    }

    /// Hide the primary window's application when the native close button is used.
    pub fn hide_window_on_close(mut self) -> Self {
        self.hide_on_close = true;
        self
    }

    fn validate_item_ids(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for item in self.items.borrow().iter() {
            if !ids.insert(&item.id) {
                return Err(format!("duplicate native menu item id `{}`", item.id));
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn build_menu(&self) -> Result<Menu, String> {
        let menu = Menu::new();
        let items = self.items.borrow();
        for position in 0..=items.len() {
            if self.separators.contains(&position) {
                menu.append(&PredefinedMenuItem::separator())
                    .map_err(|error| error.to_string())?;
            }
            if let Some(item) = items.get(position) {
                menu.append(&MenuItem::with_id(
                    MenuId::new(item.id.clone()),
                    &item.label,
                    true,
                    None,
                ))
                .map_err(|error| error.to_string())?;
            }
        }
        Ok(menu)
    }

    #[cfg(not(target_os = "linux"))]
    fn build_native(&self, sender: flume::Sender<String>) -> Result<TrayIcon, String> {
        let menu = self.build_menu()?;
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let _ = sender.send(event.id().0.clone());
        }));
        let mut builder = TrayIconBuilder::new()
            .with_icon(self.icon.to_icon()?)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true);
        if let Some(tooltip) = &self.tooltip {
            builder = builder.with_tooltip(tooltip);
        }
        builder.build().map_err(|error| error.to_string())
    }
}

impl ApplicationExtension for SystemTray {
    fn install(
        &mut self,
        context: ApplicationExtensionContext<'_>,
        app: &mut App,
    ) -> Result<(), String> {
        self.validate_item_ids()?;
        let primary_window = context.primary_window();
        if self.hide_on_close
            && let Some(primary) = primary_window
        {
            primary
                .update(app, |_, window, app| {
                    window.on_window_should_close(app, |_window, app| {
                        app.hide();
                        false
                    });
                })
                .map_err(|error| error.to_string())?;
        }

        let (sender, receiver) = flume::unbounded();
        #[cfg(target_os = "linux")]
        {
            let icon = self.icon.clone();
            let tooltip = self.tooltip.clone();
            let labels = self
                .items
                .borrow()
                .iter()
                .map(|item| (item.id.clone(), item.label.clone()))
                .collect::<Vec<_>>();
            let separators = self.separators.clone();
            let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);
            std::thread::Builder::new()
                .name("wabou-tray-gtk".into())
                .spawn(move || {
                    let result = gtk::init()
                        .map_err(|error| error.to_string())
                        .and_then(|()| {
                            let menu = Menu::new();
                            for position in 0..=labels.len() {
                                if separators.contains(&position) {
                                    menu.append(&PredefinedMenuItem::separator())
                                        .map_err(|error| error.to_string())?;
                                }
                                if let Some((id, label)) = labels.get(position) {
                                    menu.append(&MenuItem::with_id(
                                        MenuId::new(id.clone()),
                                        label,
                                        true,
                                        None,
                                    ))
                                    .map_err(|error| error.to_string())?;
                                }
                            }
                            MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
                                let _ = sender.send(event.id().0.clone());
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
                        });
                    match result {
                        Ok(_native) => {
                            let _ = ready_sender.send(Ok(()));
                            gtk::main();
                        }
                        Err(error) => {
                            let _ = ready_sender.send(Err(error));
                        }
                    }
                })
                .map_err(|error| error.to_string())?;
            ready_receiver.recv().map_err(|error| error.to_string())??;
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.native = Some(self.build_native(sender)?);
        }

        let items = self.items.clone();
        self.task = Some(app.spawn(async move |app| {
            while let Ok(id) = receiver.recv_async().await {
                let mut context = TrayContext {
                    app,
                    primary_window,
                };
                if let Some(item) = items.borrow_mut().iter_mut().find(|item| item.id == id) {
                    (item.action)(&mut context);
                }
            }
        }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_dimensions_and_item_ids_are_validated_before_native_install() {
        assert!(TrayImage::from_rgba(vec![0; 3], 1, 1).is_err());
        let icon = TrayImage::from_rgba(vec![255; 4], 1, 1).unwrap();
        let tray =
            SystemTray::new(icon)
                .item("open", "Open", |_| {})
                .item("open", "Duplicate", |_| {});
        assert_eq!(
            tray.validate_item_ids().unwrap_err(),
            "duplicate native menu item id `open`"
        );
    }
}
