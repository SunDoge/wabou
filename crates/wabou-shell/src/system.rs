use std::path::Path;

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use winit::window::Window;

use crate::{
    DialogFilter, MessageDialogButtons, MessageDialogLevel, MessageDialogRequest,
    NotificationRequest, OpenDialogRequest, PickDirectoryRequest, SaveDialogRequest,
};

fn configure_file_dialog(
    mut dialog: FileDialog,
    parent: Option<&dyn Window>,
    title: Option<&str>,
    directory: Option<&str>,
    filters: &[DialogFilter],
) -> FileDialog {
    if let Some(parent) = parent {
        dialog = dialog.set_parent(parent);
    }
    if let Some(title) = title.filter(|title| !title.is_empty()) {
        dialog = dialog.set_title(title);
    }
    if let Some(directory) = directory.filter(|directory| !directory.is_empty()) {
        dialog = dialog.set_directory(Path::new(directory));
    }
    for filter in filters {
        if !filter.name.is_empty() && !filter.extensions.is_empty() {
            dialog = dialog.add_filter(&filter.name, &filter.extensions);
        }
    }
    dialog
}

fn path_string(path: std::path::PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

pub(crate) fn open_dialog(
    parent: Option<&dyn Window>,
    request: OpenDialogRequest,
) -> Option<Vec<String>> {
    let dialog = configure_file_dialog(
        FileDialog::new(),
        parent,
        request.title.as_deref(),
        request.directory.as_deref(),
        &request.filters,
    );
    if request.multiple {
        dialog
            .pick_files()
            .map(|paths| paths.into_iter().map(path_string).collect())
    } else {
        dialog.pick_file().map(|path| vec![path_string(path)])
    }
}

pub(crate) fn save_dialog(
    parent: Option<&dyn Window>,
    request: SaveDialogRequest,
) -> Option<Vec<String>> {
    let mut dialog = configure_file_dialog(
        FileDialog::new(),
        parent,
        request.title.as_deref(),
        request.directory.as_deref(),
        &request.filters,
    );
    if let Some(name) = request.default_name.filter(|name| !name.is_empty()) {
        dialog = dialog.set_file_name(name);
    }
    dialog.save_file().map(|path| vec![path_string(path)])
}

pub(crate) fn pick_directory(
    parent: Option<&dyn Window>,
    request: PickDirectoryRequest,
) -> Option<Vec<String>> {
    configure_file_dialog(
        FileDialog::new(),
        parent,
        request.title.as_deref(),
        request.directory.as_deref(),
        &[],
    )
    .pick_folder()
    .map(|path| vec![path_string(path)])
}

pub(crate) fn message_dialog(parent: Option<&dyn Window>, request: MessageDialogRequest) -> String {
    let mut dialog = MessageDialog::new()
        .set_description(request.message)
        .set_level(match request.level {
            MessageDialogLevel::Info => MessageLevel::Info,
            MessageDialogLevel::Warning => MessageLevel::Warning,
            MessageDialogLevel::Error => MessageLevel::Error,
        })
        .set_buttons(match request.buttons {
            MessageDialogButtons::Ok => MessageButtons::Ok,
            MessageDialogButtons::OkCancel => MessageButtons::OkCancel,
            MessageDialogButtons::YesNo => MessageButtons::YesNo,
            MessageDialogButtons::YesNoCancel => MessageButtons::YesNoCancel,
        });
    if let Some(parent) = parent {
        dialog = dialog.set_parent(parent);
    }
    if let Some(title) = request.title.filter(|title| !title.is_empty()) {
        dialog = dialog.set_title(title);
    }
    match dialog.show() {
        MessageDialogResult::Yes => "yes",
        MessageDialogResult::No => "no",
        MessageDialogResult::Ok => "ok",
        MessageDialogResult::Cancel => "cancel",
        MessageDialogResult::Custom(_) => "custom",
    }
    .into()
}

pub(crate) fn show_notification(
    app_name: &str,
    request: NotificationRequest,
) -> Result<(), String> {
    if request.title.trim().is_empty() {
        return Err("notification title must not be empty".into());
    }
    let mut notification = notify_rust::Notification::new();
    notification.appname(app_name).summary(&request.title);
    if let Some(body) = request.body.as_deref() {
        notification.body(body);
    }
    if let Some(icon) = request.icon.as_deref() {
        #[cfg(target_os = "windows")]
        notification.image_path(icon);
        #[cfg(not(target_os = "windows"))]
        notification.icon(icon);
    }
    #[cfg(target_os = "windows")]
    notification.app_id(app_name);
    #[cfg(all(unix, not(target_os = "macos")))]
    if request.silent {
        notification.hint(notify_rust::Hint::SuppressSound(true));
    }
    notification
        .show()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_configuration_accepts_empty_optional_fields() {
        let request = OpenDialogRequest::default();
        let _dialog = configure_file_dialog(
            FileDialog::new(),
            None,
            request.title.as_deref(),
            request.directory.as_deref(),
            &request.filters,
        );
    }

    #[test]
    fn empty_notification_titles_are_rejected_without_contacting_the_os() {
        let result = show_notification(
            "Wabou Test",
            NotificationRequest {
                title: "   ".into(),
                body: None,
                icon: None,
                silent: false,
            },
        );
        assert_eq!(result, Err("notification title must not be empty".into()));
    }
}
