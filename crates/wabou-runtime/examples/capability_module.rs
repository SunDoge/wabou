//! Run after building the SDK counter dylib; pass its path as the sole argument.
#[cfg(feature = "capability-modules")]
fn main() -> Result<(), String> {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };
    use wabou_runtime::capability_module::CapabilityModule;
    let path = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("expected counter library path")?,
    );
    let count = Arc::new(Mutex::new(0u32));
    let retained = count.clone();
    let module = CapabilityModule::new(
        1,
        Arc::new(move |operation, request| {
            let mut value = retained.lock().map_err(|e| e.to_string())?;
            match operation {
                "counter.increment" => {
                    *value += serde_json::from_slice::<u32>(request).map_err(|e| e.to_string())?;
                }
                "counter.read" => {}
                _ => return Err(format!("unknown operation: {operation}")),
            }
            serde_json::to_vec(&*value).map_err(|e| e.to_string())
        }),
    );
    // This example loads the trusted library built from our SDK example.
    unsafe {
        module.reload(&path)?;
    }
    let mut call = module.start("increment", b"2")?;
    assert_eq!(call.poll()?, Some(b"2".to_vec()));
    drop(call);
    unsafe {
        module.reload(&path)?;
    }
    let mut call = module.start("increment", b"3")?;
    assert_eq!(call.poll()?, Some(b"5".to_vec()));
    drop(call);
    let mut label = module.start("label", b"null")?;
    let label = label.poll()?.ok_or("unexpected pending label")?;
    assert_eq!(
        serde_json::from_slice::<String>(&label).map_err(|e| e.to_string())?,
        "Count: 5"
    );
    println!(
        "SDK dylib replacement retained host counter: {}",
        *count.lock().unwrap()
    );
    Ok(())
}
#[cfg(not(feature = "capability-modules"))]
fn main() {
    eprintln!("enable the capability-modules feature");
}
