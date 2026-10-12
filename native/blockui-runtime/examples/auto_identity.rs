fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 8 {
        return Err(
            "auto_identity base replacement ambiguous static-ambiguous disabled row-root string-ambiguous".into(),
        );
    }
    blockui_runtime::native::auto_key_proof(
        std::path::Path::new(&args[1]),
        std::path::Path::new(&args[2]),
        std::path::Path::new(&args[3]),
        std::path::Path::new(&args[4]),
        std::path::Path::new(&args[5]),
        std::path::Path::new(&args[6]),
        std::path::Path::new(&args[7]),
    )
}
