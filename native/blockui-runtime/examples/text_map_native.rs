fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() == 3 {
        return blockui_runtime::native::text_map_migration_proof(
            std::path::Path::new(&paths[0]),
            std::path::Path::new(&paths[1]),
            std::path::Path::new(&paths[2]),
        );
    }
    if paths.len() != 2 {
        return Err("text_map_native base replacement".into());
    }
    blockui_runtime::native::text_map_proof(
        std::path::Path::new(&paths[0]),
        std::path::Path::new(&paths[1]),
    )
}
