use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("expected `write` or `check`")?;
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/core/src/renderer/generated/native-host.ts");
    match mode.as_str() {
        "write" => wabou_host_api::bindings().write(&output)?,
        "check" => wabou_host_api::bindings().check(&output)?,
        _ => return Err(format!("unknown bindings mode `{mode}`").into()),
    }
    println!("[wabou] native host bindings {mode}: {}", output.display());
    Ok(())
}
