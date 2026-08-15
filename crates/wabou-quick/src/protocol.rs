//! Binary bridge wire protocol (JS SolidJS reconciler → Rust).
//!
//! Frame layout (all little-endian):
//!   [seq: u32][count: u32][op...]
//! Each op: [op: u8][operands...]. Strings are either `[len: u16][utf8]` or
//! `[0xffff][frame-local string index: u16]`. Inline strings of at least four
//! bytes enter the frame-local table; the table is discarded after decoding.
//! Structural strings are runtime-scoped [`crate::Atom`] IDs (`u32`); dynamic
//! values and text use inline UTF-8 plus frame-local references.
//! Node ids are u32; 0 is the "none / append" sentinel.
//!
//! Decoder copied from blitz-js (host-agnostic); op constants are generated
//! from `packages/protocol/src/index.ts` by `scripts/gen-rust-op.ts`.

#![allow(dead_code)]

include!("gen/op.rs");

use crate::Atom;

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

/// A decoded operation. String operands borrow from the frame buffer
/// (`'a`) so decode is allocation-free; the applier owns them into `String`s.
#[derive(Debug)]
pub enum Op<'a> {
    CreateElement {
        id: u32,
        tag: Atom,
        attrs: Vec<(Atom, &'a str)>,
    },
    CreateText {
        id: u32,
        text: &'a str,
    },
    CreateComment {
        id: u32,
        text: &'a str,
    },
    AppendChild {
        parent: u32,
        child: u32,
    },
    InsertBefore {
        parent: u32,
        child: u32,
        ref_id: u32,
    },
    RemoveChild {
        parent: u32,
        child: u32,
    },
    ReplaceNode {
        parent: u32,
        old_id: u32,
        new_id: u32,
    },
    SetText {
        id: u32,
        text: &'a str,
    },
    SetAttribute {
        id: u32,
        name: Atom,
        value: &'a str,
    },
    RemoveAttribute {
        id: u32,
        name: Atom,
    },
    SetWidgetConfig {
        id: u32,
        json: &'a str,
    },
    RemoveWidgetConfig {
        id: u32,
    },
    SetStyle {
        id: u32,
        prop: Atom,
        value: &'a str,
    },
    SetStyleValue {
        id: u32,
        prop: Atom,
        value: StyleValue,
    },
    SetShadows {
        id: u32,
        shadows: Vec<ShadowValue>,
    },
    RemoveStyle {
        id: u32,
        prop: Atom,
    },
    AddEventListener {
        id: u32,
        event_type: u8,
    },
    RemoveEventListener {
        id: u32,
        event_type: u8,
    },
    SetClassName {
        id: u32,
        classes: Vec<Atom>,
    },
    FrameEnd,
    DropNode {
        id: u32,
    },
    SetTransform2D {
        id: u32,
        matrix: [f32; 6],
    },
    SetOverlayPlane {
        id: u32,
        plane: u8,
    },
    SetScrollbarStyle {
        id: u32,
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
        id: u32,
    },
    ScrollTo {
        id: u32,
        x: f32,
        y: f32,
    },
    ScrollBy {
        id: u32,
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
}

pub fn decode_frame(buf: &[u8]) -> Result<Frame<'_>, DecodeError> {
    let mut r = Reader::new(buf);
    let seq = r.u32()?;
    let count = r.u32()?;
    let mut ops = Vec::with_capacity(count as usize);
    for _ in 0..count {
        ops.push(decode_op(&mut r)?);
    }
    Ok(Frame { seq, ops })
}

fn decode_op<'a>(r: &mut Reader<'a>) -> Result<Op<'a>, DecodeError> {
    let code = r.u8()?;
    Ok(match code {
        op::CREATE_ELEMENT => {
            let id = r.u32()?;
            let tag = Atom::from_raw(r.u32()?);
            let n_attr = r.u16()?;
            let mut attrs = Vec::with_capacity(n_attr as usize);
            for _ in 0..n_attr {
                let name = Atom::from_raw(r.u32()?);
                let value = r.str()?;
                attrs.push((name, value));
            }
            Op::CreateElement { id, tag, attrs }
        }
        op::CREATE_TEXT => {
            let id = r.u32()?;
            let text = r.str()?;
            Op::CreateText { id, text }
        }
        op::CREATE_COMMENT => {
            let id = r.u32()?;
            let text = r.str()?;
            Op::CreateComment { id, text }
        }
        op::APPEND_CHILD => {
            let parent = r.u32()?;
            let child = r.u32()?;
            Op::AppendChild { parent, child }
        }
        op::INSERT_BEFORE => {
            let parent = r.u32()?;
            let child = r.u32()?;
            let ref_id = r.u32()?;
            Op::InsertBefore {
                parent,
                child,
                ref_id,
            }
        }
        op::REMOVE_CHILD => {
            let parent = r.u32()?;
            let child = r.u32()?;
            Op::RemoveChild { parent, child }
        }
        op::REPLACE_NODE => {
            let parent = r.u32()?;
            let old_id = r.u32()?;
            let new_id = r.u32()?;
            Op::ReplaceNode {
                parent,
                old_id,
                new_id,
            }
        }
        op::SET_TEXT => {
            let id = r.u32()?;
            let text = r.str()?;
            Op::SetText { id, text }
        }
        op::SET_ATTRIBUTE => {
            let id = r.u32()?;
            let name = Atom::from_raw(r.u32()?);
            let value = r.str()?;
            Op::SetAttribute { id, name, value }
        }
        op::REMOVE_ATTRIBUTE => {
            let id = r.u32()?;
            let name = Atom::from_raw(r.u32()?);
            Op::RemoveAttribute { id, name }
        }
        op::SET_WIDGET_CONFIG => {
            let id = r.u32()?;
            let json = r.str()?;
            Op::SetWidgetConfig { id, json }
        }
        op::REMOVE_WIDGET_CONFIG => {
            let id = r.u32()?;
            Op::RemoveWidgetConfig { id }
        }
        op::SET_STYLE => {
            let id = r.u32()?;
            let prop = Atom::from_raw(r.u32()?);
            let value = r.str()?;
            Op::SetStyle { id, prop, value }
        }
        op::SET_STYLE_VALUE => {
            let id = r.u32()?;
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
            let id = r.u32()?;
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
            let id = r.u32()?;
            let prop = Atom::from_raw(r.u32()?);
            Op::RemoveStyle { id, prop }
        }
        op::ADD_EVENT_LISTENER => {
            let id = r.u32()?;
            let event_type = r.u8()?;
            Op::AddEventListener { id, event_type }
        }
        op::REMOVE_EVENT_LISTENER => {
            let id = r.u32()?;
            let event_type = r.u8()?;
            Op::RemoveEventListener { id, event_type }
        }
        op::SET_CLASS_NAME => {
            let id = r.u32()?;
            let count = r.u16()?;
            let mut classes = Vec::with_capacity(count as usize);
            for _ in 0..count {
                classes.push(Atom::from_raw(r.u32()?));
            }
            Op::SetClassName { id, classes }
        }
        op::FRAME_END => Op::FrameEnd,
        op::DROP_NODE => {
            let id = r.u32()?;
            Op::DropNode { id }
        }
        op::SET_TRANSFORM2_D => {
            let id = r.u32()?;
            let matrix = [r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?];
            Op::SetTransform2D { id, matrix }
        }
        op::SET_OVERLAY_PLANE => {
            let id = r.u32()?;
            let plane = r.u8()?;
            if plane > 2 {
                return Err(DecodeError::BadOverlayPlane { plane });
            }
            Op::SetOverlayPlane { id, plane }
        }
        op::SET_SCROLLBAR_STYLE => {
            let id = r.u32()?;
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
            let id = r.u32()?;
            Op::FocusNode { id }
        }
        op::SCROLL_TO => {
            let id = r.u32()?;
            let x = r.f32()?;
            let y = r.f32()?;
            Op::ScrollTo { id, x, y }
        }
        op::SCROLL_BY => {
            let id = r.u32()?;
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

    #[test]
    fn decodes_frame_local_string_references() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1); // sequence
        push_u32(&mut bytes, 2); // op count
        bytes.push(op::CREATE_TEXT);
        push_u32(&mut bytes, 1);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice("🚀".as_bytes());
        bytes.push(op::CREATE_TEXT);
        push_u32(&mut bytes, 2);
        bytes.extend_from_slice(&u16::MAX.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::CreateText {
                id: 1, text: "🚀"
            }
        ));
        assert!(matches!(
            &frame.ops[1],
            Op::CreateText {
                id: 2, text: "🚀"
            }
        ));
    }

    #[test]
    fn rejects_unknown_frame_local_string_reference() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::CREATE_TEXT);
        push_u32(&mut bytes, 1);
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
        push_u32(&mut bytes, 42);

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(&frame.ops[0], Op::FocusNode { id: 42 }));
    }

    #[test]
    fn decodes_native_scrollbar_style_without_strings() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::SET_SCROLLBAR_STYLE);
        push_u32(&mut bytes, 42);
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
                id: 42,
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
        push_u32(&mut bytes, 42);
        let json = br##"{"caret":"#fff"}"##;
        bytes.extend_from_slice(&(json.len() as u16).to_le_bytes());
        bytes.extend_from_slice(json);

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetWidgetConfig {
                id: 42,
                json: r##"{"caret":"#fff"}"##
            }
        ));
    }

    #[test]
    fn decodes_typed_style_without_utf8() {
        let mut bytes = Vec::new();
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        bytes.push(op::SET_STYLE_VALUE);
        push_u32(&mut bytes, 7);
        push_u32(&mut bytes, 9);
        bytes.push(1);
        push_u32(&mut bytes, 12.5f32.to_bits());

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetStyleValue {
                id: 7,
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
        push_u32(&mut bytes, 7);
        bytes.extend_from_slice(&1u16.to_le_bytes());
        for value in [1.0_f32, 2.0, -3.0, 4.5] {
            push_u32(&mut bytes, value.to_bits());
        }
        push_u32(&mut bytes, 0x336699cc);
        push_u32(&mut bytes, 8.0_f32.to_bits());

        let frame = decode_frame(&bytes).unwrap();
        assert!(matches!(
            &frame.ops[0],
            Op::SetShadows { id: 7, shadows }
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
