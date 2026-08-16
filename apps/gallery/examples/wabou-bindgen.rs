use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("expected `write` or `check`")?;
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/generated/host-bindings.ts");
    match mode.as_str() {
        "write" => gallery::bindings::manifest().write(&output)?,
        "check" => gallery::bindings::manifest().check(&output)?,
        _ => return Err(format!("unknown bindings mode `{mode}`").into()),
    }
    println!("[wabou] bindings {mode}: {}", output.display());
    Ok(())
}
