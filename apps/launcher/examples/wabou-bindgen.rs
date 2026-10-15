use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("expected `write` or `check`")?;
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/generated/host-bindings.ts");
    let bindings = wabou_launcher::service::manifest();
    match mode.as_str() {
        "write" => bindings.write(&output)?,
        "check" => bindings.check(&output)?,
        _ => return Err(format!("unknown bindings mode `{mode}`").into()),
    }
    println!("[wabou] launcher bindings {mode}: {}", output.display());
    Ok(())
}
