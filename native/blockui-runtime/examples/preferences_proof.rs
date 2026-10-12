fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    let window = paths.get(3).is_some_and(|arg| arg == "--window");
    if paths.len() != 3 && !(paths.len() == 4 && window) {
        return Err("preferences_proof base replacement evolved [--window]".into());
    }
    blockui_runtime::native::preferences_proof(
        std::path::Path::new(&paths[0]),
        std::path::Path::new(&paths[1]),
        std::path::Path::new(&paths[2]),
        window,
    )
}
