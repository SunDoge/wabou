//! One native library can export many capabilities and widgets. The runtime
//! owns their separate state namespaces and swaps the complete registry at once.
use crate::{
    Host, Module, Ready, Task,
    widget::{WidgetImplementation, WidgetModule},
};
use wabou_host_api::plugin::MANIFEST_METHOD;
pub use wabou_host_api::plugin::{Export, ExportKind, PLUGIN_CONTRACT_VERSION};
type Start = fn(&Host<'_>, &str, &[u8]) -> Result<Box<dyn Task>, String>;
struct Registration {
    metadata: Export,
    start: Start,
}
/// Small registration table allocated inside module code; never crosses the ABI.
#[derive(Default)]
pub struct Registry {
    entries: Vec<Registration>,
}
impl Registry {
    /// Register an ordinary capability implementation by its stable contract name.
    pub fn capability<M: Module>(&mut self, name: &str) -> Result<(), String> {
        self.register(ExportKind::Capability, name, M::CONTRACT_VERSION, M::start)
    }
    /// Register an ordinary native widget implementation by its stable tag.
    pub fn widget<W: WidgetImplementation>(&mut self, tag: &str) -> Result<(), String> {
        self.register(
            ExportKind::Widget,
            tag,
            W::SCHEMA_VERSION,
            WidgetModule::<W>::start,
        )
    }
    fn register(
        &mut self,
        kind: ExportKind,
        name: &str,
        version: u32,
        start: Start,
    ) -> Result<(), String> {
        if name.trim().is_empty()
            || self
                .entries
                .iter()
                .any(|e| e.metadata.kind == kind && e.metadata.name == name)
        {
            return Err(format!(
                "invalid or duplicate plugin export: {kind:?}/{name}"
            ));
        }
        self.entries.push(Registration {
            metadata: Export {
                kind,
                name: name.into(),
                version,
            },
            start,
        });
        Ok(())
    }
}
/// Register implementations using Rust types; application state belongs to host.
pub trait Plugin: 'static {
    /// Called to describe this code version; do not create durable state here.
    fn register(registry: &mut Registry) -> Result<(), String>;
}
/// Adapter retaining the existing stabby ABI, task disposal and panic containment.
pub struct PluginModule<P>(std::marker::PhantomData<P>);
impl<P: Plugin> Module for PluginModule<P> {
    const CONTRACT_VERSION: u32 = PLUGIN_CONTRACT_VERSION;
    fn start(host: &Host<'_>, method: &str, request: &[u8]) -> Result<Box<dyn Task>, String> {
        let mut registry = Registry::default();
        P::register(&mut registry)?;
        if method == MANIFEST_METHOD {
            return Ready::json(
                &registry
                    .entries
                    .iter()
                    .map(|e| &e.metadata)
                    .collect::<Vec<_>>(),
            );
        }
        let (kind, name, method): (ExportKind, String, String) =
            serde_json::from_str(method).map_err(|e| e.to_string())?;
        let entry = registry
            .entries
            .iter()
            .find(|e| e.metadata.kind == kind && e.metadata.name == name)
            .ok_or_else(|| format!("unknown plugin export: {kind:?}/{name}"))?;
        (entry.start)(host, &method, request)
    }
}
/// Export one registry, with the same dynamic/static symbol hygiene as export_module!.
/// Names are contract identities; capability and widget namespaces are separate.
#[macro_export]
macro_rules! export_plugin {
    (capabilities { $($cap_name:literal => $cap:ty),* $(,)? } widgets { $($tag:literal => $widget:ty),* $(,)? }) => {
        struct __WabouPlugin;
        impl $crate::plugin::Plugin for __WabouPlugin {
            fn register(registry: &mut $crate::plugin::Registry) -> Result<(), String> {
                $(registry.capability::<$cap>($cap_name)?;)*
                $(registry.widget::<$widget>($tag)?;)*
                Ok(())
            }
        }
        $crate::export_module!($crate::plugin::PluginModule<__WabouPlugin>);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{Draw, Event, Inputs, Update};
    struct Capability;
    impl Module for Capability {
        const CONTRACT_VERSION: u32 = 1;
        fn start(_: &Host<'_>, _: &str, _: &[u8]) -> Result<Box<dyn Task>, String> {
            Ready::json(&())
        }
    }
    struct Widget;
    impl WidgetImplementation for Widget {
        const SCHEMA_VERSION: u32 = 2;
        fn measure(_: &Inputs) -> Result<Option<[f32; 2]>, String> {
            Ok(None)
        }
        fn paint(_: &Inputs) -> Result<Vec<Draw>, String> {
            Ok(Vec::new())
        }
        fn event(inputs: &Inputs, _: Event) -> Result<Update, String> {
            Ok(Update {
                state: inputs.state.clone(),
                handled: false,
            })
        }
    }
    #[test]
    fn registry_rejects_duplicates_but_namespaces_can_share_names() {
        let mut registry = Registry::default();
        registry.capability::<Capability>("counter").unwrap();
        registry.widget::<Widget>("counter").unwrap();
        assert!(registry.capability::<Capability>("counter").is_err());
        assert!(registry.widget::<Widget>("counter").is_err());
        assert!(registry.capability::<Capability>(" ").is_err());
        assert_eq!(registry.entries.len(), 2);
    }
}
