fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 3 {
        return Err("text_list_native base replacement rejected".into());
    }
    blockui_runtime::native::text_list_app_proof(
        std::path::Path::new(&paths[0]),
        std::path::Path::new(&paths[1]),
        std::path::Path::new(&paths[2]),
    )
}
