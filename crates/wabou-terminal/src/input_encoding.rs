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

        let modifier = 1
            + u8::from(key.modifiers.shift())
            + u8::from(key.modifiers.alt()) * 2
            + u8::from(key.modifiers.control()) * 4;
        let modified = modifier != 1;
        let csi_key = |final_byte: char| format!("\x1b[1;{modifier}{final_byte}").into_bytes();

        match key.key.as_str() {
            "Enter" => b"\r".to_vec(),
            "Backspace" if key.modifiers.alt() => b"\x1b\x7f".to_vec(),
            "Backspace" => vec![0x7f],
            "Tab" if key.modifiers.shift() => b"\x1b[Z".to_vec(),
            "Tab" => b"\t".to_vec(),
            "Escape" => vec![0x1b],
            "ArrowUp" if modified => csi_key('A'),
            "ArrowDown" if modified => csi_key('B'),
            "ArrowRight" if modified => csi_key('C'),
            "ArrowLeft" if modified => csi_key('D'),
            "ArrowUp" if mode.contains(Mode::APP_CURSOR) => b"\x1bOA".to_vec(),
            "ArrowDown" if mode.contains(Mode::APP_CURSOR) => b"\x1bOB".to_vec(),
            "ArrowRight" if mode.contains(Mode::APP_CURSOR) => b"\x1bOC".to_vec(),
            "ArrowLeft" if mode.contains(Mode::APP_CURSOR) => b"\x1bOD".to_vec(),
            "ArrowUp" => b"\x1b[A".to_vec(),
            "ArrowDown" => b"\x1b[B".to_vec(),
            "ArrowRight" => b"\x1b[C".to_vec(),
            "ArrowLeft" => b"\x1b[D".to_vec(),
            "Home" if modified => csi_key('H'),
            "End" if modified => csi_key('F'),
            "Home" if mode.contains(Mode::APP_CURSOR) => b"\x1bOH".to_vec(),
            "End" if mode.contains(Mode::APP_CURSOR) => b"\x1bOF".to_vec(),
            "Home" => b"\x1b[H".to_vec(),
            "End" => b"\x1b[F".to_vec(),
            "Insert" => format!(
                "\x1b[2{}~",
                if modified {
                    format!(";{modifier}")
                } else {
                    String::new()
                }
            )
            .into_bytes(),
            "Delete" => format!(
                "\x1b[3{}~",
                if modified {
                    format!(";{modifier}")
                } else {
                    String::new()
                }
            )
            .into_bytes(),
            "PageUp" => format!(
                "\x1b[5{}~",
                if modified {
                    format!(";{modifier}")
                } else {
                    String::new()
                }
            )
            .into_bytes(),
            "PageDown" => format!(
                "\x1b[6{}~",
                if modified {
                    format!(";{modifier}")
                } else {
                    String::new()
                }
            )
            .into_bytes(),
            "F1" if modified => csi_key('P'),
            "F2" if modified => csi_key('Q'),
            "F3" if modified => csi_key('R'),
            "F4" if modified => csi_key('S'),
            "F1" => b"\x1bOP".to_vec(),
            "F2" => b"\x1bOQ".to_vec(),
            "F3" => b"\x1bOR".to_vec(),
            "F4" => b"\x1bOS".to_vec(),
            "F5" => function_key(15, modifier),
            "F6" => function_key(17, modifier),
            "F7" => function_key(18, modifier),
            "F8" => function_key(19, modifier),
            "F9" => function_key(20, modifier),
            "F10" => function_key(21, modifier),
            "F11" => function_key(23, modifier),
            "F12" => function_key(24, modifier),
            "F13" => csi_function_key('P', force_shift_modifier(modifier)),
            "F14" => csi_function_key('Q', force_shift_modifier(modifier)),
            "F15" => csi_function_key('R', force_shift_modifier(modifier)),
            "F16" => csi_function_key('S', force_shift_modifier(modifier)),
            "F17" => function_key(15, force_shift_modifier(modifier)),
            "F18" => function_key(17, force_shift_modifier(modifier)),
            "F19" => function_key(18, force_shift_modifier(modifier)),
            "F20" => function_key(19, force_shift_modifier(modifier)),
            "F21" => function_key(20, force_shift_modifier(modifier)),
            "F22" => function_key(21, force_shift_modifier(modifier)),
            "F23" => function_key(23, force_shift_modifier(modifier)),
            "F24" => function_key(24, force_shift_modifier(modifier)),
            "F25" => csi_function_key('P', force_control_modifier(modifier)),
            "F26" => csi_function_key('Q', force_control_modifier(modifier)),
            "F27" => csi_function_key('R', force_control_modifier(modifier)),
            "F28" => csi_function_key('S', force_control_modifier(modifier)),
            "F29" => function_key(15, force_control_modifier(modifier)),
            "F30" => function_key(17, force_control_modifier(modifier)),
            "F31" => function_key(18, force_control_modifier(modifier)),
            "F32" => function_key(19, force_control_modifier(modifier)),
            "F33" => function_key(20, force_control_modifier(modifier)),
            "F34" => function_key(21, force_control_modifier(modifier)),
            "F35" => function_key(23, force_control_modifier(modifier)),
            _ => Vec::new(),
        }
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
