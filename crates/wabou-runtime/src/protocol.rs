//! Binary bridge wire protocol (JS SolidJS reconciler → Rust).
//!
//! Frame layout (all little-endian):
//!   `[seq: u32][count: u32][op...]`
//! Each op is `[op: u8][operands...]`. Strings are either `[len: u16][utf8]` or
//! `[0xffff][frame-local string index: u16]`. Inline strings of at least four
//! bytes enter the frame-local table; the table is discarded after decoding.
//! Structural strings are runtime-scoped atom IDs (`u32`); dynamic
//! values and text use inline UTF-8 plus frame-local references.
//! Node identities are `[lo: u32, hi: u32]` full-width generational keys.
//!
//! Decoder copied from blitz-js (host-agnostic); op constants are generated
//! from `packages/core/src/protocol/index.ts` by `scripts/codegen/protocol/op.ts`.
//! New records and resource handles follow `docs/runtime-contract.md`.

#![allow(dead_code)]

include!("gen/op.rs");

use crate::atom::Atom;
pub use wabou_host_api::NodeKey;

pub const TEXT_BEHAVIOR_AGGREGATE_DIRECT: u8 = 0x01;
pub const TEXT_BEHAVIOR_SINGLE_LINE: u8 = 0x02;
const TEXT_BEHAVIOR_MASK: u8 = TEXT_BEHAVIOR_AGGREGATE_DIRECT | TEXT_BEHAVIOR_SINGLE_LINE;
pub const INTERACTION_POLICY_FOCUSABLE: u8 = 0x01;
pub const INTERACTION_POLICY_BLOCK_SUBTREE: u8 = 0x02;
pub const INTERACTION_POLICY_CONTAIN_FOCUS: u8 = 0x04;
const INTERACTION_POLICY_MASK: u8 = INTERACTION_POLICY_FOCUSABLE
    | INTERACTION_POLICY_BLOCK_SUBTREE
    | INTERACTION_POLICY_CONTAIN_FOCUS;
pub const GRAPHIC_SOURCE_SVG: u8 = 0x01;
pub const GRAPHIC_SOURCE_NETWORK_RASTER: u8 = 0x02;

fn valid_graphic_source_kind(kind: u8) -> bool {
    matches!(kind, GRAPHIC_SOURCE_SVG | GRAPHIC_SOURCE_NETWORK_RASTER)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StyleValue {
    Px(f32),
    Percent(f32),
    Number(f32),
    Boolean(bool),
    Color(u32),
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowValue {
    pub offset_x: f32,
    pub offset_y: f32,
    pub spread: f32,
    pub std_dev: f32,
    pub color: u32,
    pub radius: Option<f32>,
}

pub const GRAPHIC_DATA_VECTOR_PATH: u8 = 0x01;
const MAX_GRAPHIC_DATA_BYTES: usize = 16 * 1024 * 1024;

/// A decoded operation. String operands borrow from the frame buffer
/// (`'a`) so decode is allocation-free; the applier owns them into `String`s.
#[derive(Debug)]
pub enum Op<'a> {
    CreateElement {
        id: NodeKey,
        tag: Atom,
    },
    CreateText {
        id: NodeKey,
        text: &'a str,
    },
    AppendChild {
        parent: NodeKey,
        child: NodeKey,
    },
    InsertBefore {
        parent: NodeKey,
        child: NodeKey,
        ref_id: NodeKey,
    },
    RemoveChild {
        parent: NodeKey,
        child: NodeKey,
    },
    SetText {
        id: NodeKey,
        text: &'a str,
    },
    SetAttribute {
        id: NodeKey,
        name: Atom,
        value: &'a str,
    },
    RemoveAttribute {
        id: NodeKey,
        name: Atom,
    },
    SetWidgetConfig {
        id: NodeKey,
        json: &'a str,
    },
    RemoveWidgetConfig {
        id: NodeKey,
    },
    SetTextBehavior {
        id: NodeKey,
        flags: u8,
    },
    SetInteractionPolicy {
        id: NodeKey,
        flags: u8,
        focus_order: i32,
    },
    SetGraphicSource {
        id: NodeKey,
        kind: u8,
        source: &'a str,
    },
    ClearGraphicSource {
        id: NodeKey,
        kind: u8,
    },
    SetGraphicData {
        id: NodeKey,
        kind: u8,
        data: &'a [u8],
    },
    ClearGraphicData {
        id: NodeKey,
        kind: u8,
    },
    SetStyle {
        id: NodeKey,
        prop: Atom,
        value: &'a str,
    },
    SetStyleValue {
        id: NodeKey,
        prop: Atom,
        value: StyleValue,
    },
    SetShadows {
        id: NodeKey,
        shadows: Vec<ShadowValue>,
    },
    RemoveStyle {
        id: NodeKey,
        prop: Atom,
    },
    AddEventListener {
        id: NodeKey,
        event_type: u8,
    },
    RemoveEventListener {
        id: NodeKey,
        event_type: u8,
    },
    SetClassName {
        id: NodeKey,
        classes: Vec<Atom>,
    },
    DropNode {
        id: NodeKey,
    },
    SetTransform2D {
        id: NodeKey,
        matrix: [f32; 6],
    },
    SetOverlayPlane {
        id: NodeKey,
        plane: u8,
    },
    SetScrollbarStyle {
        id: NodeKey,
        visibility: u8,
        hide_delay: f32,
        fade_duration: f32,
        thickness: f32,
        margin: f32,
        min_thumb_length: f32,
        radius: f32,
        colors: [u32; 4],
    },
    FocusNode {
        id: NodeKey,
    },
    ScrollTo {
        id: NodeKey,
        x: f32,
        y: f32,
    },
    ScrollBy {
        id: NodeKey,
        x: f32,
        y: f32,
    },
}

#[derive(Debug)]
pub struct Frame<'a> {
    pub seq: u32,
    pub ops: Vec<Op<'a>>,
}

#[derive(Debug, snafu::Snafu)]
pub enum DecodeError {
    #[snafu(display("unexpected end of frame"))]
    UnexpectedEof,

    #[snafu(display("unknown opcode 0x{opcode:02x}"))]
    BadOp { opcode: u8 },

    #[snafu(display("invalid node key {lo}v{hi}"))]
    BadNodeKey { lo: u32, hi: u32 },

    #[snafu(display("invalid UTF-8 in string operand: {source}"))]
    BadUtf8 { source: std::str::Utf8Error },

    #[snafu(display("invalid frame-local string reference {index}"))]
    BadStringRef { index: u16 },

    #[snafu(display("unknown typed style value tag {tag}"))]
    BadStyleValue { tag: u8 },

    #[snafu(display("invalid Vello shadow record"))]
    BadShadow,

    #[snafu(display("unknown overlay plane {plane}"))]
    BadOverlayPlane { plane: u8 },

    #[snafu(display("invalid native scrollbar style"))]
    BadScrollbarStyle,

    #[snafu(display("invalid text behavior flags 0x{flags:02x}"))]
    BadTextBehavior { flags: u8 },

    #[snafu(display(
        "invalid interaction policy flags 0x{flags:02x} with focus order {focus_order}"
    ))]
    BadInteractionPolicy { flags: u8, focus_order: i32 },

    #[snafu(display("invalid graphic source kind {kind}"))]
    BadGraphicSourceKind { kind: u8 },

    #[snafu(display("invalid graphic data kind {kind}"))]
    BadGraphicDataKind { kind: u8 },

    #[snafu(display("graphic data length {len} exceeds the protocol limit"))]
    BadGraphicDataLength { len: usize },

    #[snafu(display("protocol frame contains {remaining} trailing bytes"))]
    TrailingBytes { remaining: usize },
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
    strings: Vec<&'a str>,
}

impl<'a> Reader<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self {
            b,
            pos: 0,
            strings: Vec::new(),
        }
    }
    fn u8(&mut self) -> Result<u8, DecodeError> {
        let v = self
            .b
            .get(self.pos)
            .copied()
            .ok_or(DecodeError::UnexpectedEof)?;
        self.pos += 1;
        Ok(v)
    }
    fn u16(&mut self) -> Result<u16, DecodeError> {
        if self.pos + 2 > self.b.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let v = u16::from_le_bytes([self.b[self.pos], self.b[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }
    fn u32(&mut self) -> Result<u32, DecodeError> {
        if self.pos + 4 > self.b.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let v = u32::from_le_bytes([
            self.b[self.pos],
            self.b[self.pos + 1],
            self.b[self.pos + 2],
            self.b[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    fn node_key(&mut self) -> Result<NodeKey, DecodeError> {
        let key = NodeKey::new(self.u32()?, self.u32()?);
        if !key.is_valid() {
            return Err(DecodeError::BadNodeKey {
                lo: key.lo,
                hi: key.hi,
            });
        }
        Ok(key)
    }
    fn i32(&mut self) -> Result<i32, DecodeError> {
        self.u32().map(|value| value as i32)
    }
    fn f32(&mut self) -> Result<f32, DecodeError> {
        self.u32().map(f32::from_bits)
    }
    fn str(&mut self) -> Result<&'a str, DecodeError> {
        let encoded_len = self.u16()?;
        if encoded_len == u16::MAX {
            let index = self.u16()?;
            return self
                .strings
                .get(index as usize)
                .copied()
                .ok_or(DecodeError::BadStringRef { index });
        }
        let len = encoded_len as usize;
        if self.pos + len > self.b.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let s = std::str::from_utf8(&self.b[self.pos..self.pos + len])
            .map_err(|source| DecodeError::BadUtf8 { source })?;
        self.pos += len;
        if len >= 4 && self.strings.len() <= u16::MAX as usize {
            self.strings.push(s);
        }
        Ok(s)
    }
    fn bytes(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        if self.pos + len > self.b.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let bytes = &self.b[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }
}

pub fn decode_frame(buf: &[u8]) -> Result<Frame<'_>, DecodeError> {
    let mut r = Reader::new(buf);
    let seq = r.u32()?;
    let count = r.u32()?;
    let mut ops = Vec::with_capacity(count as usize);
    for _ in 0..count {
        ops.push(decode_op(&mut r)?);
    }
    if r.pos != buf.len() {
        return Err(DecodeError::TrailingBytes {
            remaining: buf.len() - r.pos,
        });
    }
    Ok(Frame { seq, ops })
}

fn decode_op<'a>(r: &mut Reader<'a>) -> Result<Op<'a>, DecodeError> {
    let code = r.u8()?;
    Ok(match code {
        op::CREATE_ELEMENT => {
            let id = r.node_key()?;
            let tag = Atom::from_raw(r.u32()?);
            Op::CreateElement { id, tag }
        }
        op::CREATE_TEXT => {
            let id = r.node_key()?;
            let text = r.str()?;
            Op::CreateText { id, text }
        }
        op::APPEND_CHILD => {
            let parent = r.node_key()?;
            let child = r.node_key()?;
            Op::AppendChild { parent, child }
        }
        op::INSERT_BEFORE => {
            let parent = r.node_key()?;
            let child = r.node_key()?;
            let ref_id = r.node_key()?;
            Op::InsertBefore {
                parent,
                child,
                ref_id,
            }
        }
        op::REMOVE_CHILD => {
            let parent = r.node_key()?;
            let child = r.node_key()?;
            Op::RemoveChild { parent, child }
        }
        op::SET_TEXT => {
            let id = r.node_key()?;
            let text = r.str()?;
            Op::SetText { id, text }
        }
        op::SET_ATTRIBUTE => {
            let id = r.node_key()?;
            let name = Atom::from_raw(r.u32()?);
            let value = r.str()?;
            Op::SetAttribute { id, name, value }
        }
        op::REMOVE_ATTRIBUTE => {
            let id = r.node_key()?;
            let name = Atom::from_raw(r.u32()?);
            Op::RemoveAttribute { id, name }
        }
        op::SET_WIDGET_CONFIG => {
            let id = r.node_key()?;
            let json = r.str()?;
            Op::SetWidgetConfig { id, json }
        }
        op::REMOVE_WIDGET_CONFIG => {
            let id = r.node_key()?;
            Op::RemoveWidgetConfig { id }
        }
        op::SET_TEXT_BEHAVIOR => {
            let id = r.node_key()?;
            let flags = r.u8()?;
            if flags & !TEXT_BEHAVIOR_MASK != 0 {
                return Err(DecodeError::BadTextBehavior { flags });
            }
            Op::SetTextBehavior { id, flags }
        }
        op::SET_INTERACTION_POLICY => {
            let id = r.node_key()?;
            let flags = r.u8()?;
            let focus_order = r.i32()?;
            if flags & !INTERACTION_POLICY_MASK != 0
                || flags & INTERACTION_POLICY_FOCUSABLE == 0 && focus_order != 0
            {
                return Err(DecodeError::BadInteractionPolicy { flags, focus_order });
            }
            Op::SetInteractionPolicy {
                id,
                flags,
                focus_order,
            }
        }
        op::SET_GRAPHIC_SOURCE => {
            let id = r.node_key()?;
            let kind = r.u8()?;
            if !valid_graphic_source_kind(kind) {
                return Err(DecodeError::BadGraphicSourceKind { kind });
            }
            let source = r.str()?;
            Op::SetGraphicSource { id, kind, source }
        }
        op::CLEAR_GRAPHIC_SOURCE => {
            let id = r.node_key()?;
            let kind = r.u8()?;
            if !valid_graphic_source_kind(kind) {
                return Err(DecodeError::BadGraphicSourceKind { kind });
            }
            Op::ClearGraphicSource { id, kind }
        }
        op::SET_GRAPHIC_DATA => {
            let id = r.node_key()?;
            let kind = r.u8()?;
            if kind != GRAPHIC_DATA_VECTOR_PATH {
                return Err(DecodeError::BadGraphicDataKind { kind });
            }
            let len = r.u32()? as usize;
            if len > MAX_GRAPHIC_DATA_BYTES {
                return Err(DecodeError::BadGraphicDataLength { len });
            }
            let data = r.bytes(len)?;
            Op::SetGraphicData { id, kind, data }
        }
        op::CLEAR_GRAPHIC_DATA => {
            let id = r.node_key()?;
            let kind = r.u8()?;
            if kind != GRAPHIC_DATA_VECTOR_PATH {
                return Err(DecodeError::BadGraphicDataKind { kind });
            }
            Op::ClearGraphicData { id, kind }
        }
        op::SET_STYLE => {
            let id = r.node_key()?;
            let prop = Atom::from_raw(r.u32()?);
            let value = r.str()?;
            Op::SetStyle { id, prop, value }
        }
        op::SET_STYLE_VALUE => {
            let id = r.node_key()?;
            let prop = Atom::from_raw(r.u32()?);
            let tag = r.u8()?;
            let value = match tag {
                1 => StyleValue::Px(r.f32()?),
                2 => StyleValue::Percent(r.f32()?),
                3 => StyleValue::Number(r.f32()?),
                4 => StyleValue::Boolean(r.u8()? != 0),
                5 => StyleValue::Color(r.u32()?),
                6 => StyleValue::Auto,
                tag => return Err(DecodeError::BadStyleValue { tag }),
            };
            Op::SetStyleValue { id, prop, value }
        }
        op::SET_SHADOWS => {
            let id = r.node_key()?;
            let count = r.u16()?;
            let mut shadows = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let offset_x = r.f32()?;
                let offset_y = r.f32()?;
                let spread = r.f32()?;
                let std_dev = r.f32()?;
                let color = r.u32()?;
                let radius = r.f32()?;
                if !offset_x.is_finite()
                    || !offset_y.is_finite()
                    || !spread.is_finite()
                    || !std_dev.is_finite()
                    || std_dev < 0.0
                    || (!radius.is_nan() && (!radius.is_finite() || radius < 0.0))
                {
                    return Err(DecodeError::BadShadow);
                }
                shadows.push(ShadowValue {
                    offset_x,
                    offset_y,
                    spread,
                    std_dev,
                    color,
                    radius: (!radius.is_nan()).then_some(radius),
                });
            }
            Op::SetShadows { id, shadows }
        }
        op::REMOVE_STYLE => {
            let id = r.node_key()?;
            let prop = Atom::from_raw(r.u32()?);
            Op::RemoveStyle { id, prop }
        }
        op::ADD_EVENT_LISTENER => {
            let id = r.node_key()?;
            let event_type = r.u8()?;
            Op::AddEventListener { id, event_type }
        }
        op::REMOVE_EVENT_LISTENER => {
            let id = r.node_key()?;
            let event_type = r.u8()?;
            Op::RemoveEventListener { id, event_type }
        }
        op::SET_CLASS_NAME => {
            let id = r.node_key()?;
            let count = r.u16()?;
            let mut classes = Vec::with_capacity(count as usize);
            for _ in 0..count {
                classes.push(Atom::from_raw(r.u32()?));
            }
            Op::SetClassName { id, classes }
        }
        op::DROP_NODE => {
            let id = r.node_key()?;
            Op::DropNode { id }
        }
        op::SET_TRANSFORM2_D => {
            let id = r.node_key()?;
            let matrix = [r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?];
            Op::SetTransform2D { id, matrix }
        }
        op::SET_OVERLAY_PLANE => {
            let id = r.node_key()?;
            let plane = r.u8()?;
            if plane > 2 {
                return Err(DecodeError::BadOverlayPlane { plane });
            }
            Op::SetOverlayPlane { id, plane }
        }
        op::SET_SCROLLBAR_STYLE => {
            let id = r.node_key()?;
            let visibility = r.u8()?;
            let hide_delay = r.f32()?;
            let fade_duration = r.f32()?;
            let thickness = r.f32()?;
            let margin = r.f32()?;
            let min_thumb_length = r.f32()?;
            let radius = r.f32()?;
            if visibility > 2
                || !hide_delay.is_finite()
                || !(0.0..=86_400_000.0).contains(&hide_delay)
                || !fade_duration.is_finite()
                || !(0.0..=86_400_000.0).contains(&fade_duration)
                || !thickness.is_finite()
                || thickness <= 0.0
                || !margin.is_finite()
                || margin < 0.0
                || !min_thumb_length.is_finite()
                || min_thumb_length <= 0.0
                || !radius.is_finite()
            {
                return Err(DecodeError::BadScrollbarStyle);
            }
            Op::SetScrollbarStyle {
                id,
                visibility,
                hide_delay,
                fade_duration,
                thickness,
                margin,
                min_thumb_length,
                radius,
                colors: [r.u32()?, r.u32()?, r.u32()?, r.u32()?],
            }
        }
        op::FOCUS_NODE => {
            let id = r.node_key()?;
            Op::FocusNode { id }
        }
        op::SCROLL_TO => {
            let id = r.node_key()?;
            let x = r.f32()?;
            let y = r.f32()?;
            Op::ScrollTo { id, x, y }
        }
        op::SCROLL_BY => {
            let id = r.node_key()?;
            let x = r.f32()?;
            let y = r.f32()?;
            Op::ScrollBy { id, x, y }
        }
        other => return Err(DecodeError::BadOp { opcode: other }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_node(bytes: &mut Vec<u8>, lo: u32) {
        push_u32(bytes, lo);
        push_u32(bytes, 1);
    }

    #[test]
    fn decodes_frame_local_string_references() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1); // sequence
        push_u32(&mut bytes, 2); // op count
        bytes.push(op::CREATE_TEXT);
        push_node(&mut bytes, 1);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice("🚀".as_bytes());
        bytes.push(op::CREATE_TEXT);
        push_u32(&mut bytes, 2);
        push_u32(&mut bytes, 3);
        bytes.extend_from_slice(&u16::MAX.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::CreateText {
                id: NodeKey { lo: 1, hi: 1 },
                text: "🚀"
            }
        ));
        assert!(matches!(
            &frame.ops[1],
            Op::CreateText {
                id: NodeKey { lo: 2, hi: 3 },
                text: "🚀"
            }
        ));
    }

    #[test]
    fn create_element_has_no_legacy_attribute_payload() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::CREATE_ELEMENT);
        push_node(&mut bytes, 42);
        push_u32(&mut bytes, 7);

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::CreateElement { id: NodeKey { lo: 42, hi: 1 }, tag } if tag.get() == 7
        ));

        bytes.extend_from_slice(&0_u16.to_le_bytes());
        assert!(matches!(
            decode_frame(&bytes),
            Err(DecodeError::TrailingBytes { remaining: 2 })
        ));
    }

    #[test]
    fn rejects_removed_opcodes() {
        let frame = |opcode| {
            let mut bytes = Vec::new();
            push_u32(&mut bytes, 1);
            push_u32(&mut bytes, 1);
            bytes.push(opcode);
            bytes
        };

        assert!(matches!(
            decode_frame(&frame(0x03)),
            Err(DecodeError::BadOp { opcode: 0x03 })
        ));
        assert!(matches!(
            decode_frame(&frame(0x10)),
            Err(DecodeError::BadOp { opcode: 0x10 })
        ));
        assert!(matches!(
            decode_frame(&frame(0x07)),
            Err(DecodeError::BadOp { opcode: 0x07 })
        ));
    }

    #[test]
    fn rejects_unknown_frame_local_string_reference() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::CREATE_TEXT);
        push_node(&mut bytes, 1);
        bytes.extend_from_slice(&u16::MAX.to_le_bytes());
        bytes.extend_from_slice(&7u16.to_le_bytes());

        assert!(matches!(
            decode_frame(&bytes),
            Err(DecodeError::BadStringRef { index: 7 })
        ));
    }

    #[test]
    fn decodes_imperative_focus_target() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::FOCUS_NODE);
        push_node(&mut bytes, 42);

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::FocusNode {
                id: NodeKey { lo: 42, hi: 1 }
            }
        ));
    }

    #[test]
    fn decodes_native_scrollbar_style_without_strings() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::SET_SCROLLBAR_STYLE);
        push_node(&mut bytes, 42);
        bytes.push(1);
        for value in [700.0_f32, 160.0, 14.0, 3.0, 40.0, 5.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for color in [0x11182788, 0x38bdf8ff, 0x7dd3fcff, 0x0284c7ff] {
            push_u32(&mut bytes, color);
        }

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetScrollbarStyle {
                id: NodeKey { lo: 42, hi: 1 },
                visibility: 1,
                hide_delay: 700.0,
                fade_duration: 160.0,
                thickness: 14.0,
                margin: 3.0,
                min_thumb_length: 40.0,
                radius: 5.0,
                colors: [0x11182788, 0x38bdf8ff, 0x7dd3fcff, 0x0284c7ff],
            }
        ));
    }

    #[test]
    fn decodes_widget_config_json() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::SET_WIDGET_CONFIG);
        push_node(&mut bytes, 42);
        let json = br##"{"caret":"#fff"}"##;
        bytes.extend_from_slice(&(json.len() as u16).to_le_bytes());
        bytes.extend_from_slice(json);

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetWidgetConfig {
                id: NodeKey { lo: 42, hi: 1 },
                json: r##"{"caret":"#fff"}"##
            }
        ));
    }

    #[test]
    fn decodes_and_validates_typed_text_behavior() {
        let frame_bytes = |flags| {
            let mut bytes = Vec::new();
            push_u32(&mut bytes, 1);
            push_u32(&mut bytes, 1);
            bytes.push(op::SET_TEXT_BEHAVIOR);
            push_node(&mut bytes, 42);
            bytes.push(flags);
            bytes
        };

        let valid = frame_bytes(0x03);
        let frame = decode_frame(&valid).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetTextBehavior {
                id: NodeKey { lo: 42, hi: 1 },
                flags: 0x03
            }
        ));
        assert!(matches!(
            decode_frame(&frame_bytes(0x04)),
            Err(DecodeError::BadTextBehavior { flags: 0x04 })
        ));
    }

    #[test]
    fn decodes_and_validates_interaction_policy() {
        let frame_bytes = |flags, focus_order: i32| {
            let mut bytes = Vec::new();
            push_u32(&mut bytes, 1);
            push_u32(&mut bytes, 1);
            bytes.push(op::SET_INTERACTION_POLICY);
            push_node(&mut bytes, 42);
            bytes.push(flags);
            push_u32(&mut bytes, focus_order as u32);
            bytes
        };

        let valid = frame_bytes(INTERACTION_POLICY_FOCUSABLE, -1);
        let frame = decode_frame(&valid).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetInteractionPolicy {
                id: NodeKey { lo: 42, hi: 1 },
                flags: INTERACTION_POLICY_FOCUSABLE,
                focus_order: -1,
            }
        ));
        assert!(matches!(
            decode_frame(&frame_bytes(0, 1)),
            Err(DecodeError::BadInteractionPolicy {
                flags: 0,
                focus_order: 1,
            })
        ));
        assert!(matches!(
            decode_frame(&frame_bytes(0x08, 0)),
            Err(DecodeError::BadInteractionPolicy {
                flags: 0x08,
                focus_order: 0,
            })
        ));
    }

    #[test]
    fn decodes_and_validates_graphic_sources() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 2);
        bytes.push(op::SET_GRAPHIC_SOURCE);
        push_node(&mut bytes, 42);
        bytes.push(GRAPHIC_SOURCE_NETWORK_RASTER);
        let source = b"https://x.test/a.png";
        bytes.extend_from_slice(&(source.len() as u16).to_le_bytes());
        bytes.extend_from_slice(source);
        bytes.push(op::CLEAR_GRAPHIC_SOURCE);
        push_node(&mut bytes, 42);
        bytes.push(GRAPHIC_SOURCE_NETWORK_RASTER);

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetGraphicSource {
                id: NodeKey { lo: 42, hi: 1 },
                kind: GRAPHIC_SOURCE_NETWORK_RASTER,
                source: "https://x.test/a.png",
            }
        ));
        assert!(matches!(
            &frame.ops[1],
            Op::ClearGraphicSource {
                id: NodeKey { lo: 42, hi: 1 },
                kind: GRAPHIC_SOURCE_NETWORK_RASTER,
            }
        ));

        bytes[17] = 3;
        assert!(matches!(
            decode_frame(&bytes),
            Err(DecodeError::BadGraphicSourceKind { kind: 3 })
        ));
    }

    #[test]
    fn decodes_length_delimited_graphic_data() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 2);
        bytes.push(op::SET_GRAPHIC_DATA);
        push_node(&mut bytes, 42);
        bytes.push(GRAPHIC_DATA_VECTOR_PATH);
        push_u32(&mut bytes, 3);
        bytes.extend_from_slice(&[7, 8, 9]);
        bytes.push(op::CLEAR_GRAPHIC_DATA);
        push_node(&mut bytes, 42);
        bytes.push(GRAPHIC_DATA_VECTOR_PATH);

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetGraphicData {
                id: NodeKey { lo: 42, hi: 1 },
                kind: GRAPHIC_DATA_VECTOR_PATH,
                data: [7, 8, 9],
            }
        ));
        assert!(matches!(
            &frame.ops[1],
            Op::ClearGraphicData {
                id: NodeKey { lo: 42, hi: 1 },
                kind: GRAPHIC_DATA_VECTOR_PATH,
            }
        ));

        bytes[17] = 2;
        assert!(matches!(
            decode_frame(&bytes),
            Err(DecodeError::BadGraphicDataKind { kind: 2 })
        ));
    }

    #[test]
    fn decodes_typed_style_without_utf8() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::SET_STYLE_VALUE);
        push_node(&mut bytes, 7);
        push_u32(&mut bytes, 9);
        bytes.push(1);
        push_u32(&mut bytes, 12.5f32.to_bits());

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetStyleValue {
                id: NodeKey { lo: 7, hi: 1 },
                prop,
                value: StyleValue::Px(value),
            } if prop.get() == 9 && *value == 12.5
        ));
    }

    #[test]
    fn decodes_ordered_vello_shadow_records() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::SET_SHADOWS);
        push_node(&mut bytes, 7);
        bytes.extend_from_slice(&1u16.to_le_bytes());
        for value in [1.0_f32, 2.0, -3.0, 4.5] {
            push_u32(&mut bytes, value.to_bits());
        }
        push_u32(&mut bytes, 0x336699cc);
        push_u32(&mut bytes, 8.0_f32.to_bits());

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetShadows { id: NodeKey { lo: 7, hi: 1 }, shadows }
                if shadows == &[ShadowValue {
                    offset_x: 1.0,
                    offset_y: 2.0,
                    spread: -3.0,
                    std_dev: 4.5,
                    color: 0x336699cc,
                    radius: Some(8.0),
                }]
        ));
    }
}
