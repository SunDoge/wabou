fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    let window = paths.get(4).is_some_and(|arg| arg == "--window");
    if paths.len() != 4 && !(paths.len() == 5 && window) {
        return Err("record_notepad_proof base replacement missing evolved [--window]".into());
    }
    blockui_runtime::native::record_notepad_proof(
        std::path::Path::new(&paths[0]),
        std::path::Path::new(&paths[1]),
        std::path::Path::new(&paths[2]),
        std::path::Path::new(&paths[3]),
        window,
    )
}
