use snafu::{ResultExt, Whatever};
use wabou::rquickjs::{Function, prelude::Async};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .widget("fractal", || Box::new(gallery::fractal::JuliaWidget::new()))
        .window(
            WindowOptions::new()
                .title("Wabou Components")
                .initial_inner_size(1080, 760)
                .min_inner_size(720, 520),
        )
        .capability(gallery::bindings::CAPABILITY, |ctx, capability| {
            capability.set(
                gallery::bindings::DESCRIBE_PALETTE,
                Function::new(
                    ctx,
                    Async(|raw: String| async move {
                        gallery::bindings::invoke_describe_palette(&raw).await
                    }),
                )?,
            )
        })
        .run()
        .whatever_context("failed to run component gallery")
}
