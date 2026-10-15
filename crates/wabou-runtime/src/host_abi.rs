//! Generated inventory for the private Rust↔QuickJS ABI.

include!("gen/host_abi.rs");

/// Install a generated host ABI function and give it the schema-owned name.
pub(crate) fn register_function<'js>(
    globals: &rquickjs::Object<'js>,
    binding: HostBinding,
    function: rquickjs::Function<'js>,
) -> rquickjs::Result<()> {
    let name = binding.name();
    globals.set(name, function.with_name(name)?)
}

/// Install a generated host ABI value under its schema-owned name.
pub(crate) fn register_value<'js, Value>(
    globals: &rquickjs::Object<'js>,
    binding: HostBinding,
    value: Value,
) -> rquickjs::Result<()>
where
    Value: rquickjs::IntoJs<'js>,
{
    globals.set(binding.name(), value)
}
