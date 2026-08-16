use wabou::{HostBuilder, WindowOptions};

fn main() -> wabou::Result<()> {
    HostBuilder::new()
        .window(WindowOptions::new().title("__WABOU_PROJECT_NAME__"))
        .run()
}
