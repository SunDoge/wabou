use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("expected `write` or `check`")?;
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/generated/native-downloads.ts");
    match mode.as_str() {
        "write" => motrix_wabou::downloads::native_bindings().write(&output)?,
        "check" => motrix_wabou::downloads::native_bindings().check(&output)?,
        _ => return Err(format!("unknown bindings mode `{mode}`").into()),
    }
    println!("[motrix] native bindings {mode}: {}", output.display());
    Ok(())
}
