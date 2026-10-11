use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("expected `write` or `check`")?;
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/generated/host-bindings.ts");
    let contracts = output.with_file_name("host-contracts.ts");
    let bindings = gallery::bindings::manifest();
    match mode.as_str() {
        "write" => {
            bindings.write(&output)?;
            bindings.write_contracts(&contracts)?;
        }
        "check" => {
            bindings.check(&output)?;
            bindings.check_contracts(&contracts)?;
        }
        _ => return Err(format!("unknown bindings mode `{mode}`").into()),
    }
    println!("[wabou] bindings {mode}: {}", output.display());
    Ok(())
}
