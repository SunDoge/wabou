use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use wabou::{
    VelloHybridExtensionContext, VelloHybridShellExtension, VelloHybridWakeCallback,
    initial_window_resource_key,
};

/// Best-effort global launcher shortcut. Linux support is limited to X11 by
/// the platform crate; failure leaves the tray and ordinary window usable.
pub struct LauncherShortcut {
    hotkey: HotKey,
    manager: Option<GlobalHotKeyManager>,
    receiver: Option<flume::Receiver<GlobalHotKeyEvent>>,
}

impl LauncherShortcut {
    pub fn new() -> Self {
        Self {
            hotkey: HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space),
            manager: None,
            receiver: None,
        }
    }
}

impl VelloHybridShellExtension for LauncherShortcut {
    fn initialize(&mut self, wake: VelloHybridWakeCallback) -> Result<(), String> {
        let manager = match GlobalHotKeyManager::new() {
            Ok(manager) => manager,
            Err(error) => {
                tracing::warn!(%error, "global launcher shortcut is unavailable");
                return Ok(());
            }
        };
        if let Err(error) = manager.register(self.hotkey) {
            tracing::warn!(%error, "global launcher shortcut could not be registered");
            return Ok(());
        }
        let (sender, receiver) = flume::unbounded();
        GlobalHotKeyEvent::set_event_handler(Some(move |event| {
            let _ = sender.send(event);
            wake();
        }));
        self.manager = Some(manager);
        self.receiver = Some(receiver);
        Ok(())
    }

    fn poll(&mut self, context: &mut VelloHybridExtensionContext<'_>) {
        let Some(receiver) = &self.receiver else {
            return;
        };
        while let Ok(event) = receiver.try_recv() {
            if event.id() == self.hotkey.id() && event.state() == HotKeyState::Pressed {
                context.show_window(initial_window_resource_key(0));
            }
        }
    }

    fn shutdown(&mut self, _context: &mut VelloHybridExtensionContext<'_>) {
        if let Some(manager) = self.manager.take() {
            let _ = manager.unregister(self.hotkey);
        }
        self.receiver = None;
    }
}
