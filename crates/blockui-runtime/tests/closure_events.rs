#![cfg(feature = "testing")]
use blockui::*;
use std::{cell::Cell, rc::Rc};
thread_local! { static DROPS: Rc<Cell<usize>> = Default::default(); }

const DRAFT: State<String> = State::<String>::new("input.draft");
const STATUS: State<String> = State::<String>::new("input.status");
const LEVEL: State<i64> = State::<i64>::new("control.level", 0);
const ENABLED: State<bool> = State::<bool>::new("control.enabled", false);
struct Controls;
impl Actor for Controls {
    const STATE: &'static [StateSpec] = &[LEVEL.spec(), ENABLED.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        let slider = Slider::with_handler(
            LEVEL,
            IntegerRange {
                min: 0,
                max: 10,
                step: 1,
            },
            move |cx, value| {
                LEVEL.set(cx, value)?;
                if value == 5 {
                    return Err(Error::Application("rejected level".into()));
                }
                Ok(())
            },
        )
        .key("controls.slider");
        let select = Select::with_handler(
            "Level",
            LEVEL,
            vec![
                SelectOption {
                    value: 0,
                    label: "Zero".into(),
                    enabled: true,
                },
                SelectOption {
                    value: 2,
                    label: "Two".into(),
                    enabled: true,
                },
                SelectOption {
                    value: 3,
                    label: "Three".into(),
                    enabled: true,
                },
            ],
            move |cx, value| LEVEL.set(cx, value),
        )
        .key("controls.select");
        let checkbox = Checkbox::with_handler("Enabled", ENABLED, move |cx, value| {
            ENABLED.set(cx, value)?;
            if !value {
                return Err(Error::Application("must stay enabled".into()));
            }
            Ok(())
        });
        let checkbox = checkbox.key("superseded.checkbox").key("controls.checkbox");
        let slider = Field::new("Slider level", slider);
        let select = Field::new("Selected level", select);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(slider).__local_identity(stringify!(slider)),
                );
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(select).__local_identity(stringify!(select)),
                );
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(checkbox).__local_identity(stringify!(checkbox)),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn scalar_closures_validate_current_controls_and_rollback() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Controls>()?;
    let id = |kind| {
        actor
            .view
            .bindings
            .values()
            .find(|b| b.node.kind == kind)
            .unwrap()
            .node
            .key
    };
    let slider = id(abi::SLIDER);
    let select = id(abi::SELECT);
    let checkbox = id(abi::CHECKBOX);
    assert_eq!(slider, Key::named("controls.slider").0);
    assert_eq!(select, Key::named("controls.select").0);
    assert_eq!(checkbox, Key::named("controls.checkbox").0);
    assert!(
        !actor
            .view
            .bindings
            .contains_key(&Key::named("superseded.checkbox").0)
    );
    actor.dispatch_widget(checkbox)?;
    assert!(actor.boolean("control.enabled"));
    assert!(actor.dispatch_widget(checkbox).is_err());
    assert!(actor.boolean("control.enabled"));
    actor.edit_slider(slider, 3)?;
    assert_eq!(actor.integer("control.level"), 3);
    assert!(actor.edit_slider(slider, 5).is_err());
    assert!(actor.edit_slider(slider, 11).is_err());
    assert_eq!(actor.integer("control.level"), 3);
    assert!(actor.edit_select(select, 9).is_err());
    actor.edit_select(select, 2)?;
    assert_eq!(actor.integer("control.level"), 2);
    Ok(())
}
struct InputUi<const REVISION: i64>;
impl<const REVISION: i64> Actor for InputUi<REVISION> {
    const STATE: &'static [StateSpec] = &[DRAFT.spec(), STATUS.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        let edit = move |cx: &Cx<'_>, text: String| {
            assert_eq!(DRAFT.get(cx)?, text);
            STATUS.set(cx, &format!("revision {REVISION}: {text}"))?;
            if text == "reject" {
                return Err(Error::Application("invalid draft".into()));
            }
            Ok(())
        };
        let editor = Field::new(
            "Draft",
            TextInput::new(DRAFT)
                .key("old.editor")
                .on_input(edit)
                .key("controls.editor"),
        );
        let multiline = Field::new(
            "Multiline draft",
            TextArea::new(DRAFT)
                .on_input(edit)
                .key("controls.multiline"),
        );
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(editor).__local_identity(stringify!(editor)),
                );
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(multiline)
                        .__local_identity(stringify!(multiline)),
                );
                __blockui_tree.add(Text::new(STATUS));
            }
            __blockui_tree.finish()
        })
    }
}

#[test]
fn text_closures_receive_current_text_rollback_and_use_new_code_after_replacement()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<InputUi<1>>()?;
    let widget = |actor: &blockui_runtime::Actor, kind| {
        actor
            .view
            .bindings
            .values()
            .find(|binding| binding.node.kind == kind)
            .unwrap()
            .node
            .key
    };
    let editor = widget(&actor, abi::ENTRY);
    let multiline = widget(&actor, abi::TEXTAREA);
    assert_eq!(editor, Key::named("controls.editor").0);
    assert_eq!(multiline, Key::named("controls.multiline").0);
    assert!(
        !actor
            .view
            .bindings
            .contains_key(&Key::named("old.editor").0)
    );
    actor.edit_text(editor, "Hello 🦀")?;
    assert_eq!(actor.text("input.status"), "revision 1: Hello 🦀");
    assert!(actor.edit_text(editor, "reject").is_err());
    assert_eq!(actor.text("input.draft"), "Hello 🦀");
    assert_eq!(actor.text("input.status"), "revision 1: Hello 🦀");
    let slot = actor.slot_address("input.draft");
    actor.replace_for_test::<InputUi<2>>()?;
    assert_eq!(actor.slot_address("input.draft"), slot);
    assert_eq!(widget(&actor, abi::ENTRY), editor);
    assert_eq!(widget(&actor, abi::TEXTAREA), multiline);
    actor.edit_text(multiline, "two\nlines")?;
    assert_eq!(actor.text("input.status"), "revision 2: two\nlines");
    assert!(actor.edit_text(multiline, "reject").is_err());
    assert_eq!(actor.text("input.draft"), "two\nlines");
    Ok(())
}
struct Capture(Rc<Cell<usize>>);
impl Drop for Capture {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
struct Ui<const STEP: i64, const INVALID: bool>;
const COUNT: State<i64> = State::<i64>::new("closure.count", 0);
impl<const STEP: i64, const INVALID: bool> Actor for Ui<STEP, INVALID> {
    const STATE: &'static [StateSpec] = &[COUNT.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let capture = Capture(DROPS.with(Clone::clone));
        let button = Button::label(format!("Increment {}", COUNT.get(cx)?)).on_click(move |cx| {
            let _ = &capture;
            COUNT.update(cx, |n| Ok(n + STEP))
        });
        if INVALID {
            return Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    {
                        let _scope_0 = __blockui_tree.scope(Row::default());
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(button)
                                .__local_identity(stringify!(button)),
                        );
                        __blockui_tree.add(Text::new("first"));
                        __blockui_tree.add(Text::new("second"));
                    }
                }
                __blockui_tree.finish()
            });
        }
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(COUNT));
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(button).__local_identity(stringify!(button)),
                    );
                }
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn closures_replace_transactionally_and_release_before_implementation_disposal()
-> std::result::Result<(), String> {
    DROPS.with(|count| count.set(0));
    let mut actor = blockui_runtime::Actor::for_test::<Ui<1, false>>()?;
    let button = actor
        .view
        .bindings
        .values()
        .find(|b| b.node.kind == abi::BUTTON)
        .unwrap()
        .node
        .key;
    actor.dispatch_widget(button)?;
    assert_eq!(actor.integer("closure.count"), 1);
    assert_eq!(
        DROPS.with(|n| n.get()),
        1,
        "successful projection disposes old capture"
    );
    assert!(actor.replace_for_test::<Ui<100, true>>().is_err());
    assert_eq!(
        DROPS.with(|n| n.get()),
        2,
        "rejected candidate releases its capture"
    );
    actor.dispatch_widget(button)?;
    assert_eq!(
        actor.integer("closure.count"),
        2,
        "old callback stays installed after rejected view"
    );
    let slot = actor.slot_address("closure.count");
    actor.mailbox.push_back(
        blockui_runtime::Message {
            action: actor.view.bindings[&button].node.action,
            value: 0,
        }
        .into(),
    );
    actor.replace_for_test::<Ui<10, false>>()?;
    actor.tick()?;
    assert_eq!(
        actor.integer("closure.count"),
        12,
        "queued event resolves the new closure"
    );
    assert_eq!(actor.slot_address("closure.count"), slot);
    actor.dispatch_widget(button)?;
    assert_eq!(actor.integer("closure.count"), 22);
    let drops = DROPS.with(|n| n.get());
    drop(actor);
    assert_eq!(
        DROPS.with(|n| n.get()),
        drops + 1,
        "last implementation releases its live capture"
    );
    Ok(())
}

#[test]
fn same_component_in_separate_roots_cannot_share_or_dispose_each_others_callbacks()
-> std::result::Result<(), String> {
    let mut left = blockui_runtime::Actor::for_test::<Ui<1, false>>()?;
    let mut right = blockui_runtime::Actor::for_test::<Ui<10, false>>()?;
    let button = left
        .view
        .bindings
        .values()
        .find(|b| b.node.kind == abi::BUTTON)
        .unwrap()
        .node
        .key;
    left.dispatch_widget(button)?;
    right.dispatch_widget(button)?;
    assert_eq!(left.integer("closure.count"), 1);
    assert_eq!(right.integer("closure.count"), 10);
    drop(left);
    right.dispatch_widget(button)?;
    assert_eq!(right.integer("closure.count"), 20);
    Ok(())
}

struct Scoped;
const OTHER: State<i64> = State::<i64>::new("closure.other", 0);
const SHOW: State<bool> = State::<bool>::new("closure.show", true);
const BREAK: Action = Action::new("closure.break", |cx, _| COUNT.set(cx, -1));
const HIDE: Action = Action::new("closure.hide", |cx, _| SHOW.set(cx, false));
impl Actor for Scoped {
    const STATE: &'static [StateSpec] = &[COUNT.spec(), OTHER.spec(), SHOW.spec()];
    const ACTIONS: &'static [Action] = &[HIDE, BREAK];
    const SCOPES: &'static [Scope] = &[
        Scope::new("left", |cx| {
            if COUNT.get(cx)? < 0 {
                return Err(Error::Application("scope projection rejected".into()));
            }
            let button = Button::label(format!("Left {}", COUNT.get(cx)?))
                .on_click(|cx| COUNT.update(cx, |n| Ok(n + 1)));
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(button)
                                .__local_identity(stringify!(button)),
                        );
                    }
                }
                __blockui_tree.finish()
            })
        }),
        Scope::new("right", |cx| {
            let button = Button::label(format!("Right {}", OTHER.get(cx)?))
                .on_click(|cx| OTHER.update(cx, |n| Ok(n + 10)));
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default());
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(button)
                                .__local_identity(stringify!(button)),
                        );
                    }
                }
                __blockui_tree.finish()
            })
        }),
    ];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Self::SCOPES[0]);
                    if SHOW.get(cx)? {
                        __blockui_tree.add(Self::SCOPES[1]);
                    }
                }
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn cached_scope_callbacks_survive_sibling_updates_and_retire_when_removed()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Scoped>()?;
    let find = |actor: &blockui_runtime::Actor, prefix: &str| {
        actor
            .view
            .bindings
            .values()
            .find(|b| b.node.kind == abi::BUTTON && b.text.starts_with(prefix))
            .unwrap()
            .node
    };
    let left = find(&actor, "Left");
    let right = find(&actor, "Right");
    actor.dispatch_widget(left.key)?;
    actor.dispatch_widget(right.key)?;
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: BREAK.id,
                value: 0
            })
            .is_err()
    );
    assert_eq!(actor.integer("closure.count"), 1);
    actor.dispatch_widget(left.key)?;
    assert_eq!(actor.integer("closure.count"), 2);
    assert_eq!(actor.integer("closure.other"), 10);
    actor.dispatch(blockui_runtime::Message {
        action: HIDE.id,
        value: 0,
    })?;
    assert!(!actor.view.bindings.contains_key(&right.key));
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: right.action,
                value: 0
            })
            .is_err()
    );
    actor.dispatch_widget(left.key)?;
    assert_eq!(actor.integer("closure.count"), 3);
    Ok(())
}
