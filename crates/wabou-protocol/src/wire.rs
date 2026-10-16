//! Rust-owned numeric definitions for the cross-language wire protocol.
//!
//! Keep numeric assignments stable. TypeScript consumers are generated from
//! [`typescript_wire_constants`] and must never redeclare these values.

#[derive(Clone, Copy)]
struct WireConstant {
    ts_name: &'static str,
    value: u32,
}

macro_rules! wire_module {
    ($module:ident, $entries:ident, $ty:ty, { $($rust:ident => $ts:literal = $value:expr),+ $(,)? }) => {
        #[allow(missing_docs)]
        pub mod $module {
            $(pub const $rust: $ty = $value;)+
        }

        const $entries: &[WireConstant] = &[
            $(WireConstant { ts_name: $ts, value: $value as u32 },)+
        ];
    };
}

wire_module!(op, OP, u8, {
    CREATE_ELEMENT => "CreateElement" = 0x01,
    CREATE_TEXT => "CreateText" = 0x02,
    APPEND_CHILD => "AppendChild" = 0x04,
    INSERT_BEFORE => "InsertBefore" = 0x05,
    REMOVE_CHILD => "RemoveChild" = 0x06,
    SET_TEXT => "SetText" = 0x08,
    SET_ATTRIBUTE => "SetAttribute" = 0x09,
    REMOVE_ATTRIBUTE => "RemoveAttribute" = 0x0a,
    SET_STYLE => "SetStyle" = 0x0b,
    REMOVE_STYLE => "RemoveStyle" = 0x0c,
    ADD_EVENT_LISTENER => "AddEventListener" = 0x0d,
    REMOVE_EVENT_LISTENER => "RemoveEventListener" = 0x0e,
    SET_CLASS_NAME => "SetClassName" = 0x0f,
    DROP_NODE => "DropNode" = 0x11,
    SET_TRANSFORM2_D => "SetTransform2D" = 0x12,
    FOCUS_NODE => "FocusNode" = 0x13,
    SCROLL_TO => "ScrollTo" = 0x14,
    SCROLL_BY => "ScrollBy" = 0x15,
    SET_STYLE_VALUE => "SetStyleValue" = 0x16,
    SET_SHADOWS => "SetShadows" = 0x17,
    SET_OVERLAY_PLANE => "SetOverlayPlane" = 0x18,
    SET_SCROLLBAR_STYLE => "SetScrollbarStyle" = 0x19,
    SET_WIDGET_CONFIG => "SetWidgetConfig" = 0x1a,
    REMOVE_WIDGET_CONFIG => "RemoveWidgetConfig" = 0x1b,
    SET_TEXT_BEHAVIOR => "SetTextBehavior" = 0x1c,
    SET_INTERACTION_POLICY => "SetInteractionPolicy" = 0x1d,
    SET_GRAPHIC_SOURCE => "SetGraphicSource" = 0x1e,
    CLEAR_GRAPHIC_SOURCE => "ClearGraphicSource" = 0x1f,
    SET_GRAPHIC_DATA => "SetGraphicData" = 0x20,
    CLEAR_GRAPHIC_DATA => "ClearGraphicData" = 0x21,
    SET_TEXT_MAX_LINES => "SetTextMaxLines" = 0x22,
    SET_TEXT_SELECTION => "SetTextSelection" = 0x23,
    TEXT_COMMAND => "TextCommand" = 0x24,
    BLUR_NODE => "BlurNode" = 0x25,
    SET_PROJECTION_BOUNDARY => "SetProjectionBoundary" = 0x26,
    ACKNOWLEDGE_TEXT_VALUE => "AcknowledgeTextValue" = 0x27,
});

wire_module!(event, EVENT, u8, {
    CLICK => "click" = 1,
    INPUT => "input" = 2,
    SUBMIT => "submit" = 3,
    KEYDOWN => "keydown" = 4,
    KEYUP => "keyup" = 5,
    CHANGE => "change" = 6,
    POINTERDOWN => "pointerdown" = 7,
    POINTERMOVE => "pointermove" = 8,
    POINTERUP => "pointerup" = 9,
    POINTERENTER => "pointerenter" = 10,
    POINTERLEAVE => "pointerleave" = 11,
    WHEEL => "wheel" = 12,
    FOCUS => "focus" = 13,
    BLUR => "blur" = 14,
    IMECOMMIT => "imecommit" = 15,
    POINTERCANCEL => "pointercancel" = 16,
    POINTEROVER => "pointerover" = 17,
    POINTEROUT => "pointerout" = 18,
    CONTEXTMENU => "contextmenu" = 19,
    DBLCLICK => "dblclick" = 20,
    FOCUSIN => "focusin" = 21,
    FOCUSOUT => "focusout" = 22,
    SCROLL => "scroll" = 23,
    TERMINALEXIT => "terminalexit" = 24,
    TERMINALPROGRESS => "terminalprogress" = 25,
    TERMINALNOTIFICATION => "terminalnotification" = 26,
    TERMINALTITLECHANGE => "terminaltitlechange" = 27,
    TERMINALCWDCHANGE => "terminalcwdchange" = 28,
    TERMINALSELECTIONCHANGE => "terminalselectionchange" = 29,
    TEXTSELECTIONCHANGE => "textselectionchange" = 30,
    TERMINALBELL => "terminalbell" = 31,
    RESOURCEREADY => "resourceready" = 32,
    RESOURCEERROR => "resourceerror" = 33,
    IMEENABLED => "imeenabled" = 34,
    IMEPREEDIT => "imepreedit" = 35,
    IMEDELETESURROUNDING => "imedeletesurrounding" = 36,
    IMEDISABLED => "imedisabled" = 37,
    WINDOWCLOSEREQUESTED => "windowcloserequested" = 38,
    TRANSITIONEND => "transitionend" = 39,
    SECRETSTATECHANGE => "secretstatechange" = 40,
});

#[allow(missing_docs)]
pub mod event_data {
    pub const CLIENT_X: u8 = 0;
    pub const CLIENT_Y: u8 = 1;
    pub const OFFSET_X: u8 = 2;
    pub const OFFSET_Y: u8 = 3;
    pub const BUTTON: u8 = 4;
    pub const BUTTONS: u8 = 5;
    pub const MODS: u8 = 6;
    pub const DELTA_X: u8 = 7;
    pub const DELTA_Y: u8 = 8;
    pub const SCROLL_X: u8 = 9;
    pub const SCROLL_Y: u8 = 10;
    pub const PHASE: u8 = 11;
    pub const POINTER_ID_LO: u8 = 12;
    pub const POINTER_ID_HI: u8 = 13;
    pub const POINTER_TYPE: u8 = 14;
    pub const PRIMARY: u8 = 15;
    pub const PRESSURE: u8 = 16;
    pub const TANGENTIAL_PRESSURE: u8 = 17;
    pub const TILT_X: u8 = 18;
    pub const TILT_Y: u8 = 19;
    pub const TWIST: u8 = 20;

    /// Number of numeric slots in a complete event record.
    pub const LEN: usize = super::EVENT_DATA.len();
}

const EVENT_DATA: &[WireConstant] = &[
    WireConstant {
        ts_name: "clientX",
        value: event_data::CLIENT_X as u32,
    },
    WireConstant {
        ts_name: "clientY",
        value: event_data::CLIENT_Y as u32,
    },
    WireConstant {
        ts_name: "offsetX",
        value: event_data::OFFSET_X as u32,
    },
    WireConstant {
        ts_name: "offsetY",
        value: event_data::OFFSET_Y as u32,
    },
    WireConstant {
        ts_name: "button",
        value: event_data::BUTTON as u32,
    },
    WireConstant {
        ts_name: "buttons",
        value: event_data::BUTTONS as u32,
    },
    WireConstant {
        ts_name: "mods",
        value: event_data::MODS as u32,
    },
    WireConstant {
        ts_name: "deltaX",
        value: event_data::DELTA_X as u32,
    },
    WireConstant {
        ts_name: "deltaY",
        value: event_data::DELTA_Y as u32,
    },
    WireConstant {
        ts_name: "scrollX",
        value: event_data::SCROLL_X as u32,
    },
    WireConstant {
        ts_name: "scrollY",
        value: event_data::SCROLL_Y as u32,
    },
    WireConstant {
        ts_name: "phase",
        value: event_data::PHASE as u32,
    },
    WireConstant {
        ts_name: "pointerIdLo",
        value: event_data::POINTER_ID_LO as u32,
    },
    WireConstant {
        ts_name: "pointerIdHi",
        value: event_data::POINTER_ID_HI as u32,
    },
    WireConstant {
        ts_name: "pointerType",
        value: event_data::POINTER_TYPE as u32,
    },
    WireConstant {
        ts_name: "primary",
        value: event_data::PRIMARY as u32,
    },
    WireConstant {
        ts_name: "pressure",
        value: event_data::PRESSURE as u32,
    },
    WireConstant {
        ts_name: "tangentialPressure",
        value: event_data::TANGENTIAL_PRESSURE as u32,
    },
    WireConstant {
        ts_name: "tiltX",
        value: event_data::TILT_X as u32,
    },
    WireConstant {
        ts_name: "tiltY",
        value: event_data::TILT_Y as u32,
    },
    WireConstant {
        ts_name: "twist",
        value: event_data::TWIST as u32,
    },
];

wire_module!(host_frame, HOST_FRAME, u32, {
    MAGIC => "Magic" = 0x3146_4857,
    VERSION => "Version" = 3,
    HEADER_LEN => "HeaderLen" = 32,
});

wire_module!(host_record, HOST_RECORD, u8, {
    NODE_EVENT => "NodeEvent" = 1,
    RESIZE => "Resize" = 2,
    APPLICATION_MESSAGE => "ApplicationMessage" = 3,
    WINDOW => "Window" = 4,
    WIDGET => "Widget" = 5,
});

wire_module!(host_node_payload, HOST_NODE_PAYLOAD, u8, {
    NONE => "None" = 0,
    NUMERIC => "Numeric" = 1,
    JSON => "Json" = 2,
});

/// Text aggregation and wrapping flags accepted by `SetTextBehavior`.
pub const TEXT_BEHAVIOR_AGGREGATE_DIRECT: u8 = 0x01;
/// Restrict the text node to a single line.
pub const TEXT_BEHAVIOR_SINGLE_LINE: u8 = 0x02;
/// Aggregate styled descendant text.
pub const TEXT_BEHAVIOR_AGGREGATE_STYLED: u8 = 0x04;
/// Clip lines beyond the configured maximum.
pub const TEXT_BEHAVIOR_CLIP_LINES: u8 = 0x08;

/// Include the node in focus navigation.
pub const INTERACTION_POLICY_FOCUSABLE: u8 = 0x01;
/// Prevent descendants from receiving pointer interaction.
pub const INTERACTION_POLICY_BLOCK_SUBTREE: u8 = 0x02;
/// Keep focus navigation within this subtree.
pub const INTERACTION_POLICY_CONTAIN_FOCUS: u8 = 0x04;

/// An inline SVG source string.
pub const GRAPHIC_SOURCE_SVG: u8 = 0x01;
/// A raster image stored in the native resource registry.
pub const GRAPHIC_SOURCE_RESOURCE_RASTER: u8 = 0x04;
/// Encoded vector path bytes.
pub const GRAPHIC_DATA_VECTOR_PATH: u8 = 0x01;
/// Maximum byte length accepted for one graphic-data operation.
pub const MAX_GRAPHIC_DATA_BYTES: usize = 16 * 1024 * 1024;

const TEXT_BEHAVIOR: &[WireConstant] = &[
    WireConstant {
        ts_name: "AggregateDirectText",
        value: TEXT_BEHAVIOR_AGGREGATE_DIRECT as u32,
    },
    WireConstant {
        ts_name: "SingleLine",
        value: TEXT_BEHAVIOR_SINGLE_LINE as u32,
    },
    WireConstant {
        ts_name: "AggregateStyledText",
        value: TEXT_BEHAVIOR_AGGREGATE_STYLED as u32,
    },
    WireConstant {
        ts_name: "ClipLines",
        value: TEXT_BEHAVIOR_CLIP_LINES as u32,
    },
];
const INTERACTION_POLICY: &[WireConstant] = &[
    WireConstant {
        ts_name: "Focusable",
        value: INTERACTION_POLICY_FOCUSABLE as u32,
    },
    WireConstant {
        ts_name: "BlockSubtree",
        value: INTERACTION_POLICY_BLOCK_SUBTREE as u32,
    },
    WireConstant {
        ts_name: "ContainFocus",
        value: INTERACTION_POLICY_CONTAIN_FOCUS as u32,
    },
];
const GRAPHIC_SOURCE: &[WireConstant] = &[
    WireConstant {
        ts_name: "Svg",
        value: GRAPHIC_SOURCE_SVG as u32,
    },
    WireConstant {
        ts_name: "ResourceRaster",
        value: GRAPHIC_SOURCE_RESOURCE_RASTER as u32,
    },
];
const GRAPHIC_DATA: &[WireConstant] = &[WireConstant {
    ts_name: "VectorPath",
    value: GRAPHIC_DATA_VECTOR_PATH as u32,
}];

fn push_object(output: &mut String, name: &str, entries: &[WireConstant]) {
    output.push_str("export const ");
    output.push_str(name);
    output.push_str(" = {\n");
    for entry in entries {
        output.push_str("  ");
        output.push_str(entry.ts_name);
        output.push_str(": ");
        output.push_str(&entry.value.to_string());
        output.push_str(",\n");
    }
    output.push_str("} as const;\n\n");
}

/// Render the TypeScript constants and literal-derived types consumed by
/// `@wabou/core`. The returned module is deterministic and contains no runtime
/// dependency on Rust.
#[must_use]
pub fn typescript_wire_constants() -> String {
    let mut output = String::from(
        "// AUTO-GENERATED by wabou-protocol. DO NOT EDIT.\n\
         // Rust definitions in crates/wabou-protocol/src/wire.rs are authoritative.\n\n",
    );
    push_object(&mut output, "OP", OP);
    output.push_str("export type OpCode = (typeof OP)[keyof typeof OP];\n\n");
    push_object(&mut output, "TEXT_BEHAVIOR", TEXT_BEHAVIOR);
    push_object(&mut output, "INTERACTION_POLICY", INTERACTION_POLICY);
    push_object(&mut output, "GRAPHIC_SOURCE", GRAPHIC_SOURCE);
    push_object(&mut output, "GRAPHIC_DATA", GRAPHIC_DATA);
    output.push_str(&format!(
        "export const MAX_GRAPHIC_DATA_BYTES = {MAX_GRAPHIC_DATA_BYTES};\n\n"
    ));
    push_object(&mut output, "EVENT_CODE", EVENT);
    output.push_str("export type EventType = keyof typeof EVENT_CODE;\n\n");
    push_object(&mut output, "EVENT_DATA_SLOT", EVENT_DATA);
    output.push_str(&format!(
        "export const EVENT_DATA_LEN = {};\n",
        event_data::LEN
    ));
    output.push_str("export type EventDataSlot = keyof typeof EVENT_DATA_SLOT;\n\n");
    push_object(&mut output, "HOST_FRAME", HOST_FRAME);
    push_object(&mut output, "HOST_RECORD_KIND", HOST_RECORD);
    push_object(&mut output, "HOST_NODE_PAYLOAD", HOST_NODE_PAYLOAD);
    output.truncate(output.trim_end().len());
    output.push('\n');
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_typescript_contains_every_rust_owned_table() {
        let generated = typescript_wire_constants();
        for name in [
            "OP",
            "TEXT_BEHAVIOR",
            "INTERACTION_POLICY",
            "GRAPHIC_SOURCE",
            "GRAPHIC_DATA",
            "EVENT_CODE",
            "EVENT_DATA_SLOT",
            "HOST_FRAME",
            "HOST_RECORD_KIND",
            "HOST_NODE_PAYLOAD",
        ] {
            assert!(generated.contains(&format!("export const {name} = {{")));
        }
        assert!(generated.contains("AcknowledgeTextValue: 39"));
        assert!(generated.contains("secretstatechange: 40"));
        assert!(generated.contains("export const EVENT_DATA_LEN = 21"));
    }
}
