use super::*;

impl TerminalWidget {
    pub(super) fn key_bytes(&self, key: &wabou_shell::KeyEvent) -> Vec<u8> {
        let mode = self.terminal.lock().mode();
        if mode.intersects(Mode::KITTY_KEYBOARD_PROTOCOL) {
            let sequence = kitty_keyboard::build_key_sequence(key, mode);
            if !sequence.is_empty() || key.phase == KeyPhase::Up {
                return sequence;
            }
        } else if key.phase == KeyPhase::Up {
            return Vec::new();
        }
        if mode.contains(Mode::APP_KEYPAD)
            && let Some(sequence) = application_keypad_sequence(key)
        {
            return sequence;
        }
        if alt_graph_text(key).is_some() {
            return Vec::new();
        }
        if key.modifiers.control()
            && let Some(byte) = legacy_control_byte(&key.key)
        {
            let mut bytes = Vec::with_capacity(1 + usize::from(key.modifiers.alt()));
            if key.modifiers.alt() {
                bytes.push(0x1b);
            }
            bytes.push(byte);
            return bytes;
        }
        if key.modifiers.alt() && key.key.chars().count() == 1 {
            let mut bytes = vec![0x1b];
            bytes.extend_from_slice(key.key.as_bytes());
            return bytes;
        }

        legacy_special_key(key, mode)
    }
}

fn legacy_special_key(key: &wabou_shell::KeyEvent, mode: Mode) -> Vec<u8> {
    let modifier = modifier_code(key.modifiers);
    if let Some(sequence) = cursor_key_sequence(&key.key, mode, modifier) {
        return sequence;
    }
    if let Some(number) = key
        .key
        .strip_prefix('F')
        .and_then(|number| number.parse::<u8>().ok())
        && let Some(sequence) = legacy_function_key(number, modifier)
    {
        return sequence;
    }

    match key.key.as_str() {
        "Enter" => b"\r".to_vec(),
        "Backspace" if key.modifiers.alt() => b"\x1b\x7f".to_vec(),
        "Backspace" => vec![0x7f],
        "Tab" if key.modifiers.shift() => b"\x1b[Z".to_vec(),
        "Tab" => b"\t".to_vec(),
        "Escape" => vec![0x1b],
        "Insert" => tilde_key(2, modifier),
        "Delete" => tilde_key(3, modifier),
        "PageUp" => tilde_key(5, modifier),
        "PageDown" => tilde_key(6, modifier),
        _ => Vec::new(),
    }
}

fn modifier_code(modifiers: wabou_shell::Modifiers) -> u8 {
    1 + u8::from(modifiers.shift())
        + u8::from(modifiers.alt()) * 2
        + u8::from(modifiers.control()) * 4
}

fn cursor_key_sequence(key: &str, mode: Mode, modifier: u8) -> Option<Vec<u8>> {
    let final_byte = match key {
        "ArrowUp" => 'A',
        "ArrowDown" => 'B',
        "ArrowRight" => 'C',
        "ArrowLeft" => 'D',
        "Home" => 'H',
        "End" => 'F',
        _ => return None,
    };
    if modifier != 1 {
        return Some(csi_function_key(final_byte, modifier));
    }
    let prefix = if mode.contains(Mode::APP_CURSOR) {
        "\x1bO"
    } else {
        "\x1b["
    };
    Some(format!("{prefix}{final_byte}").into_bytes())
}

fn legacy_function_key(number: u8, modifier: u8) -> Option<Vec<u8>> {
    const TILDE_CODES: [u8; 8] = [15, 17, 18, 19, 20, 21, 23, 24];
    match number {
        1..=4 => {
            let final_byte = char::from(b'P' + number - 1);
            Some(if modifier == 1 {
                format!("\x1bO{final_byte}").into_bytes()
            } else {
                csi_function_key(final_byte, modifier)
            })
        }
        5..=12 => Some(function_key(TILDE_CODES[usize::from(number - 5)], modifier)),
        13..=16 => Some(csi_function_key(
            char::from(b'P' + number - 13),
            force_shift_modifier(modifier),
        )),
        17..=24 => Some(function_key(
            TILDE_CODES[usize::from(number - 17)],
            force_shift_modifier(modifier),
        )),
        25..=28 => Some(csi_function_key(
            char::from(b'P' + number - 25),
            force_control_modifier(modifier),
        )),
        29..=35 => Some(function_key(
            TILDE_CODES[usize::from(number - 29)],
            force_control_modifier(modifier),
        )),
        _ => None,
    }
}

fn tilde_key(number: u8, modifier: u8) -> Vec<u8> {
    if modifier == 1 {
        format!("\x1b[{number}~").into_bytes()
    } else {
        format!("\x1b[{number};{modifier}~").into_bytes()
    }
}

pub(super) fn legacy_control_byte(key: &str) -> Option<u8> {
    let byte = key.as_bytes().first().copied()?;
    if key.len() != 1 {
        return None;
    }
    match byte {
        b'@'..=b'_' => Some(byte & 0x1f),
        b'a'..=b'z' => Some(byte & 0x1f),
        b' ' | b'2' => Some(0x00),
        b'3'..=b'7' => Some(byte - b'3' + 0x1b),
        b'8' | b'?' => Some(0x7f),
        b'/' => Some(0x1f),
        _ => None,
    }
}

pub(super) fn application_keypad_sequence(key: &wabou_shell::KeyEvent) -> Option<Vec<u8>> {
    if key.location != wabou_shell::KeyLocation::Numpad {
        return None;
    }
    let final_byte = if key.code.contains("Numpad0") {
        'p'
    } else if key.code.contains("Numpad1") {
        'q'
    } else if key.code.contains("Numpad2") {
        'r'
    } else if key.code.contains("Numpad3") {
        's'
    } else if key.code.contains("Numpad4") {
        't'
    } else if key.code.contains("Numpad5") {
        'u'
    } else if key.code.contains("Numpad6") {
        'v'
    } else if key.code.contains("Numpad7") {
        'w'
    } else if key.code.contains("Numpad8") {
        'x'
    } else if key.code.contains("Numpad9") {
        'y'
    } else if key.code.contains("NumpadDecimal") {
        'n'
    } else if key.code.contains("NumpadComma") {
        'l'
    } else if key.code.contains("NumpadAdd") {
        'k'
    } else if key.code.contains("NumpadSubtract") {
        'm'
    } else if key.code.contains("NumpadMultiply") {
        'j'
    } else if key.code.contains("NumpadDivide") {
        'o'
    } else if key.code.contains("NumpadEnter") {
        'M'
    } else if key.code.contains("NumpadEqual") {
        'X'
    } else {
        return None;
    };
    Some(vec![0x1b, b'O', final_byte as u8])
}

pub(super) fn alt_graph_text(key: &wabou_shell::KeyEvent) -> Option<&str> {
    if !key.modifiers.control() || !key.modifiers.alt() || key.modifiers.meta() {
        return None;
    }
    let text = key.text_with_all_modifiers.as_deref()?;
    text.chars()
        .all(|character| !character.is_control())
        .then_some(text)
        .filter(|text| !text.is_empty())
}

pub(super) fn function_key(number: u8, modifier: u8) -> Vec<u8> {
    if modifier == 1 {
        format!("\x1b[{number}~").into_bytes()
    } else {
        format!("\x1b[{number};{modifier}~").into_bytes()
    }
}

pub(super) fn force_shift_modifier(modifier: u8) -> u8 {
    ((modifier - 1) | 1) + 1
}

pub(super) fn force_control_modifier(modifier: u8) -> u8 {
    ((modifier - 1) | 4) + 1
}

pub(super) fn csi_function_key(final_byte: char, modifier: u8) -> Vec<u8> {
    format!("\x1b[1;{modifier}{final_byte}").into_bytes()
}

pub(super) fn encode_paste(text: &str, bracketed: bool) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(text.len() + usize::from(bracketed) * 12);
    if bracketed {
        encoded.extend_from_slice(b"\x1b[200~");
        for character in text
            .chars()
            .filter(|character| !character.is_control() || matches!(character, '\t' | '\n' | '\r'))
        {
            let mut utf8 = [0; 4];
            encoded.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
        }
        encoded.extend_from_slice(b"\x1b[201~");
        return encoded;
    }

    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\r' if characters.peek() == Some(&'\n') => {
                characters.next();
                encoded.push(b'\r');
            }
            '\n' | '\r' => encoded.push(b'\r'),
            '\t' => encoded.push(b'\t'),
            character if character.is_control() => {}
            character => {
                let mut utf8 = [0; 4];
                encoded.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
            }
        }
    }
    encoded
}

pub(super) fn normal_mouse_sequence(
    code: u8,
    column: usize,
    row: usize,
    utf8: bool,
) -> Option<Vec<u8>> {
    let max_coordinate = if utf8 { 2015 } else { 223 };
    if column > max_coordinate || row > max_coordinate {
        return None;
    }

    let mut sequence = vec![0x1b, b'[', b'M', code + 32];
    let mut encode_coordinate = |coordinate: usize| {
        let value = coordinate + 32;
        if utf8 && coordinate >= 96 {
            sequence.push((0xc0 + value / 64) as u8);
            sequence.push((0x80 + (value & 63)) as u8);
        } else {
            sequence.push(value as u8);
        }
    };
    encode_coordinate(column);
    encode_coordinate(row);
    Some(sequence)
}
