// Kitty keyboard encoding adapted from Rio's frontend implementation, which
// in turn derives from Alacritty (Apache-2.0). The VT parser owns negotiated
// mode state; this module is the platform-independent Wabou KeyEvent adapter.

use rio_vt::crosswords::Mode;
use wabou_shell::{KeyEvent, KeyLocation, KeyPhase, Modifiers};

pub fn build_key_sequence(key: &KeyEvent, mode: Mode) -> Vec<u8> {
    let event_modifiers = normalized_modifiers(key);
    let kitty_seq = mode.intersects(
        Mode::REPORT_ALL_KEYS_AS_ESC | Mode::DISAMBIGUATE_ESC_CODES | Mode::REPORT_EVENT_TYPES,
    );
    if !should_build_sequence(key, mode, kitty_seq, event_modifiers) {
        return Vec::new();
    }

    let encode_all = mode.contains(Mode::REPORT_ALL_KEYS_AS_ESC);
    let report_event =
        mode.contains(Mode::REPORT_EVENT_TYPES) && (key.repeat || key.phase == KeyPhase::Up);
    let associated_text = key.text_with_all_modifiers.as_deref().filter(|text| {
        mode.contains(Mode::REPORT_ASSOCIATED_TEXT)
            && key.phase == KeyPhase::Down
            && !text.is_empty()
            && !is_control_character(text)
    });
    let mut modifiers = sequence_modifiers(event_modifiers);

    let base = numpad_base(key, kitty_seq)
        .or_else(|| named_kitty_base(&key.key, kitty_seq))
        .or_else(|| named_normal_base(&key.key, modifiers, report_event, associated_text.is_some()))
        .or_else(|| control_or_modifier_base(key, encode_all, kitty_seq, &mut modifiers))
        .or_else(|| textual_base(key, mode, kitty_seq, encode_all, associated_text));
    let Some((payload, terminator)) = base else {
        return Vec::new();
    };

    let mut sequence = format!("\x1b[{payload}");
    if report_event || modifiers != 0 || associated_text.is_some() {
        sequence.push_str(&format!(";{}", modifiers + 1));
    }
    if report_event {
        sequence.push(':');
        sequence.push(if key.phase == KeyPhase::Up {
            '3'
        } else if key.repeat {
            '2'
        } else {
            '1'
        });
    }
    if let Some(text) = associated_text {
        for (index, character) in text.chars().enumerate() {
            sequence.push(if index == 0 { ';' } else { ':' });
            sequence.push_str(&u32::from(character).to_string());
        }
    }
    sequence.push(terminator);
    sequence.into_bytes()
}

fn should_build_sequence(
    key: &KeyEvent,
    mode: Mode,
    kitty_seq: bool,
    modifiers: Modifiers,
) -> bool {
    if key.phase == KeyPhase::Up && !mode.contains(Mode::REPORT_EVENT_TYPES) {
        return false;
    }
    if mode.contains(Mode::REPORT_ALL_KEYS_AS_ESC) {
        return true;
    }
    if mode.contains(Mode::REPORT_EVENT_TYPES) && (key.repeat || key.phase == KeyPhase::Up) {
        return true;
    }
    let disambiguate = mode.contains(Mode::DISAMBIGUATE_ESC_CODES)
        && (key.key == "Escape"
            || key.location == KeyLocation::Numpad
            || (!modifiers.is_empty()
                && (modifiers != Modifiers::SHIFT
                    || matches!(key.key.as_str(), "Tab" | "Enter" | "Backspace"))));
    if disambiguate {
        return true;
    }
    if is_named_key(&key.key) {
        return !named_key_has_text(&key.key);
    }
    kitty_seq
        && key
            .text_with_all_modifiers
            .as_deref()
            .is_none_or(str::is_empty)
}

fn textual_base(
    key: &KeyEvent,
    mode: Mode,
    kitty_seq: bool,
    encode_all: bool,
    associated_text: Option<&str>,
) -> Option<(String, char)> {
    if !kitty_seq {
        return None;
    }
    let mut characters = key.key.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return (encode_all && associated_text.is_some()).then(|| ("0".into(), 'u'));
    }

    let shifted = key.modifiers.shift();
    let shifted_code = u32::from(character);
    let mut base_character = if shifted {
        character.to_lowercase().next().unwrap_or(character)
    } else {
        character
    };
    if shifted && u32::from(base_character) == shifted_code {
        base_character = key
            .key_without_modifiers
            .chars()
            .next()
            .unwrap_or(base_character);
    }
    let base_code = u32::from(base_character);
    let payload = if mode.contains(Mode::REPORT_ALTERNATE_KEYS) && base_code != shifted_code {
        format!("{base_code}:{shifted_code}")
    } else {
        base_code.to_string()
    };
    Some((payload, 'u'))
}

fn numpad_base(key: &KeyEvent, kitty_seq: bool) -> Option<(String, char)> {
    if !kitty_seq || key.location != KeyLocation::Numpad {
        return None;
    }
    let code = if key.code.contains("NumpadComma") {
        57416
    } else if key.code.contains("Numpad5") && matches!(key.key.as_str(), "Clear" | "Begin") {
        57427
    } else {
        match key.key.as_str() {
            "0" => 57399,
            "1" => 57400,
            "2" => 57401,
            "3" => 57402,
            "4" => 57403,
            "5" => 57404,
            "6" => 57405,
            "7" => 57406,
            "8" => 57407,
            "9" => 57408,
            "." => 57409,
            "/" => 57410,
            "*" => 57411,
            "-" => 57412,
            "+" => 57413,
            "Enter" => 57414,
            "=" => 57415,
            "ArrowLeft" => 57417,
            "ArrowRight" => 57418,
            "ArrowUp" => 57419,
            "ArrowDown" => 57420,
            "PageUp" => 57421,
            "PageDown" => 57422,
            "Home" => 57423,
            "End" => 57424,
            "Insert" => 57425,
            "Delete" => 57426,
            "Clear" | "Begin" => 57427,
            _ => return None,
        }
    };
    Some((code.to_string(), 'u'))
}

fn named_kitty_base(key: &str, kitty_seq: bool) -> Option<(String, char)> {
    if !kitty_seq {
        return None;
    }
    if key == "F3" {
        return Some(("13".into(), '~'));
    }
    if let Some(number) = function_number(key)
        && (13..=35).contains(&number)
    {
        return Some(((57376 + number - 13).to_string(), 'u'));
    }
    let code = match key {
        "ScrollLock" => 57359,
        "PrintScreen" => 57361,
        "Pause" => 57362,
        "ContextMenu" => 57363,
        "MediaPlay" => 57428,
        "MediaPause" => 57429,
        "MediaPlayPause" => 57430,
        "MediaReverse" => 57431,
        "MediaStop" => 57432,
        "MediaFastForward" => 57433,
        "MediaRewind" => 57434,
        "MediaTrackNext" => 57435,
        "MediaTrackPrevious" => 57436,
        "MediaRecord" => 57437,
        "AudioVolumeDown" => 57438,
        "AudioVolumeUp" => 57439,
        "AudioVolumeMute" => 57440,
        _ => return None,
    };
    Some((code.to_string(), 'u'))
}

fn named_normal_base(
    key: &str,
    modifiers: u8,
    report_event: bool,
    associated_text: bool,
) -> Option<(String, char)> {
    let one = if modifiers == 0 && !report_event && !associated_text {
        ""
    } else {
        "1"
    };
    let result = match key {
        "PageUp" => ("5".into(), '~'),
        "PageDown" => ("6".into(), '~'),
        "Insert" => ("2".into(), '~'),
        "Delete" => ("3".into(), '~'),
        "Home" => (one.into(), 'H'),
        "End" => (one.into(), 'F'),
        "ArrowLeft" => (one.into(), 'D'),
        "ArrowRight" => (one.into(), 'C'),
        "ArrowUp" => (one.into(), 'A'),
        "ArrowDown" => (one.into(), 'B'),
        "F1" => (one.into(), 'P'),
        "F2" => (one.into(), 'Q'),
        "F3" => (one.into(), 'R'),
        "F4" => (one.into(), 'S'),
        _ => {
            let number = function_number(key)?;
            let parameter = match number {
                5 => 15,
                6 => 17,
                7 => 18,
                8 => 19,
                9 => 20,
                10 => 21,
                11 => 23,
                12 => 24,
                13 => 25,
                14 => 26,
                15 => 28,
                16 => 29,
                17 => 31,
                18 => 32,
                19 => 33,
                20 => 34,
                _ => return None,
            };
            (parameter.to_string(), '~')
        }
    };
    Some(result)
}

fn control_or_modifier_base(
    key: &KeyEvent,
    encode_all: bool,
    kitty_seq: bool,
    modifiers: &mut u8,
) -> Option<(String, char)> {
    if !encode_all && !kitty_seq {
        return None;
    }
    let mut code = match key.key.as_str() {
        "Tab" => Some(9),
        "Enter" => Some(13),
        "Escape" => Some(27),
        "Space" | " " => Some(32),
        "Backspace" => Some(127),
        _ => None,
    };
    if encode_all {
        code = match (key.key.as_str(), key.location) {
            ("Shift", KeyLocation::Left) => Some(57441),
            ("Control", KeyLocation::Left) => Some(57442),
            ("Alt", KeyLocation::Left) => Some(57443),
            ("Super", KeyLocation::Left) => Some(57444),
            ("Hyper", KeyLocation::Left) => Some(57445),
            ("Meta", KeyLocation::Left) => Some(57446),
            ("Shift", _) => Some(57447),
            ("Control", _) => Some(57448),
            ("Alt", _) => Some(57449),
            ("Super", _) => Some(57450),
            ("Hyper", _) => Some(57451),
            ("Meta", _) => Some(57452),
            ("CapsLock", _) => Some(57358),
            ("NumLock", _) => Some(57360),
            ("AltGraph", _) => Some(57453),
            ("IsoLevel5Shift", _) => Some(57454),
            _ => code,
        };
    }
    let pressed = key.phase == KeyPhase::Down;
    match key.key.as_str() {
        "Shift" => set_modifier(modifiers, 1, pressed),
        "Alt" => set_modifier(modifiers, 2, pressed),
        "Control" => set_modifier(modifiers, 4, pressed),
        "Super" | "Meta" => set_modifier(modifiers, 8, pressed),
        _ => {}
    }
    code.map(|code| (code.to_string(), 'u'))
}

fn set_modifier(modifiers: &mut u8, flag: u8, enabled: bool) {
    if enabled {
        *modifiers |= flag;
    } else {
        *modifiers &= !flag;
    }
}

fn sequence_modifiers(modifiers: Modifiers) -> u8 {
    u8::from(modifiers.shift())
        | (u8::from(modifiers.alt()) << 1)
        | (u8::from(modifiers.control()) << 2)
        | (u8::from(modifiers.meta()) << 3)
}

fn normalized_modifiers(key: &KeyEvent) -> Modifiers {
    if key.modifiers.control()
        && key.modifiers.alt()
        && !key.modifiers.meta()
        && key
            .text_with_all_modifiers
            .as_deref()
            .is_some_and(|text| !text.is_empty() && !is_control_character(text))
    {
        key.modifiers - (Modifiers::CONTROL | Modifiers::ALT)
    } else {
        key.modifiers
    }
}

fn function_number(key: &str) -> Option<u32> {
    key.strip_prefix('F')?.parse().ok()
}

fn is_named_key(key: &str) -> bool {
    function_number(key).is_some()
        || matches!(
            key,
            "Alt"
                | "ArrowDown"
                | "ArrowLeft"
                | "ArrowRight"
                | "ArrowUp"
                | "AudioVolumeDown"
                | "AudioVolumeMute"
                | "AudioVolumeUp"
                | "Backspace"
                | "CapsLock"
                | "ContextMenu"
                | "Control"
                | "Delete"
                | "End"
                | "Enter"
                | "Escape"
                | "Home"
                | "Hyper"
                | "Insert"
                | "MediaFastForward"
                | "MediaPause"
                | "MediaPlay"
                | "MediaPlayPause"
                | "MediaRecord"
                | "MediaReverse"
                | "MediaRewind"
                | "MediaStop"
                | "MediaTrackNext"
                | "MediaTrackPrevious"
                | "Meta"
                | "NumLock"
                | "PageDown"
                | "PageUp"
                | "Pause"
                | "PrintScreen"
                | "ScrollLock"
                | "Shift"
                | "Space"
                | "Super"
                | "Tab"
                | "AltGraph"
                | "IsoLevel5Shift"
                | "Clear"
                | "Begin"
        )
}

fn named_key_has_text(key: &str) -> bool {
    matches!(key, "Enter" | "Tab" | "Space")
}

fn is_control_character(text: &str) -> bool {
    let Some(character) = text.chars().next() else {
        return false;
    };
    text.chars().count() == 1
        && (character <= '\u{1f}' || ('\u{7f}'..='\u{9f}').contains(&character))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str, modifiers: Modifiers) -> KeyEvent {
        KeyEvent {
            phase: KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.to_lowercase(),
            code: String::new(),
            text: Some(name.into()),
            text_with_all_modifiers: Some(name.into()),
            location: KeyLocation::Standard,
            modifiers,
            repeat: false,
        }
    }

    #[test]
    fn encodes_disambiguated_and_alternate_text_keys() {
        let mut event = key("A", Modifiers::SHIFT | Modifiers::CONTROL);
        event.key_without_modifiers = "a".into();
        event.text_with_all_modifiers = None;
        let mode = Mode::DISAMBIGUATE_ESC_CODES | Mode::REPORT_ALTERNATE_KEYS;
        assert_eq!(build_key_sequence(&event, mode), b"\x1b[97:65;6u");
    }

    #[test]
    fn reports_press_repeat_release_and_associated_text() {
        let mut event = key("x", Modifiers::empty());
        let mode =
            Mode::REPORT_ALL_KEYS_AS_ESC | Mode::REPORT_EVENT_TYPES | Mode::REPORT_ASSOCIATED_TEXT;
        assert_eq!(build_key_sequence(&event, mode), b"\x1b[120;1;120u");
        event.repeat = true;
        assert_eq!(build_key_sequence(&event, mode), b"\x1b[120;1:2;120u");
        event.repeat = false;
        event.phase = KeyPhase::Up;
        assert_eq!(build_key_sequence(&event, mode), b"\x1b[120;1:3u");
    }

    #[test]
    fn ignores_release_without_report_event_types() {
        let mut event = key("Backspace", Modifiers::empty());
        let mode = Mode::DISAMBIGUATE_ESC_CODES;
        assert_eq!(build_key_sequence(&event, mode), b"\x1b[127u");

        event.phase = KeyPhase::Up;
        assert!(build_key_sequence(&event, mode).is_empty());
    }

    #[test]
    fn encodes_numpad_and_extended_function_keys() {
        let mut numpad = key("1", Modifiers::empty());
        numpad.location = KeyLocation::Numpad;
        assert_eq!(
            build_key_sequence(&numpad, Mode::DISAMBIGUATE_ESC_CODES),
            b"\x1b[57400u"
        );
        assert_eq!(
            build_key_sequence(
                &key("F35", Modifiers::empty()),
                Mode::REPORT_ALL_KEYS_AS_ESC
            ),
            b"\x1b[57398u"
        );
    }

    #[test]
    fn alt_graph_text_is_not_misreported_as_control_alt() {
        let mut event = key("@", Modifiers::CONTROL | Modifiers::ALT);
        event.key_without_modifiers = "q".into();
        assert!(build_key_sequence(&event, Mode::DISAMBIGUATE_ESC_CODES).is_empty());
        assert_eq!(
            build_key_sequence(&event, Mode::REPORT_ALL_KEYS_AS_ESC),
            b"\x1b[64u"
        );
    }
}
