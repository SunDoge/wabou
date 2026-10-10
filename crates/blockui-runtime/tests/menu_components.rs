#![cfg(feature = "testing")]
use blockui::*;
use blockui_runtime::Actor as Host;
const OPEN: State<bool> = State::<bool>::new("menu.open", false);
const LAST: State<String> = State::<String>::new("menu.last");
struct Fixture;
fn menu(cx: &Cx<'_>, saved: &'static str) -> Result<View> {
    let mut disabled = MenuItem::new("disabled", "Unavailable", |cx| LAST.set(cx, "wrong"));
    disabled.enabled = false;
    DropdownMenu::new(OPEN, "Actions").key("commands").entries(
        cx,
        vec![
            MenuItem::new("save", "Save 🦀", move |cx| LAST.set(cx, saved)).into(),
            disabled.into(),
            MenuEntry::separator("divider"),
            MenuItem::new("reject", "Reject", |cx| {
                LAST.set(cx, "must roll back")?;
                Err(Error::Application("command rejected".into()))
            })
            .into(),
        ],
    )
}
impl Actor for Fixture {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), LAST.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        menu(cx, "Saved")
    }
}
struct Replacement;
impl Actor for Replacement {
    const STATE: &'static [StateSpec] = Fixture::STATE;
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        menu(cx, "Saved by current implementation")
    }
}
fn part(name: &str) -> u64 {
    Key::named("commands").scoped(abi::key(name)).0
}
fn item(name: &str) -> u64 {
    Key::named("commands")
        .scoped(abi::key("item"))
        .scoped(abi::key(name))
        .0
}
#[test]
fn current_commands_close_atomically_and_rejected_commands_roll_back()
-> std::result::Result<(), String> {
    let mut actor = Host::for_test::<Fixture>()?;
    assert_eq!(
        actor.projected_view().bindings[&part("trigger")]
            .node
            .semantics
            .role,
        abi::semantics::MENU_TRIGGER
    );
    assert!(actor.dismiss_popover(part("panel")).is_err());
    assert!(!actor.projected_view().bindings.contains_key(&item("save")));
    actor.dispatch_widget(part("trigger"))?;
    assert_eq!(
        actor.projected_view().bindings[&part("panel")]
            .node
            .semantics
            .role,
        abi::semantics::MENU_PANEL
    );
    assert_eq!(
        actor.projected_view().bindings[&item("save")]
            .node
            .semantics
            .role,
        abi::semantics::MENU_ITEM
    );
    actor.dispatch_widget(item("disabled"))?;
    assert_eq!(actor.text("menu.last"), "");
    assert!(
        actor
            .dispatch_widget(item("reject"))
            .unwrap_err()
            .contains("command rejected")
    );
    assert_eq!(actor.text("menu.last"), "");
    assert_eq!(
        actor.projected_view().bindings[&part("trigger")]
            .node
            .semantics
            .expanded,
        2
    );
    actor.replace_for_test::<Replacement>()?;
    assert_eq!(
        actor.projected_view().bindings[&part("trigger")]
            .node
            .semantics
            .expanded,
        2
    );
    let divider = &actor.projected_view().bindings[&item("divider")].node;
    assert_eq!(divider.semantics.role, abi::semantics::HORIZONTAL_SEPARATOR);
    assert_eq!(divider.action, 0);
    assert!(actor.dispatch_widget(item("divider")).is_err());
    actor.dispatch_widget(item("save"))?;
    assert_eq!(actor.text("menu.last"), "Saved by current implementation");
    assert_eq!(
        actor.projected_view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    assert!(!actor.projected_view().bindings.contains_key(&item("save")));
    assert!(actor.dispatch_widget(item("save")).is_err());
    actor.dispatch_widget(part("trigger"))?;
    actor.dismiss_popover(part("panel"))?;
    assert_eq!(
        actor.projected_view().bindings[&part("trigger")]
            .node
            .semantics
            .expanded,
        1
    );
    Ok(())
}
struct Invalid;
impl Actor for Invalid {
    const STATE: &'static [StateSpec] = &[OPEN.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        DropdownMenu::new(OPEN, "Actions").items(
            cx,
            vec![
                MenuItem::new("same", "One", |_| Ok(())),
                MenuItem::new("same", "Two", |_| Ok(())),
            ],
        )
    }
}
#[test]
fn duplicate_command_identity_is_rejected_even_when_closed() {
    assert!(
        Host::for_test::<Invalid>()
            .err()
            .unwrap()
            .contains("distinct nonzero keys")
    );
}

struct DuplicateSeparator;
impl Actor for DuplicateSeparator {
    const STATE: &'static [StateSpec] = &[OPEN.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        DropdownMenu::new(OPEN, "Actions").entries(
            cx,
            vec![
                MenuItem::new("same", "Command", |_| Ok(())).into(),
                MenuEntry::separator("same"),
            ],
        )
    }
}
#[test]
fn separator_identity_cannot_collide_with_a_command_when_closed() {
    assert!(
        Host::for_test::<DuplicateSeparator>()
            .err()
            .unwrap()
            .contains("distinct nonzero keys")
    );
}

const CHECKED: State<bool> = State::<bool>::new("menu.checked", false);
struct Checkable;
impl Actor for Checkable {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), CHECKED.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let mut closing = MenuItem::checkbox("close", "Toggle and close", CHECKED);
        closing.close_on_select = true;
        let mut disabled = MenuItem::checkbox("disabled", "Disabled setting", CHECKED);
        disabled.enabled = false;
        DropdownMenu::new(OPEN, "Settings").key("commands").items(
            cx,
            vec![
                MenuItem::checkbox("toggle", "Setting", CHECKED),
                disabled,
                closing,
            ],
        )
    }
}
#[test]
fn menu_checkbox_tracks_state_and_can_keep_open_or_close() -> std::result::Result<(), String> {
    let mut actor = Host::for_test::<Checkable>()?;
    actor.dispatch_widget(part("trigger"))?;
    assert_eq!(
        actor.projected_view().bindings[&item("toggle")]
            .node
            .semantics
            .selected,
        0
    );
    actor.dispatch_widget(item("disabled"))?;
    assert!(!actor.boolean("menu.checked"));
    actor.dispatch_widget(item("toggle"))?;
    assert!(actor.boolean("menu.checked"));
    assert_eq!(
        actor.projected_view().bindings[&item("toggle")]
            .node
            .semantics
            .selected,
        1
    );
    assert_eq!(
        actor.projected_view().bindings[&item("toggle")]
            .node
            .semantics
            .role,
        abi::semantics::MENU_CHECKBOX
    );
    assert_eq!(
        actor.projected_view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        0
    );
    actor.replace_for_test::<Checkable>()?;
    actor.dispatch_widget(item("close"))?;
    assert!(!actor.boolean("menu.checked"));
    assert_eq!(
        actor.projected_view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    Ok(())
}

const ORDER: State<i64> = State::<i64>::new("menu.order", 0);
struct RadioMenu;
impl Actor for RadioMenu {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), ORDER.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let mut disabled = MenuItem::radio("disabled", "Unavailable", ORDER, 2);
        disabled.enabled = false;
        DropdownMenu::new(OPEN, "Sort").key("commands").items(
            cx,
            vec![
                MenuItem::radio("name", "Name", ORDER, 0),
                MenuItem::radio("updated", "Updated", ORDER, 1),
                disabled,
            ],
        )
    }
}
#[test]
fn menu_radio_selection_is_mutually_exclusive_and_reselection_is_idempotent()
-> std::result::Result<(), String> {
    let mut actor = Host::for_test::<RadioMenu>()?;
    actor.dispatch_widget(part("trigger"))?;
    actor.dispatch_widget(item("updated"))?;
    assert_eq!(actor.integer("menu.order"), 1);
    assert_eq!(
        actor.projected_view().bindings[&item("name")]
            .node
            .semantics
            .selected,
        0
    );
    assert_eq!(
        actor.projected_view().bindings[&item("updated")]
            .node
            .semantics
            .selected,
        1
    );
    assert_eq!(
        actor.projected_view().bindings[&item("updated")]
            .node
            .semantics
            .role,
        abi::semantics::MENU_RADIO
    );
    actor.dispatch_widget(item("updated"))?;
    actor.dispatch_widget(item("disabled"))?;
    assert_eq!(actor.integer("menu.order"), 1);
    actor.replace_for_test::<RadioMenu>()?;
    actor.dispatch_widget(item("name"))?;
    assert_eq!(actor.integer("menu.order"), 0);
    assert_eq!(
        actor.projected_view().bindings[&item("updated")]
            .node
            .semantics
            .selected,
        0
    );
    assert_eq!(
        actor.projected_view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        0
    );
    Ok(())
}

struct Grouped;
impl Actor for Grouped {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), LAST.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        DropdownMenu::new(OPEN, "Groups").key("commands").entries(
            cx,
            vec![
                MenuEntry::group(
                    "project",
                    "Project",
                    vec![MenuItem::new("save", "Save", |cx| LAST.set(cx, "Saved"))],
                ),
                MenuEntry::separator("divider"),
                MenuEntry::group(
                    "other",
                    "Other",
                    vec![MenuItem::new("cancel", "Cancel", |_| Ok(()))],
                ),
            ],
        )
    }
}
#[test]
fn named_groups_keep_flat_command_keys_and_current_actions() -> std::result::Result<(), String> {
    let mut actor = Host::for_test::<Grouped>()?;
    actor.dispatch_widget(part("trigger"))?;
    let group = Key::named("commands")
        .scoped(abi::key("group"))
        .scoped(abi::key("project"))
        .0;
    let node = actor.projected_view().bindings[&group].node;
    assert_eq!(node.semantics.role, abi::semantics::MENU_GROUP);
    let title = &actor.projected_view().bindings[&node.semantics.labelled_by];
    assert_eq!(title.text, "Project");
    assert_eq!(title.node.parent, group);
    assert_eq!(title.node.semantics.role, abi::semantics::MENU_LABEL);
    assert!(actor.dispatch_widget(node.semantics.labelled_by).is_err());
    assert_eq!(
        actor.projected_view().bindings[&item("save")].node.parent,
        group
    );
    actor.replace_for_test::<Grouped>()?;
    actor.dispatch_widget(item("save"))?;
    assert_eq!(actor.text("menu.last"), "Saved");
    assert_eq!(
        actor.projected_view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    Ok(())
}
struct DuplicateGrouped;
impl Actor for DuplicateGrouped {
    const STATE: &'static [StateSpec] = &[OPEN.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        DropdownMenu::new(OPEN, "Groups").entries(
            cx,
            vec![
                MenuEntry::group(
                    "one",
                    "One",
                    vec![MenuItem::new("same", "Command", |_| Ok(()))],
                ),
                MenuEntry::group(
                    "two",
                    "Two",
                    vec![MenuItem::new("same", "Other", |_| Ok(()))],
                ),
            ],
        )
    }
}
#[test]
fn command_keys_are_unique_across_groups_even_when_closed() {
    assert!(
        Host::for_test::<DuplicateGrouped>()
            .err()
            .unwrap()
            .contains("distinct nonzero keys")
    );
}

const EXPORT: State<bool> = State::<bool>::new("menu.export", false);
const SHARE: State<bool> = State::<bool>::new("menu.share", false);
struct Nested;
impl Actor for Nested {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), EXPORT.spec(), SHARE.spec(), LAST.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        DropdownMenu::new(OPEN, "Actions").key("commands").entries(
            cx,
            vec![
                MenuEntry::submenu(
                    DropdownMenu::new(EXPORT, "Export").key("export"),
                    vec![
                        MenuItem::new("csv", "CSV", |cx| LAST.set(cx, "Exported CSV")).into(),
                        MenuItem::new("reject", "Rejected export", |cx| {
                            LAST.set(cx, "rolled back")?;
                            Err(Error::Application("export failed".into()))
                        })
                        .into(),
                    ],
                ),
                MenuEntry::submenu(
                    DropdownMenu::new(SHARE, "Share").key("share"),
                    vec![
                        MenuItem::new("copy", "Copy link", |cx| LAST.set(cx, "Link copied")).into(),
                    ],
                ),
            ],
        )
    }
}
fn nested(name: &str) -> Key {
    Key::named("commands")
        .scoped(abi::key("submenu"))
        .scoped(abi::key(name))
}
#[test]
fn submenu_current_commands_close_tree_and_failure_retains_tree() -> std::result::Result<(), String>
{
    let mut actor = Host::for_test::<Nested>()?;
    let trigger = |name| nested(name).scoped(abi::key("trigger")).0;
    let panel = |name| nested(name).scoped(abi::key("panel")).0;
    let command = |name, item| {
        nested(name)
            .scoped(abi::key("item"))
            .scoped(abi::key(item))
            .0
    };
    actor.dispatch_widget(part("trigger"))?;
    actor.dispatch_widget(trigger("export"))?;
    assert!(actor.boolean("menu.open") && actor.boolean("menu.export"));
    actor.dispatch_widget(trigger("export"))?;
    assert!(actor.boolean("menu.export"));
    assert_eq!(
        actor.projected_view().bindings[&trigger("export")]
            .node
            .semantics
            .role,
        abi::semantics::SUBMENU_TRIGGER
    );
    assert!(actor.dispatch_widget(command("export", "reject")).is_err());
    assert_eq!(actor.text("menu.last"), "");
    assert!(actor.boolean("menu.open") && actor.boolean("menu.export"));
    actor.dispatch_widget(trigger("share"))?;
    assert!(!actor.boolean("menu.export"));
    assert!(actor.boolean("menu.share"));
    actor.dismiss_popover(panel("share"))?;
    assert!(actor.boolean("menu.open"));
    assert!(!actor.boolean("menu.share"));
    actor.dispatch_widget(trigger("export"))?;
    actor.replace_for_test::<Nested>()?;
    actor.dispatch_widget(command("export", "csv"))?;
    assert_eq!(actor.text("menu.last"), "Exported CSV");
    assert!(
        !actor.boolean("menu.open")
            && !actor.boolean("menu.export")
            && !actor.boolean("menu.share")
    );
    assert!(actor.dispatch_widget(command("export", "csv")).is_err());
    actor.dispatch_widget(part("trigger"))?;
    assert_eq!(
        actor.projected_view().bindings[&panel("export")]
            .node
            .semantics
            .hidden,
        1
    );
    actor.dispatch_widget(trigger("export"))?;
    actor.dismiss_popover(part("panel"))?;
    assert!(!actor.boolean("menu.open") && !actor.boolean("menu.export"));
    Ok(())
}

struct AliasedSubmenu;
impl Actor for AliasedSubmenu {
    const STATE: &'static [StateSpec] = &[OPEN.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        DropdownMenu::new(OPEN, "Root").entries(
            cx,
            vec![MenuEntry::submenu(DropdownMenu::new(OPEN, "Child"), vec![])],
        )
    }
}
#[test]
fn submenu_cannot_alias_ancestor_open_state() {
    assert!(
        Host::for_test::<AliasedSubmenu>()
            .err()
            .unwrap()
            .contains("distinct open states")
    );
}

const FILE: State<bool> = State::<bool>::new("bar.file", false);
const EDIT: State<bool> = State::<bool>::new("bar.edit", false);
struct Bar;
impl Actor for Bar {
    const STATE: &'static [StateSpec] = &[FILE.spec(), EDIT.spec(), EXPORT.spec(), LAST.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Menubar::new("bar").menus(
            cx,
            vec![
                (
                    DropdownMenu::new(FILE, "File").key("file"),
                    vec![MenuEntry::submenu(
                        DropdownMenu::new(EXPORT, "Export").key("export"),
                        vec![MenuItem::new("csv", "CSV", |cx| LAST.set(cx, "CSV exported")).into()],
                    )],
                ),
                (
                    DropdownMenu::new(EDIT, "Edit").key("edit"),
                    vec![
                        MenuItem::new("copy", "Copy", |cx| LAST.set(cx, "Copied")).into(),
                        MenuItem::new("reject", "Reject", |cx| {
                            LAST.set(cx, "must roll back")?;
                            Err(Error::Application("bar command rejected".into()))
                        })
                        .into(),
                    ],
                ),
            ],
        )
    }
}
fn bar_root(name: &str) -> Key {
    Key::named("bar")
        .scoped(abi::key("menu"))
        .scoped(abi::key(name))
}
#[test]
fn menubar_root_switch_closes_previous_descendants_and_commands_are_atomic()
-> std::result::Result<(), String> {
    let mut actor = Host::for_test::<Bar>()?;
    assert_eq!(
        actor.projected_view().bindings[&Key::named("bar").0]
            .node
            .semantics
            .role,
        abi::semantics::MENUBAR
    );
    let file = bar_root("file");
    let edit = bar_root("edit");
    let export = file.scoped(abi::key("submenu")).scoped(abi::key("export"));
    actor.dispatch_widget(file.scoped(abi::key("trigger")).0)?;
    actor.dispatch_widget(export.scoped(abi::key("trigger")).0)?;
    assert!(actor.boolean("bar.file") && actor.boolean("menu.export"));
    actor.dispatch_widget(edit.scoped(abi::key("trigger")).0)?;
    assert!(!actor.boolean("bar.file") && !actor.boolean("menu.export"));
    assert!(actor.boolean("bar.edit"));
    let command = |name| edit.scoped(abi::key("item")).scoped(abi::key(name)).0;
    assert!(actor.dispatch_widget(command("reject")).is_err());
    assert_eq!(actor.text("menu.last"), "");
    assert!(actor.boolean("bar.edit"));
    actor.replace_for_test::<Bar>()?;
    actor.dispatch_widget(command("copy"))?;
    assert_eq!(actor.text("menu.last"), "Copied");
    assert!(
        !actor.boolean("bar.file") && !actor.boolean("bar.edit") && !actor.boolean("menu.export")
    );
    assert!(actor.dispatch_widget(command("copy")).is_err());
    Ok(())
}
struct AliasedBar;
impl Actor for AliasedBar {
    const STATE: &'static [StateSpec] = &[FILE.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Menubar::new("bar").menus(
            cx,
            vec![
                (DropdownMenu::new(FILE, "File").key("file"), vec![]),
                (DropdownMenu::new(FILE, "Edit").key("edit"), vec![]),
            ],
        )
    }
}
#[test]
fn menubar_rejects_aliasing_between_root_open_states() {
    assert!(
        Host::for_test::<AliasedBar>()
            .err()
            .unwrap()
            .contains("distinct open states")
    );
}

struct ContextFixture;
impl Actor for ContextFixture {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), EXPORT.spec(), LAST.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        ContextMenu::new(OPEN, "Project actions")
            .key("context")
            .entries(
                cx,
                vec![
                    MenuItem::new("copy", "Copy project", |cx| LAST.set(cx, "Copied project"))
                        .into(),
                    MenuItem::new("reject", "Reject", |cx| {
                        LAST.set(cx, "rollback")?;
                        Err(Error::Application("rejected".into()))
                    })
                    .into(),
                    MenuEntry::submenu(DropdownMenu::new(EXPORT, "Export").key("export"), vec![]),
                ],
            )
    }
}
#[test]
fn context_activation_is_idempotent_resets_children_and_commands_close_atomically()
-> std::result::Result<(), String> {
    let mut actor = Host::for_test::<ContextFixture>()?;
    let root = Key::named("context");
    let trigger = root.scoped(abi::key("trigger")).0;
    let export = root
        .scoped(abi::key("submenu"))
        .scoped(abi::key("export"))
        .scoped(abi::key("trigger"))
        .0;
    let command = |name| root.scoped(abi::key("item")).scoped(abi::key(name)).0;
    assert_eq!(
        actor.projected_view().bindings[&trigger]
            .node
            .semantics
            .role,
        abi::semantics::CONTEXT_TRIGGER
    );
    actor.dispatch_widget(trigger)?;
    actor.dispatch_widget(export)?;
    assert!(actor.boolean("menu.export"));
    actor.dispatch_widget(trigger)?;
    assert!(actor.boolean("menu.open") && !actor.boolean("menu.export"));
    assert!(actor.dispatch_widget(command("reject")).is_err());
    assert_eq!(actor.text("menu.last"), "");
    assert!(actor.boolean("menu.open"));
    actor.replace_for_test::<ContextFixture>()?;
    actor.dispatch_widget(command("copy"))?;
    assert_eq!(actor.text("menu.last"), "Copied project");
    assert!(!actor.boolean("menu.open"));
    assert!(actor.dispatch_widget(command("copy")).is_err());
    Ok(())
}
