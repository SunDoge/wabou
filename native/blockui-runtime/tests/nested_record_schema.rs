#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record)]
#[record(name = "nested.address")]
struct Address {
    city: String,
}
#[derive(Record)]
#[record(name = "nested.contact")]
struct Contact {
    name: String,
    #[record(nested)]
    address: Address,
    #[record(nested)]
    backup: Option<Address>,
    #[record(nested)]
    locations: Vec<Address>,
    tags: Vec<String>,
}
#[derive(Record)]
#[record(name = "nested.address", previous = Address)]
struct NewAddress {
    city: String,
    #[record(default)]
    verified: bool,
}
#[derive(Record)]
#[record(name = "nested.contact", previous = Contact)]
struct NewContact {
    name: String,
    #[record(nested)]
    address: Address,
    #[record(nested, default = previous.backup.map(NewAddress::upgrade).transpose()?)]
    backup: Option<NewAddress>,
    #[record(nested, default = previous.locations.into_iter().map(NewAddress::upgrade).collect::<Result<Vec<_>>>()?)]
    locations: Vec<NewAddress>,
    tags: Vec<String>,
}
#[actor]
struct Original {
    #[state(key = "contacts", codec = Contact)]
    contacts: State<Vec<String>>,
    #[state(key = "draft")]
    draft: State<String>,
}
#[actor_impl]
impl Original {
    #[action(name = "clear_backup")]
    fn clear_backup(&self, cx: &Cx<'_>) -> Result<()> {
        let map = RecordMap::<Contact>::new(self.contacts);
        let mut contact = map
            .get(cx, "42")?
            .ok_or_else(|| Error::Application("Missing contact".into()))?;
        contact.backup = None;
        contact.locations.clear();
        map.insert(cx, "42", &contact)
    }

    #[action(name = "put")]
    fn put(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<Contact>::new(self.contacts).insert(
            cx,
            "42",
            &Contact {
                name: "Ada 🦀".into(),
                address: Address {
                    city: "香港\n東京".into(),
                },
                backup: Some(Address {
                    city: "Backup 🦀".into(),
                }),
                locations: vec![
                    Address {
                        city: "First 香港".into(),
                    },
                    Address {
                        city: "Second 🦀".into(),
                    },
                ],
                tags: vec![String::new(), "🦀\nタグ".into()],
            },
        )?;
        self.draft.set(cx, "Unchanged draft")
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Original").key("status"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct Missing {
    #[state(key = "contacts", codec = NewContact)]
    contacts: State<Vec<String>>,
    #[state(key = "draft")]
    draft: State<String>,
}
#[actor_impl]
impl Missing {
    // Explicit migration overrides the derived predecessor. A no-op must fail
    // validation even when this view does not inspect the collection.
    fn migrate(&self, _: &Cx<'_>, _: u64) -> Result<()> {
        Ok(())
    }
    fn view(&self, _: &Cx<'_>) -> Result<View> {
        // Admission must catch a child schema change even without decoding records.
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Missing migration").key("status"));
            }
            __blockui_tree.finish()
        })
    }
}
#[actor]
struct Upgraded {
    #[state(key = "contacts", codec = NewContact)]
    contacts: State<Vec<String>>,
    #[state(key = "draft")]
    draft: State<String>,
}
#[actor_impl]
impl Upgraded {
    fn migrate(&self, cx: &Cx<'_>, previous: u64) -> Result<()> {
        if previous != 0 {
            RecordMap::<NewContact>::new(self.contacts).upgrade_previous::<Contact>(cx)?;
        }
        Ok(())
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let contact = RecordMap::<NewContact>::new(self.contacts)
            .get(cx, "42")?
            .ok_or_else(|| Error::Application("Missing contact".into()))?;
        if contact.tags != ["", "🦀\nタグ"] {
            return Err(Error::Application("Lost scalar tags".into()));
        }
        if contact
            .locations
            .iter()
            .map(|address| address.city.as_str())
            .collect::<Vec<_>>()
            != ["First 香港", "Second 🦀"]
            || contact.locations.iter().any(|address| address.verified)
        {
            return Err(Error::Application(
                "Lost nested locations or defaults".into(),
            ));
        }
        if contact
            .backup
            .as_ref()
            .is_none_or(|address| address.city != "Backup 🦀")
        {
            return Err(Error::Application("Lost nested backup".into()));
        }
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Text::new(format!(
                        "{} · {} · {}",
                        contact.name,
                        contact.address.city,
                        contact
                            .backup
                            .as_ref()
                            .is_some_and(|address| address.verified)
                    ))
                    .key("status"),
                );
            }
            __blockui_tree.finish()
        })
    }
}

#[actor]
struct Automatic {
    #[state(key = "contacts", codec = NewContact)]
    contacts: State<Vec<String>>,
    #[state(key = "draft")]
    draft: State<String>,
}
#[actor_impl]
impl Automatic {
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        Upgraded::new().view(cx)
    }
}

#[test]
fn nested_child_change_requires_migration_and_preserves_actor_identity()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Original>()?;
    actor.tell(Original::put_action(), 0)?;
    actor.tick()?;
    actor.tell(Original::clear_backup_action(), 0)?;
    actor.tick()?;
    let state = actor.state_address();
    let slot = actor.slot_address("contacts");
    let buffer = actor.text_list("contacts").as_ptr();
    let schema = actor.schema_version;
    assert!(actor.replace_for_test::<Missing>().is_err());
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text_list("contacts").as_ptr(), buffer);
    actor.tell(Original::put_action(), 0)?;
    actor.tick()?;
    let draft = actor.text("draft").as_ptr();
    actor.replace_for_test::<Upgraded>()?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("contacts"), slot);
    assert_eq!(actor.text("draft").as_ptr(), draft);
    assert_eq!(
        actor.view.bindings[&abi::key("status")].text,
        "Ada 🦀 · 香港\n東京 · false"
    );
    Ok(())
}

#[test]
fn nested_records_upgrade_automatically_with_declared_rust_conversions()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Original>()?;
    actor.tell(Original::put_action(), 0)?;
    actor.tick()?;
    let state = actor.state_address();
    let slot = actor.slot_address("contacts");
    let draft = actor.text("draft").as_ptr();
    actor.replace_for_test::<Automatic>()?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("contacts"), slot);
    assert_eq!(actor.text("draft").as_ptr(), draft);
    assert_eq!(
        actor.view.bindings[&abi::key("status")].text,
        "Ada 🦀 · 香港\n東京 · false"
    );
    Ok(())
}
