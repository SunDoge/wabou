#![cfg(feature = "testing")]
use blockui::*;
use std::{cell::Cell, rc::Rc};
#[derive(ui_contract::Serialize, ui_contract::Deserialize)]
#[serde(crate = "ui_contract::serde")]
struct Increment {
    amount: i64,
}
impl ui_contract::Request for Increment {
    type Response = i64;
}

#[app]
struct Ui {
    count: State<i64>,
}
#[app_impl]
impl Ui {
    #[action]
    fn increment(&self) -> Result<()> {
        let number = self.call(Increment { amount: 1 })?;
        self.count.set(number)
    }
    #[action]
    fn reject(&self) -> Result<()> {
        self.call(Increment { amount: 0 })?;
        self.count.set(999)
    }
    fn view(&self) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(&self.count));
                    __blockui_tree.add(Button::new("Increment", self.events().increment));
                }
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn business_object_survives_ui_replacement_without_state_conversion()
-> std::result::Result<(), String> {
    let model = Rc::new(Cell::new(40_i64));
    let mut services = blockui_runtime::application::Services::default();
    let business = model.clone();
    services.handle::<Increment>(move |request| {
        if request.amount == 0 {
            return Err("business validation failed".into());
        }
        business.set(business.get() + request.amount);
        Ok(business.get())
    })?;
    assert!(services.handle::<Increment>(|_| Ok(0)).is_err());
    let mut ui = blockui_runtime::Actor::for_test::<Ui>()?;
    ui.set_services(services);
    let descriptor = Ui::new();
    ui.tell(descriptor.events().increment, 0)?;
    ui.tick()?;
    assert_eq!(model.get(), 41);
    assert_eq!(ui.integer_by_key(descriptor.count.key()), 41);
    let error = ui
        .dispatch(blockui_runtime::Message {
            action: descriptor.events().reject.id,
            value: 0,
        })
        .unwrap_err();
    assert!(error.contains("business validation failed"), "{error}");
    assert_eq!(ui.integer_by_key(descriptor.count.key()), 41);
    let address = Rc::as_ptr(&model);
    ui.replace_for_test::<Ui>()?;
    ui.tell(descriptor.events().increment, 0)?;
    ui.tick()?;
    assert_eq!(Rc::as_ptr(&model), address);
    assert_eq!(model.get(), 42);
    assert_eq!(ui.integer_by_key(descriptor.count.key()), 42);
    Ok(())
}
