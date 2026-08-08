#![cfg(unix)]

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::Value;
use wabou_devtools::{DebugSnapshot, DebugState, DebugStatus, serve};

#[test]
fn mcp_lists_tools_and_forwards_status_to_the_runtime_socket() {
    let state = DebugState::shared();
    state.write().unwrap().publish(DebugSnapshot {
        status: DebugStatus {
            protocol_version: 1,
            pid: 77,
            revision: 12,
            node_count: 3,
            ..Default::default()
        },
        nodes: Vec::new(),
    });
    let path = std::env::temp_dir().join(format!(
        "wabou-mcp-test-{}-{}.sock",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _server = serve(state, path.clone()).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_wabou-mcp"))
        .env("WABOU_DEVTOOLS_SOCKET", &path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2025-03-26","capabilities":{{}},"clientInfo":{{"name":"test","version":"1"}}}}}}"#
    )
    .unwrap();
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","method":"notifications/initialized","params":{{}}}}"#
    )
    .unwrap();
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{{}}}}"#
    )
    .unwrap();
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"wabou_status","arguments":{{}}}}}}"#)
        .unwrap();
    drop(stdin);

    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    let first: Value = serde_json::from_str(&line).unwrap();
    line.clear();
    output.read_line(&mut line).unwrap();
    let second: Value = serde_json::from_str(&line).unwrap();
    line.clear();
    output.read_line(&mut line).unwrap();
    let third: Value = serde_json::from_str(&line).unwrap();
    let lines = [first, second, third];
    assert_eq!(lines[0]["result"]["serverInfo"]["name"], "wabou-devtools");
    let tools = lines[1]["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 8);
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "wabou_inspect_at_point")
    );
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "wabou_capture_case")
    );
    let text = lines[2]["result"]["content"][0]["text"].as_str().unwrap();
    let status: Value = serde_json::from_str(text).unwrap();
    assert_eq!(status["revision"], 12);
    assert_eq!(status["nodeCount"], 3);
    let _ = child.kill();
    let _ = child.wait();
}
