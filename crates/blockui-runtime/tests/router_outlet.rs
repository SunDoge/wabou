#![cfg(feature = "testing")]
use blockui::*;

#[derive(Clone, Default, PartialEq, Eq, RoutePath)]
enum Page {
    #[default]
    #[route(path = "/")]
    Home,
    #[route(path = "/documents/{id}", guard = id > 0)]
    Document { id: i64 },
}
// A separate finite route enum satisfies the Copy contract.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Finite {
    Home,
    Settings,
}
impl Route for Finite {
    fn id(self) -> i64 {
        match self {
            Self::Home => 0,
            Self::Settings => 1,
        }
    }
    fn from_id(id: i64) -> Option<Self> {
        match id {
            0 => Some(Self::Home),
            1 => Some(Self::Settings),
            _ => None,
        }
    }
}

#[actor(name = "outlet.paths")]
struct Paths {
    #[state(route = Page)]
    history: State<Vec<String>>,
}
#[actor_impl]
impl Paths {
    #[action]
    fn open(&self, cx: &Cx<'_>, id: i64) -> Result<()> {
        PathRouter::new(self.history)
            .bind(cx)
            .push(Page::Document { id })
    }
    #[action]
    fn back(&self, cx: &Cx<'_>) -> Result<()> {
        PathRouter::<Page>::new(self.history).bind(cx).back()?;
        Ok(())
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        PathRouter::<Page>::new(self.history)
            .bind(cx)
            .outlet("pages", |page| {
                let text = match page {
                    Page::Home => "Home".into(),
                    Page::Document { id: 13 } => {
                        return Err(Error::Application("broken page".into()));
                    }
                    Page::Document { id } => format!("Document {id}"),
                };
                Ok(Text::new(text).into_view())
            })
    }
}
#[actor(name = "outlet.finite")]
struct FinitePages {
    current: State<i64>,
    back: State<Vec<i64>>,
    forward: State<Vec<i64>>,
}
#[actor_impl]
impl FinitePages {
    #[action]
    fn settings(&self, cx: &Cx<'_>) -> Result<()> {
        Router::new(self.current, self.back, self.forward)
            .bind(cx)
            .push(Finite::Settings)
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        Router::<Finite>::new(self.current, self.back, self.forward)
            .bind(cx)
            .outlet("finite", |page| {
                Ok(Text::new(match page {
                    Finite::Home => "Home",
                    Finite::Settings => "Settings",
                })
                .into_view())
            })
    }
}
fn dispatch(
    actor: &mut blockui_runtime::Actor,
    action: u64,
    value: i64,
) -> std::result::Result<(), String> {
    actor.dispatch(blockui_runtime::Message { action, value })
}
#[test]
fn parameterized_outlets_restore_keys_and_preserve_history_allocations()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Paths>()?;
    let home = actor.view.order[0];
    assert_eq!(home, Key::named("pages").scoped_named("/").0);
    assert_eq!(actor.view.bindings[&home].node.focus_boundary, 1);
    dispatch(&mut actor, Paths::<DescriptorMode>::open_action().id, 42)?;
    let document = actor.view.order[0];
    let buffer = actor.text_list("outlet.paths.history").as_ptr();
    assert_ne!(home, document);
    actor.replace_for_test::<Paths>()?;
    assert_eq!(actor.view.order[0], document);
    assert_eq!(actor.text_list("outlet.paths.history").as_ptr(), buffer);
    dispatch(&mut actor, Paths::<DescriptorMode>::open_action().id, 73)?;
    assert_ne!(actor.view.order[0], document);
    dispatch(&mut actor, Paths::<DescriptorMode>::back_action().id, 0)?;
    assert_eq!(actor.view.order[0], document);
    Ok(())
}
#[test]
fn failed_page_projection_rolls_back_navigation_and_keeps_old_view()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Paths>()?;
    dispatch(&mut actor, Paths::<DescriptorMode>::open_action().id, 42)?;
    let key = actor.view.order[0];
    let history = actor.text_list("outlet.paths.history").to_vec();
    let buffer = actor.text_list("outlet.paths.history").as_ptr();
    assert!(dispatch(&mut actor, Paths::<DescriptorMode>::open_action().id, 13).is_err());
    assert_eq!(actor.view.order[0], key);
    assert_eq!(actor.text_list("outlet.paths.history"), history);
    assert_eq!(actor.text_list("outlet.paths.history").as_ptr(), buffer);
    Ok(())
}
#[test]
fn finite_outlet_uses_stable_route_id_and_focus_boundary() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<FinitePages>()?;
    dispatch(
        &mut actor,
        FinitePages::<DescriptorMode>::settings_action().id,
        0,
    )?;
    let key = actor.view.order[0];
    assert_eq!(key, Key::named("finite").scoped(1).0);
    assert_eq!(actor.view.bindings[&key].node.focus_boundary, 1);
    actor.replace_for_test::<FinitePages>()?;
    assert_eq!(actor.view.order[0], key);
    Ok(())
}
