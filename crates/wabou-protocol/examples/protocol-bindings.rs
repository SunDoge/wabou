use std::{error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("expected `write` or `check`")?;
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/core/src/generated/wire-protocol.ts");
    let generated = wabou_protocol::typescript_wire_constants();
    match mode.as_str() {
        "write" => {
            if fs::read_to_string(&output).ok().as_deref() != Some(&generated) {
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output, generated)?;
            }
        }
        "check" => {
            let current = fs::read_to_string(&output)?;
            if current != generated {
                return Err(format!(
                    "{} is stale; run `cargo run -p wabou-protocol --example protocol-bindings -- write`",
                    output.display()
                )
                .into());
            }
        }
        _ => return Err(format!("unknown bindings mode `{mode}`").into()),
    }
    println!("[wabou] protocol bindings {mode}: {}", output.display());
    Ok(())
}
