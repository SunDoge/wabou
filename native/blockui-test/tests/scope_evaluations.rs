use blockui::*;
use blockui_test::Screen;
const ROOT: State<bool> = State::<bool>::new("build.root", false);
const OUTER: State<bool> = State::<bool>::new("build.outer", false);
const INNER: State<bool> = State::<bool>::new("build.inner", false);
const MOUNT: State<bool> = State::<bool>::new("build.mount", true);
const REJECT: State<bool> = State::<bool>::new("build.reject", false);
const STABLE_VALUE: State<i64> = State::<i64>::new("build.stable.value", 0);
const STABLE: Scope = Scope::new("build.stable", |_| {
    Ok(Column::default().children(vec![
        Text::new("Cached static label")
            .key("build.stable.static")
            .into_view(),
        Text::new(STABLE_VALUE).key("build.stable.text").into_view(),
        Button::label("Stable increment")
            .key("build.stable.increment")
            .on_click(|cx| STABLE_VALUE.update(cx, |n| Ok(n + 1))),
    ]))
});
const PANEL: Scope = Scope::new("build.panel", |cx| {
    let label = if OUTER.get(cx)? {
        "Outer changed"
    } else {
        "Outer"
    };
    Ok(Column::default().children(vec![
        Text::new(label).key("build.label").into_view(),
        STABLE.into_view(),
        Scope::reactive("build.nested", |cx| {
            if REJECT.get(cx)? {
                return Err(Error::Application("reject builder".into()));
            }
            Ok(Column::default().children(vec![
                Text::new(if INNER.get(cx)? {
                    "Inner changed"
                } else {
                    "Inner"
                })
                .into_view(),
            ]))
        }),
    ]))
});
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] = &[
        ROOT.spec(),
        OUTER.spec(),
        INNER.spec(),
        MOUNT.spec(),
        REJECT.spec(),
        STABLE_VALUE.spec(),
    ];
    const ACTIONS: &'static [Action] = &[];
    const SCOPES: &'static [Scope] = &[PANEL, STABLE];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let mut children = vec![
            Text::new(if ROOT.get(cx)? {
                "Root changed"
            } else {
                "Root"
            })
            .key("build.root.label")
            .into_view(),
        ];
        if MOUNT.get(cx)? {
            children.push(PANEL.into_view());
        }
        for (name, state) in [
            ("root", ROOT),
            ("outer", OUTER),
            ("inner", INNER),
            ("mount", MOUNT),
            ("reject", REJECT),
        ] {
            children.push(
                Button::label(name)
                    .key(format!("change.{name}"))
                    .on_click(move |cx| state.set(cx, !state.get(cx)?)),
            );
        }
        Ok(Column::default().children(children))
    }
}
#[test]
fn scope_counts_follow_cache_refresh_rollback_and_remount() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<App>()?;
    let panel = screen.get_by_key("build.panel")?;
    let nested = screen.scope_of(screen.get_by_text("Inner")?)?;
    let stable = screen.get_by_key("build.stable")?;
    let stable_text = screen.get_by_key("build.stable.text")?;
    let stable_text_before = screen.evaluations(stable_text)?;
    let static_label = screen.get_by_key("build.stable.static")?;
    let static_before = screen.evaluations(static_label)?;
    assert_eq!(
        screen.scope_dependencies(panel)?,
        std::collections::BTreeSet::from([blockui_test::Dependency::State(OUTER.key())])
    );
    assert!(screen.scope_dependencies(stable)?.is_empty());
    assert_eq!(
        screen.scope_dependencies(nested)?,
        std::collections::BTreeSet::from([
            blockui_test::Dependency::State(INNER.key()),
            blockui_test::Dependency::State(REJECT.key()),
        ])
    );
    assert!(screen.scope_dependencies(stable_text).is_err());
    assert_eq!(screen.scope_evaluations(stable)?, 1);
    assert_eq!(screen.scope_evaluations(panel)?, 1);
    assert_eq!(screen.scope_evaluations(nested)?, 1);
    assert_eq!(
        screen.root_dependencies(),
        std::collections::BTreeSet::from([
            blockui_test::Dependency::State(ROOT.key()),
            blockui_test::Dependency::State(MOUNT.key()),
        ])
    );
    let tree = screen.debug_tree();
    let root_line = tree.lines().next().unwrap();
    assert!(root_line.contains("root-deps=["), "{tree}");
    assert!(
        root_line.contains(&format!("state:{:#018x}", ROOT.key())),
        "{tree}"
    );
    assert!(
        !root_line.contains(&format!("state:{:#018x}", OUTER.key())),
        "{tree}"
    );
    assert!(root_line.contains(" root=1"), "{tree}");
    let tree = screen.within(nested)?.debug_tree();
    assert!(tree.lines().next().unwrap().ends_with(" scope=1"), "{tree}");
    assert!(
        !tree.contains("root-deps="),
        "subtree must not repeat root subscriptions: {tree}"
    );
    assert!(
        tree.contains(&format!("state:{:#018x}", INNER.key())),
        "{tree}"
    );
    assert!(
        tree.contains(&format!("state:{:#018x}", REJECT.key())),
        "{tree}"
    );
    assert!(
        !tree.contains(&format!("state:{:#018x}", OUTER.key())),
        "{tree}"
    );
    assert_eq!(screen.scope_evaluations(nested)?, 1);
    assert_eq!(screen.root_evaluations(), 1);
    assert!(
        screen
            .scope_evaluations(screen.get_by_key("build.root.label")?)
            .is_err()
    );
    screen.click(screen.get_by_key("change.root")?)?;
    assert_eq!(
        screen.scope_evaluations(panel)?,
        1,
        "root rebuild reuses cached scope"
    );
    assert_eq!(screen.scope_evaluations(nested)?, 1);
    assert_eq!(
        screen.evaluations(static_label)?,
        static_before,
        "cached root refresh retains static bindings"
    );
    screen.click(screen.get_by_key("change.inner")?)?;
    assert_eq!(screen.scope_evaluations(panel)?, 1);
    assert_eq!(screen.scope_evaluations(nested)?, 2);
    let tree = screen.debug_tree();
    assert!(tree.lines().next().unwrap().contains(" root=2"), "{tree}");
    let tree = screen.within(nested)?.debug_tree();
    assert!(tree.lines().next().unwrap().ends_with(" scope=2"), "{tree}");
    assert!(screen.click(screen.get_by_key("change.reject")?).is_err());
    assert_eq!(
        screen.within(nested)?.debug_tree(),
        tree,
        "failed turn does not publish diagnostic counters"
    );
    assert_eq!(
        screen.scope_evaluations(nested)?,
        2,
        "failed builder count is not published"
    );
    screen.click(screen.get_by_key("change.outer")?)?;
    assert_eq!(screen.scope_evaluations(panel)?, 2);
    assert_eq!(
        screen.scope_evaluations(stable)?,
        1,
        "unchanged declared descendant reuses cache"
    );
    assert_eq!(screen.evaluations(stable_text)?, stable_text_before);
    assert_eq!(
        screen.evaluations(static_label)?,
        static_before,
        "cached local refresh retains static bindings"
    );
    screen.click(screen.get_by_key("build.stable.increment")?)?;
    assert_eq!(screen.text(stable_text)?, "1");
    assert_eq!(screen.scope_evaluations(stable)?, 1);
    let nested = screen.scope_of(screen.get_by_text("Inner changed")?)?;
    assert_eq!(
        screen.scope_evaluations(nested)?,
        3,
        "new nested capture reruns builder"
    );
    screen.replace::<App>()?;
    let panel = screen.get_by_key("build.panel")?;
    assert_eq!(
        screen.scope_evaluations(panel)?,
        1,
        "replacement starts fresh counters"
    );
    screen.click(screen.get_by_key("change.mount")?)?;
    assert!(screen.scope_evaluations(panel).is_err());
    assert!(screen.scope_dependencies(panel).is_err());
    screen.click(screen.get_by_key("change.mount")?)?;
    assert_eq!(
        screen.scope_evaluations(screen.get_by_key("build.panel")?)?,
        1
    );
    Ok(())
}
