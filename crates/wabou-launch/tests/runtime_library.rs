//! Actual prebuilt-library checks, run explicitly after building the runtime.
use stabby::libloading::StabbyLibrary;
use wabou_launch::{LaunchConfig, RuntimeLibrary, UiSource};

#[test]
#[ignore = "requires WABOU_PREBUILT_RUNTIME pointing at a built runtime library"]
fn prebuilt_runtime_rejects_wrong_signature_and_launch_contract() {
    let path =
        std::path::PathBuf::from(std::env::var_os("WABOU_PREBUILT_RUNTIME").expect("runtime path"));
    let library = unsafe { libloading::Library::new(&path) }.unwrap();
    let mismatched =
        unsafe { library.get_stabbied::<extern "C" fn(u64) -> u64>(b"wabou_runtime_launch") };
    assert!(
        mismatched.is_err(),
        "ABI-incompatible entry must be rejected before invocation"
    );
    drop(library);
    let runtime = unsafe { RuntimeLibrary::load(&path) }.unwrap();
    let mut config = LaunchConfig::new(UiSource::Bundle {
        path: "missing-bundle.js".into(),
    });
    config.version += 1;
    assert_eq!(
        runtime.run(&config).unwrap_err(),
        "incompatible runtime launch contract"
    );
}
