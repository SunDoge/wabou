use native_counter::Counter;
use wabou_native::native;
fn main() -> std::result::Result<(), String> {
    let options = native::WindowOptions::new().title("Wabou native counter");
    if std::env::args().any(|arg| arg == "--capture") {
        native::run_capture::<Counter>(options)
    } else {
        native::run::<Counter>(options)
    }
}
