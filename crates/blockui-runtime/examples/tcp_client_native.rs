fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if !matches!(paths.len(), 2 | 3) {
        return Err("tcp_client_native <base> <replacement> [flat]".into());
    }
    blockui_runtime::native::tcp_client_proof(
        std::path::Path::new(&paths[0]),
        std::path::Path::new(&paths[1]),
        paths.get(2).map(std::path::Path::new),
    )
}
