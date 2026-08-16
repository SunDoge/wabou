//! Unified Host → JavaScript frame codec.
//!
//! Guest-initiated mounted functions return through their call/Promise. This
//! module is only for unsolicited Host facts: targeted node events, layout
//! observations and application messages.

use std::time::Duration;

use crate::host_message::{HostMessage, HostMessagePayload};
use crate::protocol::{host_frame, host_node_payload, host_record};

/// Maximum encoded bytes accepted for one atomic host frame.
pub const MAX_HOST_FRAME_BYTES: usize = 4 * 1024 * 1024;
/// Maximum records delivered to JavaScript in one host frame.
pub const MAX_HOST_FRAME_RECORDS: usize = 512;
/// Node-event header flag indicating JavaScript may cancel the event.
pub const FLAG_CANCELLABLE: u8 = 1;

#[derive(Debug, Clone, PartialEq)]
/// Payload representation for an unsolicited event targeting one Solid node.
pub enum NodeEventPayload {
    /// Event carries only its header fields.
    None,
    /// Fixed-width numeric event-data slots defined by the generated ABI.
    Numeric([f64; crate::protocol::event_data::LEN]),
    /// Event-specific JSON object.
    Json(String),
}

#[derive(Debug, Clone, PartialEq)]
/// Unsolicited event addressed to one retained Solid node.
pub struct HostNodeEvent {
    /// Solid node identifier.
    pub target: u32,
    /// Generated event discriminator.
    pub event_code: u8,
    /// Per-runtime identifier used for cancellation routing.
    pub event_id: u32,
    /// Whether JavaScript may cancel the host's default action.
    pub cancellable: bool,
    /// Event-specific payload.
    pub payload: NodeEventPayload,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Content-box size observation for one retained Solid node.
pub struct ResizeObservation {
    /// Solid node identifier.
    pub target: u32,
    /// Content-box width in logical pixels.
    pub width: f32,
    /// Content-box height in logical pixels.
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
/// Record that can be batched into one Rust-to-JavaScript host frame.
pub enum HostEvent {
    /// Event addressed to one retained node.
    Node(HostNodeEvent),
    /// ResizeObserver-compatible content-box observation.
    Resize(ResizeObservation),
    /// Application topic message from a Rust producer.
    Application(HostMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Boundary validation failure while encoding an atomic host frame.
pub enum HostFrameError {
    /// Record count exceeds the host-frame record limit.
    TooManyRecords,
    /// Complete frame or a record exceeds the binary ABI limit.
    TooLarge,
    /// A short-string field exceeds its 16-bit length representation.
    StringTooLarge,
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn begin_record(out: &mut Vec<u8>, kind: u8, flags: u8) -> usize {
    let start = out.len();
    out.extend_from_slice(&[kind, flags, 0, 0, 0, 0, 0, 0]);
    start
}

fn end_record(out: &mut [u8], start: usize) {
    let len = (out.len() - start) as u32;
    out[start + 4..start + 8].copy_from_slice(&len.to_le_bytes());
}

fn push_short_str(out: &mut Vec<u8>, value: &str) -> Result<(), HostFrameError> {
    let len = u16::try_from(value.len()).map_err(|_| HostFrameError::StringTooLarge)?;
    push_u16(out, len);
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_application(out: &mut Vec<u8>, msg: &HostMessage) -> Result<(), HostFrameError> {
    push_short_str(out, &msg.topic)?;
    match &msg.payload {
        HostMessagePayload::Null => out.push(0),
        HostMessagePayload::Bool(value) => {
            out.extend_from_slice(&[1, u8::from(*value)]);
        }
        HostMessagePayload::I32(value) => {
            out.push(2);
            out.extend_from_slice(&value.to_le_bytes());
        }
        HostMessagePayload::F64(value) => {
            out.push(3);
            push_f64(out, *value);
        }
        HostMessagePayload::Str(value) => {
            out.push(4);
            push_short_str(out, value)?;
        }
        HostMessagePayload::Bytes(value) => {
            out.push(5);
            push_u32(out, value.len() as u32);
            out.extend_from_slice(value);
        }
    }
    Ok(())
}

/// Encode one atomic Host frame. `monotonic_time` is relative to the runtime's
/// own epoch; callers may pass zero for deterministic tests/replay.
pub fn encode_host_frame(
    sequence: u64,
    monotonic_time: Duration,
    events: &[HostEvent],
) -> Result<Vec<u8>, HostFrameError> {
    if events.len() > MAX_HOST_FRAME_RECORDS {
        return Err(HostFrameError::TooManyRecords);
    }
    let mut out = Vec::with_capacity(host_frame::HEADER_LEN as usize + events.len() * 32);
    push_u32(&mut out, host_frame::MAGIC);
    push_u16(&mut out, host_frame::VERSION as u16);
    push_u16(&mut out, 0);
    push_u64(&mut out, sequence);
    push_u64(
        &mut out,
        monotonic_time.as_nanos().min(u64::MAX as u128) as u64,
    );
    push_u32(&mut out, events.len() as u32);
    push_u32(&mut out, 0); // byte_len, patched below

    for event in events {
        let (kind, flags) = match event {
            HostEvent::Node(node) => (
                host_record::NODE_EVENT,
                if node.cancellable {
                    FLAG_CANCELLABLE
                } else {
                    0
                },
            ),
            HostEvent::Resize(_) => (host_record::RESIZE, 0),
            HostEvent::Application(_) => (host_record::APPLICATION_MESSAGE, 0),
        };
        let start = begin_record(&mut out, kind, flags);
        match event {
            HostEvent::Node(node) => {
                push_u32(&mut out, node.target);
                out.push(node.event_code);
                out.push(match node.payload {
                    NodeEventPayload::None => host_node_payload::NONE,
                    NodeEventPayload::Numeric(_) => host_node_payload::NUMERIC,
                    NodeEventPayload::Json(_) => host_node_payload::JSON,
                });
                push_u16(&mut out, 0);
                push_u32(&mut out, node.event_id);
                match &node.payload {
                    NodeEventPayload::None => {}
                    NodeEventPayload::Numeric(values) => {
                        for value in values {
                            push_f64(&mut out, *value);
                        }
                    }
                    NodeEventPayload::Json(json) => {
                        push_u32(&mut out, json.len() as u32);
                        out.extend_from_slice(json.as_bytes());
                    }
                }
            }
            HostEvent::Resize(resize) => {
                push_u32(&mut out, resize.target);
                push_f32(&mut out, resize.width);
                push_f32(&mut out, resize.height);
            }
            HostEvent::Application(msg) => encode_application(&mut out, msg)?,
        }
        let len = out.len() - start;
        if len > u32::MAX as usize || out.len() > MAX_HOST_FRAME_BYTES {
            return Err(HostFrameError::TooLarge);
        }
        end_record(&mut out, start);
    }
    if out.len() > MAX_HOST_FRAME_BYTES {
        return Err(HostFrameError::TooLarge);
    }
    let byte_len = out.len() as u32;
    out[28..32].copy_from_slice(&byte_len.to_le_bytes());
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    #[test]
    fn encodes_versioned_frame_and_record_lengths() {
        let frame = encode_host_frame(
            7,
            Duration::from_nanos(42),
            &[
                HostEvent::Resize(ResizeObservation {
                    target: 9,
                    width: 80.0,
                    height: 24.0,
                }),
                HostEvent::Application(HostMessage::str("status", "ready")),
            ],
        )
        .unwrap();
        assert_eq!(u32_at(&frame, 0), host_frame::MAGIC);
        assert_eq!(u32_at(&frame, 24), 2);
        assert_eq!(u32_at(&frame, 28), frame.len() as u32);
        assert_eq!(frame[32], host_record::RESIZE);
        let second = 32 + u32_at(&frame, 36) as usize;
        assert_eq!(frame[second], host_record::APPLICATION_MESSAGE);
    }

    #[test]
    fn rejects_oversized_short_string() {
        let error = encode_host_frame(
            1,
            Duration::ZERO,
            &[HostEvent::Application(HostMessage::str(
                "x".repeat(u16::MAX as usize + 1),
                "value",
            ))],
        )
        .unwrap_err();
        assert_eq!(error, HostFrameError::StringTooLarge);
    }
}
