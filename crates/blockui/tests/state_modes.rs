#![cfg(feature = "macros")]
mod state_mode_declaration_tests {
    use blockui::*;
    #[actor]
    struct Conditional {
        #[cfg(any())]
        count: State<i64>,
    }
    #[actor]
    struct ConditionalAttribute {
        #[cfg_attr(all(), cfg(any()))]
        count: State<i64>,
    }
    #[actor]
    struct EnabledConditionalAttribute {
        #[cfg_attr(any(), cfg(any()))]
        #[cfg_attr(all(), doc = "ordinary field documentation")]
        count: State<i64>,
    }
    #[actor]
    struct ConditionalRecord {
        #[state(record)]
        #[cfg_attr(all(), cfg_attr(all(), cfg(any())))]
        part: EnabledConditionalAttribute,
    }
    #[actor]
    struct Empty {}
    #[actor]
    struct Enabled {
        #[cfg(feature = "macros")]
        count: State<i64>,
    }
    #[test]
    fn declarations_without_active_fields_keep_empty_schemas() {
        let _ = Conditional::new();
        let _ = ConditionalAttribute::new().scoped(Key::named("scope"));
        assert!(ConditionalAttribute::SCHEMA.is_empty());
        let _ = ConditionalRecord::new().scoped(Key::named("scope"));
        assert!(ConditionalRecord::SCHEMA.is_empty());
        let enabled = EnabledConditionalAttribute::new().scoped(Key::named("scope"));
        assert_eq!(EnabledConditionalAttribute::SCHEMA.len(), 1);
        assert_eq!(
            enabled.count.key(),
            EnabledConditionalAttribute::new()
                .count
                .scoped(Key::named("scope"))
                .key()
        );
        let _ = Empty::new();
        assert!(Conditional::SCHEMA.is_empty());
        assert!(Empty::SCHEMA.is_empty());
        assert_eq!(Enabled::SCHEMA.len(), 1);
        assert_eq!(Enabled::new().count.key(), Key::named("Enabled.count").0);
        let _ = Conditional::new().scoped(Key::named("scope"));
        let _ = Enabled::new().scoped(Key::named("scope"));
    }
}

mod typed_receiver_actions {
    use blockui::*;
    #[actor]
    struct Settings {
        enabled: State<bool>,
        ratio: State<f64>,
    }
    #[actor_impl]
    impl Settings {
        #[action]
        #[allow(unused_parens)]
        fn toggle(&self, cx: &Cx<'_>, value: (core::primitive::bool)) -> Result<()> {
            self.enabled.set(cx, value)
        }
        #[action]
        fn sample(&self, cx: &Cx<'_>, value: std::primitive::f64) -> Result<()> {
            self.ratio.set(cx, value)
        }
        #[action]
        #[cfg_attr(all(), cfg(any()), doc = "absent action")]
        fn absent(&self, _: &Cx<'_>) -> Result<()> {
            Ok(())
        }
        #[computed]
        #[cfg_attr(all(), cfg_attr(all(), cfg(any())))]
        fn absent_value(&self, _: &Cx<'_>) -> Result<i64> {
            Ok(1)
        }
        fn view(&self, _: &Cx<'_>) -> View {
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new("Settings"));
                }
                __blockui_tree.finish()
            }
        }
    }
    #[test]
    fn generated_accessors_keep_scalar_types_and_stable_ids() {
        assert_eq!(<Settings as Actor>::ACTIONS.len(), 2);
        assert!(<Settings as Actor>::COMPUTEDS.is_empty());
        let boolean: BoolAction = Settings::toggle_action();
        let float: FloatAction = Settings::sample_action();
        assert_eq!(<Settings as Actor>::ACTIONS[0].id, boolean.spec().id);
        assert_eq!(<Settings as Actor>::ACTIONS[1].id, float.spec().id);
        assert_eq!(boolean.spec().id, abi::key("Settings.toggle"));
        assert_eq!(float.spec().id, abi::key("Settings.sample"));
    }
}

mod qualified_state_types {
    use blockui::*;
    #[actor]
    struct Model {
        #[state(default = &[2, 3])]
        values: blockui::State<std::vec::Vec<core::primitive::i64>>,
        other: State<Vec<std::primitive::i64>>,
    }
    #[test]
    fn qualified_integer_lists_generate_normal_typed_descriptors() {
        let model = Model::new();
        assert_eq!(Model::SCHEMA.len(), 2);
        let values: State<Vec<i64>> = model.values;
        let other: State<Vec<i64>> = model.other;
        assert_eq!(values.key(), Key::named("Model.values").0);
        assert_eq!(other.key(), Key::named("Model.other").0);
    }
}

mod conditional_migration {
    use blockui::*;
    #[actor]
    struct Model {}
    #[actor_impl]
    impl Model {
        #[cfg_attr(all(), cfg(any()))]
        fn migrate(&self, _: &Cx<'_>, _: u64) -> Result<()> {
            Ok(())
        }
        fn view(&self, _: &Cx<'_>) -> View {
            Text::new("Model").into_view()
        }
    }
    #[test]
    fn absent_migration_uses_the_default_trait_adapter() {
        let _: fn(&Cx<'_>, u64) -> Result<()> = <Model as Actor>::migrate;
    }
}

mod alternate_migrations {
    use blockui::*;
    #[actor]
    struct Model {}
    #[actor_impl]
    impl Model {
        #[cfg(feature = "macros")]
        fn migrate(&self, _: &Cx<'_>, _: u64) -> Result<()> {
            Ok(())
        }
        #[cfg_attr(all(), cfg(any()))]
        fn migrate(&self, _: &Cx<'_>, _: u64) -> Result<()> {
            Ok(())
        }
        fn view(&self, _: &Cx<'_>) -> View {
            Text::new("Model").into_view()
        }
    }
    #[test]
    fn mutually_exclusive_migrations_generate_matching_adapters() {
        let _: fn(&Cx<'_>, u64) -> Result<()> = <Model as Actor>::migrate;
    }
}
