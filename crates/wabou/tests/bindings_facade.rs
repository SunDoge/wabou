#![cfg(feature = "bindings")]

use wabou::{Bindings, Capability, JsonMethod, Type, specta};

#[allow(dead_code)]
#[derive(serde::Deserialize, serde::Serialize, Type)]
struct Payload {
    ready: bool,
}

const STATUS: JsonMethod<Payload, Payload> = JsonMethod::new("status");

#[test]
fn downstream_code_can_derive_and_generate_through_the_facade() {
    let output = Bindings::new()
        .capability(Capability::new("workspace").method(STATUS))
        .render();
    assert!(output.contains("status(request: Payload): Promise<Payload>"));
}
