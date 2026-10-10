//! Experimental Rust UI frontend, sharing Wabou's native shell and widgets.
//! SDK/runtime are local implementation crates in this checkout. This facade
//! does not yet unify Wabou capability or UI protocols.
pub use blockui::*;

#[cfg(feature = "testing")]
pub mod testing {
    pub use blockui_test::*;
}

#[cfg(feature = "native")]
pub mod native {
    pub use blockui_runtime::app::{WindowOptions, run_source};
    pub use blockui_runtime::application::Services;

    pub fn mount<A: blockui::Actor>() -> Result<blockui_runtime::native::NativeSource, String> {
        blockui_runtime::native::NativeSource::without_watch(blockui_runtime::Actor::mount::<A>(1)?)
    }

    pub fn run<A: blockui::Actor>(options: WindowOptions) -> Result<(), String> {
        run_source(mount::<A>()?, options)
    }

    pub fn run_capture<A: blockui::Actor>(options: WindowOptions) -> Result<(), String> {
        run_source(mount::<A>()?.capture(), options)
    }
}

/// Selected embedded Lucide SVG icons; enable the `lucide` feature.
#[cfg(feature = "lucide")]
pub use blockui_icons_lucide as lucide;
