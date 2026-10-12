use wabou_native::native;

fn main() -> std::result::Result<(), String> {
    let options = native::WindowOptions::new().title("APP_TITLE");
    if std::env::args().any(|arg| arg == "--capture") {
        native::run_capture::<counter_app::Counter>(options)
    } else {
        native::run::<counter_app::Counter>(options)
    }
}
