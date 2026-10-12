use std::{cell::Cell, rc::Rc};
use ui_contract::{Deserialize, Serialize};
use wabou_native::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "ui_contract::serde")]
struct LoadCount;
impl ServiceRequest for LoadCount {
    type Response = i64;
}

#[app]
struct Counter {
    count: State<i64>,
}
#[app_impl]
impl Counter {
    #[action]
    fn load(&self) -> Result<()> {
        self.count.set(self.call(LoadCount)?)
    }
    fn view(&self) -> View {
        #[ui]
        {
            Column::default();
            #[ui]
            Text::new(live!(format!("Count: {}", self.count.get()))).key("value");
            #[ui]
            Button::new("Load", self.load_action()).key("load");
        }
    }
}

#[test]
fn linked_native_services_use_host_handlers_and_preserve_failed_turns()
-> std::result::Result<(), String> {
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    let mut services = native::Services::default();
    services.handle::<LoadCount>(move |_| {
        observed.set(observed.get() + 1);
        Ok(42)
    })?;
    let mut source = native::mount::<Counter>()?;
    source.actor.set_services(services);
    assert_eq!(calls.get(), 0);
    source.actor.dispatch_widget(abi::key("load"))?;
    assert_eq!(calls.get(), 1);
    let view = source.actor.projected_view();
    let text = view
        .bindings
        .get(&abi::key("value"))
        .ok_or("missing text")?;
    assert_eq!(text.text, "Count: 42");
    let evaluations = text.evaluations;
    let roots = view.root_evaluations();
    let mut failing = native::Services::default();
    failing.handle::<LoadCount>(|_| Err("offline".into()))?;
    source.actor.set_services(failing);
    assert!(source.actor.dispatch_widget(abi::key("load")).is_err());
    let view = source.actor.projected_view();
    let text = view
        .bindings
        .get(&abi::key("value"))
        .ok_or("missing text")?;
    assert_eq!(text.text, "Count: 42");
    assert_eq!(text.evaluations, evaluations);
    assert_eq!(view.root_evaluations(), roots);
    Ok(())
}
