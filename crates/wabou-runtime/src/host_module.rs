//! Runtime-owned state for `rquickjs` native modules.

/// A static Rust value stored in the QuickJS runtime userdata map.
///
/// The wrapper promises that `T` contains no values borrowing a QuickJS
/// lifetime. Ordinary services built from owned values, `Arc`, channels, and
/// resource handles satisfy that contract.
#[derive(Clone, Debug)]
pub struct HostModuleState<T: 'static>(pub T);

unsafe impl<'js, T: 'static> rquickjs::JsLifetime<'js> for HostModuleState<T> {
    type Changed<'to> = Self;
}
