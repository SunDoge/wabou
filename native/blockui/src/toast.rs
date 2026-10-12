//! Actor-owned notification data with one host deadline, never a per-toast worker.
use crate::*;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ToastVariant {
    Default,
    Success,
    Warning,
    Error,
}
/// Fixed notification stack anchor. Identity and deadlines do not depend on placement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum ToastPosition {
    #[default]
    BottomRight = 0,
    BottomLeft = 1,
    BottomCenter = 2,
    TopRight = 3,
    TopLeft = 4,
    TopCenter = 5,
}
impl TryFrom<i64> for ToastPosition {
    type Error = Error;
    fn try_from(value: i64) -> Result<Self> {
        Ok(match value {
            0 => Self::BottomRight,
            1 => Self::BottomLeft,
            2 => Self::BottomCenter,
            3 => Self::TopRight,
            4 => Self::TopLeft,
            5 => Self::TopCenter,
            _ => return Err(Error::InvalidView("invalid toast position")),
        })
    }
}
/// Notification content. Actions and errors default to persistent delivery.
pub struct Toast {
    pub title: String,
    pub description: String,
    pub variant: ToastVariant,
    duration: Option<Duration>,
    action: Option<(String, i64)>,
}
impl Toast {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            variant: ToastVariant::Default,
            duration: Some(Duration::from_secs(5)),
            action: None,
        }
    }
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        if variant == ToastVariant::Error {
            self.duration = None;
        }
        self
    }
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self
    }
    /// Action messages stay visible until acted on or dismissed.
    pub fn action(mut self, label: impl Into<String>, value: i64) -> Self {
        self.action = Some((label.into(), value));
        self.duration = None;
        self
    }
}
#[derive(Clone, Debug)]
pub struct ToastEntry {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub variant: ToastVariant,
    pub action: Option<(String, i64)>,
    deadline: Option<u64>,
}
/// Register `state()` in the owning actor and a current-code expiry action that
/// calls `expire`. State, deadlines and IDs survive compatible implementation reload.
#[derive(Clone, Copy)]
pub struct Toasts {
    rows: State<Vec<String>>,
    serial: State<i64>,
    revision: State<i64>,
    paused: State<String>,
    timer: Task,
    key: Key,
    limit: usize,
    position: ToastPosition,
}
impl Toasts {
    pub const fn new(name: &str) -> Self {
        let key = Key::named(name);
        Self {
            rows: State::<Vec<String>>::new("toast.rows").scoped(key),
            serial: State::<i64>::new("toast.serial", 0).scoped(key),
            revision: State::<i64>::new("toast.revision", 0).scoped(key),
            paused: State::<String>::new("toast.paused").scoped(key),
            timer: Task::new("toast.deadline").scoped(key),
            key,
            limit: 3,
            position: ToastPosition::BottomRight,
        }
    }
    /// Anchor this queue without changing its durable keys, IDs or timer.
    pub const fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }
    /// Maximum visible notifications, 1..=8. Overflow retires the oldest entry.
    pub const fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    pub const fn state(self) -> [StateSpec; 4] {
        [
            self.rows.spec(),
            self.serial.spec(),
            self.revision.spec(),
            self.paused.spec(),
        ]
    }
    fn paused_at(self, cx: &Cx<'_>) -> Result<Option<u64>> {
        let value = self.paused.get(cx)?;
        if value.is_empty() {
            Ok(None)
        } else {
            value
                .parse()
                .map(Some)
                .map_err(|_| Error::InvalidView("invalid toast pause clock"))
        }
    }
    fn now(self, cx: &Cx<'_>) -> Result<u64> {
        Ok(self.paused_at(cx)?.unwrap_or(cx.monotonic_ms()?))
    }
    /// Pause the queue while it is being read or operated. Resume preserves
    /// remaining time; repeated observations neither reset nor extend deadlines.
    pub fn set_paused(self, cx: &Cx<'_>, paused: bool, expire: Action) -> Result<()> {
        let previous = self.paused_at(cx)?;
        if previous.is_some() == paused {
            return Ok(());
        }
        let now = cx.monotonic_ms()?;
        let mut entries = self.entries(cx)?;
        if let Some(start) = previous {
            let elapsed = now.checked_sub(start).ok_or(Error::Overflow)?;
            for entry in &mut entries {
                entry.deadline = entry
                    .deadline
                    .map(|d| d.checked_add(elapsed).ok_or(Error::Overflow))
                    .transpose()?;
            }
            self.paused.set(cx, "")?;
        } else {
            // A late hover cannot resurrect entries whose time has already elapsed.
            entries.retain(|e| e.deadline.is_none_or(|d| d > now));
            self.paused.set(cx, &now.to_string())?;
        }
        self.store(cx, &entries, expire)
    }
    fn validate(self) -> Result<()> {
        if !(1..=8).contains(&self.limit) {
            return Err(Error::InvalidView("toast limit must be 1..=8"));
        }
        Ok(())
    }
    pub fn entries(self, cx: &Cx<'_>) -> Result<Vec<ToastEntry>> {
        self.validate()?;
        let rows = self.rows.get(cx)?;
        if !rows.len().is_multiple_of(7) || rows.len() / 7 > 8 {
            return Err(Error::InvalidView("invalid toast storage"));
        }
        let mut entries = Vec::new();
        for row in rows.as_chunks::<7>().0 {
            let id = row[0]
                .parse::<i64>()
                .map_err(|_| Error::InvalidView("invalid toast ID"))?;
            if id <= 0 || entries.iter().any(|entry: &ToastEntry| entry.id == id) {
                return Err(Error::InvalidView("invalid toast ID"));
            }
            let variant = match row[3].as_str() {
                "0" => ToastVariant::Default,
                "1" => ToastVariant::Success,
                "2" => ToastVariant::Warning,
                "3" => ToastVariant::Error,
                _ => return Err(Error::InvalidView("invalid toast variant")),
            };
            let deadline = if row[4].is_empty() {
                None
            } else {
                Some(
                    row[4]
                        .parse::<u64>()
                        .map_err(|_| Error::InvalidView("invalid toast deadline"))?,
                )
            };
            let action = if row[5].is_empty() {
                None
            } else {
                Some((
                    row[5].clone(),
                    row[6]
                        .parse::<i64>()
                        .map_err(|_| Error::InvalidView("invalid toast action"))?,
                ))
            };
            entries.push(ToastEntry {
                id,
                title: row[1].clone(),
                description: row[2].clone(),
                variant,
                deadline,
                action,
            });
        }
        Ok(entries)
    }
    fn store(self, cx: &Cx<'_>, entries: &[ToastEntry], expire: Action) -> Result<()> {
        let mut rows = Vec::new();
        for e in entries {
            rows.extend([
                e.id.to_string(),
                e.title.clone(),
                e.description.clone(),
                (e.variant as u32).to_string(),
                e.deadline.map(|v| v.to_string()).unwrap_or_default(),
                e.action.as_ref().map(|a| a.0.clone()).unwrap_or_default(),
                e.action
                    .as_ref()
                    .map(|a| a.1.to_string())
                    .unwrap_or_default(),
            ]);
        }
        self.rows.set(cx, &rows)?;
        // Already admitted timer messages cannot be cancelled. Revision checking
        // makes them harmless after clear, dismissal, insertion or rescheduling.
        let revision = self
            .revision
            .get(cx)?
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        self.revision.set(cx, revision)?;
        self.timer.cancel(cx)?;
        if self.paused_at(cx)?.is_none()
            && let Some(deadline) = entries.iter().filter_map(|e| e.deadline).min()
        {
            self.timer.delay(
                cx,
                Duration::from_millis(deadline.saturating_sub(cx.monotonic_ms()?)),
                expire,
                revision,
            )?;
        }
        Ok(())
    }
    pub fn show(self, cx: &Cx<'_>, toast: Toast, expire: Action) -> Result<i64> {
        self.validate()?;
        let entry = self.prepare(cx, 0, toast)?;
        let id = self.serial.get(cx)?.checked_add(1).ok_or(Error::Overflow)?;
        let now = self.now(cx)?;
        let mut entries = self.entries(cx)?;
        entries.retain(|e| e.deadline.is_none_or(|d| d > now));
        entries.push(ToastEntry { id, ..entry });
        if entries.len() > self.limit {
            entries.drain(..entries.len() - self.limit);
        }
        self.serial.set(cx, id)?;
        self.store(cx, &entries, expire)?;
        Ok(id)
    }
    fn prepare(self, cx: &Cx<'_>, id: i64, toast: Toast) -> Result<ToastEntry> {
        if toast.title.trim().is_empty()
            || toast.action.as_ref().is_some_and(|a| a.0.trim().is_empty())
        {
            return Err(Error::InvalidView(
                "toast title and action must not be empty",
            ));
        }
        let deadline = toast
            .duration
            .map(|d| {
                if d.is_zero() {
                    return Err(Error::InvalidView("toast duration must be positive"));
                }
                self.now(cx)?
                    .checked_add(crate::timer_millis(d)?)
                    .ok_or(Error::Overflow)
            })
            .transpose()?;
        Ok(ToastEntry {
            id,
            title: toast.title,
            description: toast.description,
            variant: toast.variant,
            action: toast.action,
            deadline,
        })
    }
    /// Replace a live notification without changing its ID, order or widget keys.
    /// The new duration starts now; persistent content cancels its old deadline.
    /// Missing or already expired IDs return false and never recreate a toast.
    pub fn update(self, cx: &Cx<'_>, id: i64, toast: Toast, expire: Action) -> Result<bool> {
        let mut entries = self.entries(cx)?;
        let now = self.now(cx)?;
        let Some(entry) = entries
            .iter_mut()
            .find(|e| e.id == id && e.deadline.is_none_or(|d| d > now))
        else {
            return Ok(false);
        };
        *entry = self.prepare(cx, id, toast)?;
        self.store(cx, &entries, expire)?;
        Ok(true)
    }
    pub fn dismiss(self, cx: &Cx<'_>, id: i64, expire: Action) -> Result<bool> {
        let mut entries = self.entries(cx)?;
        let before = entries.len();
        entries.retain(|e| e.id != id);
        if before == entries.len() {
            return Ok(false);
        }
        self.store(cx, &entries, expire)?;
        Ok(true)
    }
    pub fn clear(self, cx: &Cx<'_>) -> Result<()> {
        self.validate()?;
        self.rows.set(cx, &[])?;
        self.revision
            .update(cx, |v| v.checked_add(1).ok_or(Error::Overflow))?;
        self.timer.cancel(cx)
    }
    /// Called only by the registered current implementation's timer action.
    pub fn expire(self, cx: &Cx<'_>, revision: i64, expire: Action) -> Result<()> {
        if self.paused_at(cx)?.is_some() || revision != self.revision.get(cx)? {
            return Ok(());
        }
        let now = self.now(cx)?;
        let mut entries = self.entries(cx)?;
        entries.retain(|e| e.deadline.is_none_or(|d| d > now));
        self.store(cx, &entries, expire)
    }
    /// Mount once outside conditional pages. No callback enters host scheduling.
    /// `on_action` runs in the current UI implementation; success dismisses the
    /// selected notification, while an error rolls back the whole action turn.
    pub fn view(
        self,
        cx: &Cx<'_>,
        expire: Action,
        on_action: fn(&Cx<'_>, i64) -> Result<()>,
    ) -> Result<View> {
        let mut children = Vec::new();
        let entries = self.entries(cx)?;
        if entries.len() > 1 {
            children.push(
                Button {
                    variant: ButtonVariant::Secondary,
                    ..Button::label("Clear notifications").key(self.key.scoped(abi::key("clear")))
                }
                .on_click(move |cx| self.clear(cx)),
            );
        }
        for e in entries {
            let key = self.key.scoped(e.id as u64);
            let mut content = vec![
                Text::new(e.title.clone())
                    .key(key.scoped(abi::key("title")))
                    .style(ViewStyle {
                        font_size: 15.0,
                        text_tone: match e.variant {
                            ToastVariant::Success => TextTone::Accent,
                            ToastVariant::Warning | ToastVariant::Error => TextTone::Error,
                            _ => TextTone::Default,
                        },
                        ..default()
                    }),
                Description::new(e.description)
                    .key(key.scoped(abi::key("description")))
                    .into_view(),
            ];
            let mut actions = Vec::new();
            if let Some((label, value)) = e.action {
                actions.push(
                    Button::label(label)
                        .key(key.scoped(abi::key("action")))
                        .on_click(move |cx| {
                            on_action(cx, value)?;
                            self.dismiss(cx, e.id, expire)?;
                            Ok(())
                        }),
                );
            }
            actions.push(
                Button {
                    variant: ButtonVariant::Secondary,
                    ..Button::label(format!("Dismiss {}", e.title))
                        .key(key.scoped(abi::key("dismiss")))
                }
                .on_click(move |cx| {
                    self.dismiss(cx, e.id, expire)?;
                    Ok(())
                }),
            );
            content.push(
                ActionGroup::default()
                    .key(key.scoped(abi::key("actions")))
                    .children(actions),
            );
            let mut card = Card {
                padding: 16.0,
                gap: 6.0,
                ..default()
            }
            .key(key)
            .children(content);
            card.node.semantics.role = if e.variant == ToastVariant::Error {
                abi::semantics::ALERT
            } else {
                abi::semantics::STATUS
            };
            card.node.semantics.labelled_by = key.scoped(abi::key("title")).0;
            children.push(card);
        }
        let mut viewport = ScrollView {
            height: 0.0,
            ..default()
        }
        .key(self.key)
        .children(children);
        viewport.node.semantics.role = abi::semantics::TOAST_VIEWPORT;
        viewport.node.gap = 12.0;
        viewport.node.overlay_position = self.position as u32;
        viewport.node.action_payload = i64::from(self.paused_at(cx)?.is_some());
        viewport.event = Some(std::rc::Rc::new(move |cx, value| {
            if !matches!(value, 0 | 1) {
                return Err(Error::InvalidView("invalid toast activity"));
            }
            self.set_paused(cx, value != 0, expire)
        }));
        Ok(viewport)
    }
}
