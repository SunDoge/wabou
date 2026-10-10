//! Controlled command menu. Layout and keyboard behavior belong to the host.
use crate::*;

type MenuAction = dyn for<'cx, 'host> Fn(&'cx Cx<'host>) -> Result<()>;

enum MenuCheck {
    Toggle(State<bool>),
    Radio(State<i64>, i64),
}

#[derive(derive_setters::Setters)]
#[setters(generate = false)]
pub struct MenuItem {
    key: Key,
    label: String,
    pub enabled: bool,
    /// Red foreground and a subtle red highlight for destructive commands.
    #[setters(generate)]
    pub destructive: bool,
    action: Box<MenuAction>,
    checked: Option<MenuCheck>,
    pub close_on_select: bool,
}
impl MenuItem {
    /// Identity belongs to the command, independently of its current position.
    pub fn new(
        key: impl Into<Key>,
        label: impl Into<String>,
        action: impl for<'cx, 'host> Fn(&'cx Cx<'host>) -> Result<()> + 'static,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            enabled: true,
            destructive: false,
            action: Box::new(action),
            checked: None,
            close_on_select: true,
        }
    }
    /// Toggle a durable setting without closing the menu. Set close_on_select
    /// to true when selection should finish the interaction.
    pub fn checkbox(key: impl Into<Key>, label: impl Into<String>, checked: State<bool>) -> Self {
        let mut item = Self::new(key, label, move |cx| checked.set(cx, !checked.get(cx)?));
        item.checked = Some(MenuCheck::Toggle(checked));
        item.close_on_select = false;
        item
    }
    /// Choose one value of a shared durable state without closing the menu.
    pub fn radio(
        key: impl Into<Key>,
        label: impl Into<String>,
        selected: State<i64>,
        value: i64,
    ) -> Self {
        let mut item = Self::new(key, label, move |cx| selected.set(cx, value));
        item.checked = Some(MenuCheck::Radio(selected, value));
        item.close_on_select = false;
        item
    }
    fn view(self, cx: &Cx<'_>, key: Key, close_tree: Vec<State<bool>>) -> Result<View> {
        let (role, checked) = match self.checked {
            Some(MenuCheck::Toggle(state)) => (abi::semantics::MENU_CHECKBOX, state.get(cx)?),
            Some(MenuCheck::Radio(state, value)) => {
                (abi::semantics::MENU_RADIO, state.get(cx)? == value)
            }
            None => (abi::semantics::MENU_ITEM, false),
        };
        let close = self.close_on_select;
        let action = self.action;
        let mut view = Button {
            enabled: self.enabled,
            variant: if self.destructive {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Secondary
            },
            ..Button::label(self.label).key(key)
        }
        .on_click(move |cx| {
            action(cx)?;
            if close {
                for state in &close_tree {
                    state.set(cx, false)?;
                }
            }
            Ok(())
        });
        view.node.semantics.role = role;
        view.node.semantics.selected = u32::from(checked);
        Ok(view)
    }
}
/// Ordered menu content. Separators are noninteractive and have stable keys.
pub enum MenuEntry {
    Item(MenuItem),
    Submenu {
        menu: DropdownMenu,
        entries: Vec<MenuEntry>,
    },
    Separator(Key),
    Group {
        key: Key,
        label: String,
        items: Vec<MenuItem>,
    },
}
impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        Self::Item(item)
    }
}
impl MenuEntry {
    pub fn submenu(menu: DropdownMenu, entries: Vec<MenuEntry>) -> Self {
        Self::Submenu { menu, entries }
    }
    pub fn group(key: impl Into<Key>, label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self::Group {
            key: key.into(),
            label: label.into(),
            items,
        }
    }
    pub fn separator(key: impl Into<Key>) -> Self {
        Self::Separator(key.into())
    }
}
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with items or entries and mount the returned view"]
pub struct DropdownMenu {
    #[setters(generate, into)]
    pub key: Key,
    pub label: String,
    pub enabled: bool,
    pub width: f32,
    #[setters(generate)]
    pub placement: PopoverPlacement,
    open: State<bool>,
}
impl DropdownMenu {
    pub fn new(open: impl Into<State<bool>>, label: impl Into<String>) -> Self {
        let open = open.into();
        Self {
            key: Key::named("component.menu").scoped(open.key()),
            label: label.into(),
            enabled: true,
            width: 240.0,
            placement: PopoverPlacement::BottomStart,
            open,
        }
    }
    /// Current implementation owns every command. An action and its successful
    /// close happen in one actor turn; rejected actions retain the open menu.
    pub fn items(self, cx: &Cx<'_>, items: Vec<MenuItem>) -> Result<View> {
        self.entries(cx, items.into_iter().map(MenuEntry::from).collect())
    }
    pub fn entries(self, cx: &Cx<'_>, entries: Vec<MenuEntry>) -> Result<View> {
        let mut close_tree = vec![self.open];
        collect_open_states(&entries, &mut close_tree);
        let mut state_keys = std::collections::HashSet::new();
        if close_tree
            .iter()
            .any(|state| !state_keys.insert(state.key()))
        {
            return Err(Error::InvalidView("nested menus need distinct open states"));
        }
        self.build(cx, entries, close_tree, vec![])
    }
    fn build(
        self,
        cx: &Cx<'_>,
        entries: Vec<MenuEntry>,
        close_tree: Vec<State<bool>>,
        mut ancestors: Vec<State<bool>>,
    ) -> Result<View> {
        ancestors.push(self.open);
        let mut keys = std::collections::HashSet::new();
        for entry in &entries {
            let (key, valid_label) = match entry {
                MenuEntry::Item(item) => (item.key, !item.label.trim().is_empty()),
                MenuEntry::Separator(key) => (*key, true),
                MenuEntry::Submenu { menu, .. } => (menu.key, !menu.label.trim().is_empty()),
                MenuEntry::Group { key, label, items } => {
                    for item in items {
                        if item.key.0 == 0
                            || !keys.insert(item.key.0)
                            || item.label.trim().is_empty()
                        {
                            return Err(Error::InvalidView(
                                "menu entries need distinct nonzero keys and command labels",
                            ));
                        }
                    }
                    (*key, !label.trim().is_empty())
                }
            };
            if key.0 == 0 || !keys.insert(key.0) || !valid_label {
                return Err(Error::InvalidView(
                    "menu entries need distinct nonzero keys and command labels",
                ));
            }
        }
        let open = self.open;
        let content = entries
            .into_iter()
            .map(|entry| {
                let item = match entry {
                    MenuEntry::Submenu { mut menu, entries } => {
                        menu.key = self.key.scoped(abi::key("submenu")).scoped(menu.key.0);
                        menu.placement = PopoverPlacement::RightStart;
                        let mut view =
                            menu.build(cx, entries, close_tree.clone(), ancestors.clone())?;
                        view.node.semantics.role = abi::semantics::SUBMENU;
                        view.children[0].node.semantics.role = abi::semantics::SUBMENU_TRIGGER;
                        return Ok(view);
                    }
                    MenuEntry::Group { key, label, items } => {
                        let group_key = self.key.scoped(abi::key("group")).scoped(key.0);
                        let label_key = group_key.scoped(abi::key("label"));
                        let mut title = Text::new(label).key(label_key).style(ViewStyle {
                            font_size: 12.0,
                            ..default()
                        });
                        title.node.semantics.role = abi::semantics::MENU_LABEL;
                        let mut children = vec![title];
                        for item in items {
                            let item_key = self.key.scoped(abi::key("item")).scoped(item.key.0);
                            children.push(item.view(cx, item_key, close_tree.clone())?);
                        }
                        let mut group = Column {
                            gap: 4.0,
                            padding: 0.0,
                            ..default()
                        }
                        .key(group_key)
                        .children(children);
                        group.node.semantics.role = abi::semantics::MENU_GROUP;
                        group.node.semantics.labelled_by = label_key.0;
                        return Ok(group);
                    }
                    MenuEntry::Item(item) => item,
                    MenuEntry::Separator(key) => {
                        return Ok(Separator::horizontal()
                            .key(self.key.scoped(abi::key("item")).scoped(key.0))
                            .into_view());
                    }
                };
                {
                    let key = self.key.scoped(abi::key("item")).scoped(item.key.0);
                    item.view(cx, key, close_tree.clone())
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let mut popover = Popover::new(open, self.label)
            .key(self.key)
            .placement(self.placement);
        popover.enabled = self.enabled;
        popover.width = self.width;
        let mut view = popover.children(cx, content)?;
        view.node.padding = 0.0;
        view.node.gap = 0.0;
        let trigger_tree = close_tree.clone();
        let trigger_path = ancestors.clone();
        view.children[0].event = Some(std::rc::Rc::new(move |cx, _| {
            let next = if trigger_path.len() > 1 {
                true
            } else {
                !open.get(cx)?
            };
            for state in &trigger_tree {
                if state.key() != open.key()
                    && !trigger_path
                        .iter()
                        .any(|parent| parent.key() == state.key())
                {
                    state.set(cx, false)?;
                }
            }
            open.set(cx, next)
        }));
        let descendants: Vec<_> = close_tree
            .into_iter()
            .filter(|state| !ancestors.iter().any(|parent| parent.key() == state.key()))
            .collect();
        view.children[1].event = Some(std::rc::Rc::new(move |cx, _| {
            for state in &descendants {
                state.set(cx, false)?;
            }
            open.set(cx, false)
        }));
        view.children[0].node.semantics.role = abi::semantics::MENU_TRIGGER;
        view.children[1].node.semantics.role = abi::semantics::MENU_PANEL;
        view.children[1].node.padding = 6.0;
        view.children[1].node.gap = 4.0;
        Ok(view)
    }
}

fn collect_open_states(entries: &[MenuEntry], result: &mut Vec<State<bool>>) {
    for entry in entries {
        if let MenuEntry::Submenu { menu, entries } = entry {
            result.push(menu.open);
            collect_open_states(entries, result);
        }
    }
}

/// Top-level menu composition with shared tree transactions.
/// Hosts provide horizontal navigation, remembered Tab entry and open-session hover.
#[must_use = "finish this widget builder with menus and mount the returned view"]
pub struct Menubar {
    pub key: Key,
}
impl Menubar {
    pub fn new(key: impl Into<Key>) -> Self {
        Self { key: key.into() }
    }
    /// Every root and descendant needs its own durable open state. Opening one
    /// root clears the other trees; successful closing commands clear all roots.
    pub fn menus(self, cx: &Cx<'_>, menus: Vec<(DropdownMenu, Vec<MenuEntry>)>) -> Result<View> {
        if self.key.0 == 0 {
            return Err(Error::InvalidView("menubar needs a nonzero key"));
        }
        let mut keys = std::collections::HashSet::new();
        let mut states = Vec::new();
        for (menu, entries) in &menus {
            if menu.key.0 == 0 || !keys.insert(menu.key.0) || menu.label.trim().is_empty() {
                return Err(Error::InvalidView(
                    "menubar menus need distinct nonzero keys and labels",
                ));
            }
            states.push(menu.open);
            collect_open_states(entries, &mut states);
        }
        keys.clear();
        if states.iter().any(|state| !keys.insert(state.key())) {
            return Err(Error::InvalidView(
                "menubar menus need distinct open states",
            ));
        }
        let children = menus
            .into_iter()
            .map(|(mut menu, entries)| {
                menu.key = self.key.scoped(abi::key("menu")).scoped(menu.key.0);
                menu.build(cx, entries, states.clone(), vec![])
            })
            .collect::<Result<Vec<_>>>()?;
        let mut view = Row {
            key: self.key,
            gap: 4.0,
            padding: 4.0,
        }
        .children(children);
        view.node.semantics.role = abi::semantics::MENUBAR;
        Ok(view)
    }
}

/// Controlled context-menu state and commands with an explicit action target.
/// Native and browser hosts support pointer and keyboard context activation.
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "finish this widget builder with surface, items or entries and mount the returned view"]
pub struct ContextMenu {
    #[setters(generate, into)]
    pub key: Key,
    pub label: String,
    pub enabled: bool,
    pub width: f32,
    open: State<bool>,
}
impl ContextMenu {
    /// Attach a context target to arbitrary content without replacing child
    /// actions. The region has its own keyboard focus and a visible label.
    /// Both hosts route descendant context activation to the nearest region.
    pub fn surface(self, cx: &Cx<'_>, content: View, entries: Vec<MenuEntry>) -> Result<View> {
        let label = self.label.clone();
        let mut view = self.entries(cx, entries)?;
        let trigger = &mut view.children[0];
        let label_key = Key(trigger.node.key).scoped(abi::key("label"));
        let mut surface = Column {
            key: Key(trigger.node.key),
            gap: 8.0,
            padding: 12.0,
        }
        .children(vec![Text::new(label).key(label_key).into_view(), content]);
        surface.node.semantics = trigger.node.semantics;
        surface.node.semantics.role = abi::semantics::CONTEXT_SURFACE;
        surface.node.semantics.labelled_by = label_key.0;
        surface.node.disabled = trigger.node.disabled;
        surface.event = trigger.event.take();
        *trigger = surface;
        Ok(view)
    }
    pub fn new(open: impl Into<State<bool>>, label: impl Into<String>) -> Self {
        let open = open.into();
        Self {
            key: Key::named("component.context_menu").scoped(open.key()),
            label: label.into(),
            enabled: true,
            width: 240.0,
            open,
        }
    }
    pub fn items(self, cx: &Cx<'_>, items: Vec<MenuItem>) -> Result<View> {
        self.entries(cx, items.into_iter().map(MenuEntry::from).collect())
    }
    pub fn entries(self, cx: &Cx<'_>, entries: Vec<MenuEntry>) -> Result<View> {
        let open = self.open;
        let mut descendants = Vec::new();
        collect_open_states(&entries, &mut descendants);
        let mut menu = DropdownMenu::new(open, self.label).key(self.key);
        menu.enabled = self.enabled;
        menu.width = self.width;
        let mut view = menu.entries(cx, entries)?;
        view.children[0].node.semantics.role = abi::semantics::CONTEXT_TRIGGER;
        // Repeated context activation opens/repositions, rather than toggling
        // closed. Reset descendants as a new context interaction starts.
        view.children[0].event = Some(std::rc::Rc::new(move |cx, _| {
            for state in &descendants {
                state.set(cx, false)?;
            }
            open.set(cx, true)
        }));
        Ok(view)
    }
}
