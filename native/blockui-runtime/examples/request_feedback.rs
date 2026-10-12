use blockui_runtime::{App, app::WindowOptions};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let client = args
        .get(1)
        .ok_or("request_feedback <client.so> <service.so>")?;
    let service = args.get(2).ok_or("missing service library")?;
    App::new(client, ".runtime")
        .implementation("feedback.service", service)?
        .window(
            WindowOptions::new()
                .title("Request feedback")
                .initial_inner_size(520, 480),
        )
        .run()
}
