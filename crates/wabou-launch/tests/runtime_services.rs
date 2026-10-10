//! Full native launch boundary; requires explicitly built trusted artifacts.
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use wabou_launch::{Capability, Implementation, LaunchConfig, Method, UiSource};

#[test]
#[ignore = "requires WABOU_PREBUILT_RUNTIME, WABOU_PREBUILT_LAUNCHER and WABOU_PREBUILT_PROBE"]
fn shared_runtime_serves_sdk_state_file_task_and_connection_calls() {
    let artifact = |name| {
        PathBuf::from(std::env::var_os(name).expect(name))
            .canonicalize()
            .unwrap()
    };
    let runtime = artifact("WABOU_PREBUILT_RUNTIME");
    let launcher = artifact("WABOU_PREBUILT_LAUNCHER");
    let probe = artifact("WABOU_PREBUILT_PROBE");
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("scenario.js");
    let build = Command::new("bun")
        .current_dir(&workspace)
        .args([
            "build",
            "--target=browser",
            "--format=iife",
            "--conditions=browser,wabou-source",
        ])
        .arg(workspace.join("crates/wabou-runtime-library/tests/services.behavior.ts"))
        .arg("--outfile")
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        // Nonblocking accept gives a failed launch a bounded fixture lifetime.
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "probe never connected");
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut byte = [0];
        stream.read_exact(&mut byte).unwrap();
        stream.write_all(&byte).unwrap();
        assert_eq!(
            stream.read(&mut byte).unwrap(),
            0,
            "runtime must close connection"
        );
    });
    let bundle = directory.path().join("bundle.js");
    std::fs::write(
        &bundle,
        format!(
            "globalThis.__probe_address = {};",
            serde_json::to_string(&address.to_string()).unwrap()
        ),
    )
    .unwrap();
    std::fs::write(directory.path().join("document.txt"), b"AB").unwrap();
    let mut config = LaunchConfig::new(UiSource::Bundle { path: bundle });
    config.capabilities.push(Capability {
        name: "probe".into(),
        version: 1,
        methods: [
            "increment",
            "sleep",
            "poll",
            "release",
            "openFile",
            "readFile",
            "closeFile",
            "connect",
            "readSocket",
            "writeSocket",
            "closeSocket",
        ]
        .into_iter()
        .map(|name| Method {
            name: name.into(),
            has_request: true,
            json: true,
        })
        .collect(),
        implementation: Implementation::Library { path: probe },
    });
    let configuration = directory.path().join("app.json");
    std::fs::write(&configuration, serde_json::to_vec(&config).unwrap()).unwrap();
    let log = directory.path().join("native.log");
    let output = std::fs::File::create(&log).unwrap();
    let mut child = Command::new(launcher)
        .arg(runtime)
        .arg(configuration)
        .env("WABOU_TEST_HEADLESS", "1")
        .env("WABOU_TEST_RENDERER", "vello-hybrid")
        .env("WABOU_TEST_SCRIPT", &script)
        .env("WABOU_TEST_ARTIFACT_DIR", directory.path())
        .stdin(Stdio::null())
        .stdout(output.try_clone().unwrap())
        .stderr(output)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!(
                "runtime did not finish: {}",
                std::fs::read_to_string(&log).unwrap()
            );
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    assert!(
        status.success(),
        "{}",
        std::fs::read_to_string(&log).unwrap()
    );
    server.join().unwrap();
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("report.json")).unwrap())
            .unwrap();
    assert_eq!(report["passed"], true, "{report}");
    assert_eq!(report["tests"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["tests"][0]["name"],
        "shared runtime supplies typed state, files, tasks and TCP to SDK modules"
    );
    assert_eq!(report["tests"][0]["passed"], true);
}
