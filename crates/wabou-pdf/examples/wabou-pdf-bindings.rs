use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("expected `write` or `check`")?;
    let output =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/pdf/generated/index.tsx");
    let bindings = wabou_pdf::bindings();
    match mode.as_str() {
        "write" => bindings.write(&output)?,
        "check" => bindings.check(&output)?,
        _ => return Err(format!("unknown bindings mode `{mode}`").into()),
    }
    println!("[wabou] PDF widget bindings {mode}: {}", output.display());
    Ok(())
}
