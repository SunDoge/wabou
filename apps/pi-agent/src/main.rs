mod service;

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pi_agent_wabou=info,wabou=info".into()),
        )
        .init();

    let service = service::PiService::new();
    let capability = service.clone();
    let events = service.clone();
    HostBuilder::new()
        .app_directories("dev", "Wabou", "Pi Agent")
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title("Pi Agent · Wabou")
                .initial_inner_size(1180, 780)
                // The fixed 288px sidebar leaves 892px for the conversation
                // toolbar and composer. Keep the native window above that
                // measured application-shell boundary instead of allowing
                // flex children to be compressed into one another.
                .min_inner_size(1180, 680),
        )
        .widget("terminal", wabou_terminal::terminal_widget)
        .capability(service::CAPABILITY, move |host| {
            service::mount(host, capability.clone())
        })
        .host_message_producer(move |context| service::stream_events(context, events.clone()))
        .run()
        .whatever_context("failed to run Pi Agent")
}
