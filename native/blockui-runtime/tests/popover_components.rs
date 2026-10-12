#![cfg(feature = "testing")]
use blockui::*;
use blockui_runtime::Actor as Host;
const OPEN: State<bool> = State::<bool>::new("popover.open", false);
const DRAFT: State<String> = State::<String>::new("popover.draft");
struct Fixture;
impl Actor for Fixture {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), DRAFT.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Popover::new(OPEN, "Settings")
            .key("settings")
            .children(cx, vec![TextInput::new(DRAFT).key("draft").into_view()])
    }
}
fn part(name: &str) -> u64 {
    Key::named("settings").scoped(abi::key(name)).0
}
#[test]
fn controlled_overlay_unmounts_content_and_preserves_draft() -> std::result::Result<(), String> {
    let mut a = Host::for_test::<Fixture>()?;
    assert_eq!(
        a.projected_view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    assert!(!a.projected_view().bindings.contains_key(&abi::key("draft")));
    assert!(a.dismiss_popover(part("panel")).is_err());
    a.dispatch_widget(part("trigger"))?;
    assert_eq!(
        a.projected_view().bindings[&part("trigger")]
            .node
            .semantics
            .expanded,
        2
    );
    a.edit_text(abi::key("draft"), "草稿 🦀")?;
    a.dismiss_popover(part("panel"))?;
    assert!(!a.projected_view().bindings.contains_key(&abi::key("draft")));
    assert!(a.edit_text(abi::key("draft"), "stale").is_err());
    assert!(a.dismiss_popover(part("trigger")).is_err());
    a.dispatch_widget(part("trigger"))?;
    assert_eq!(
        a.projected_view().bindings[&abi::key("draft")].text,
        "草稿 🦀"
    );
    a.dispatch_widget(part("trigger"))?;
    assert_eq!(
        a.projected_view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    Ok(())
}
