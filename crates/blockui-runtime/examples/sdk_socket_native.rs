fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("sdk_socket_native <base> <replacement> <rejected> <evolved>".into());
    }
    blockui_runtime::native::sdk_socket_proof(
        std::path::Path::new(&paths[0]),
        std::path::Path::new(&paths[1]),
        std::path::Path::new(&paths[2]),
        std::path::Path::new(&paths[3]),
    )
}
