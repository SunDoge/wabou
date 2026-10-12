fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 2 {
        return Err("router_native base replacement".into());
    }
    blockui_runtime::native::router_proof(
        std::path::Path::new(&paths[0]),
        std::path::Path::new(&paths[1]),
    )
}
