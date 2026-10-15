use super::decode_vector_path;

fn path_bytes(command: u8, coordinates: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0x3150_4257u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0x38bd_f8ffu32.to_le_bytes());
    bytes.extend_from_slice(&0xa78b_faffu32.to_le_bytes());
    bytes.extend_from_slice(&2.0f32.to_le_bytes());
    bytes.extend_from_slice(&[0, 1, 1, 0]);
    bytes.extend_from_slice(&4.0f32.to_le_bytes());
    bytes.extend_from_slice(&[command, 0, 0, 0]);
    for coordinate in coordinates {
        bytes.extend_from_slice(&coordinate.to_le_bytes());
    }
    let len = bytes.len() as u32;
    bytes[12..16].copy_from_slice(&len.to_le_bytes());
    bytes
}

#[test]
fn validates_path_payload_before_retain() {
    let mut line = path_bytes(1, &[3.0, 4.0]);
    line.extend_from_slice(&[2, 0, 0, 0]);
    line.extend_from_slice(&5.0f32.to_le_bytes());
    line.extend_from_slice(&6.0f32.to_le_bytes());
    line[8..12].copy_from_slice(&2u32.to_le_bytes());
    let line_len = line.len() as u32;
    line[12..16].copy_from_slice(&line_len.to_le_bytes());
    assert!(decode_vector_path(&line).is_some());
    assert!(decode_vector_path(&path_bytes(1, &[3.0, 4.0])).is_none());
    assert!(decode_vector_path(&path_bytes(99, &[])).is_none());
    assert!(decode_vector_path(&path_bytes(1, &[f32::NAN, 4.0])).is_none());
    let mut truncated = path_bytes(1, &[3.0, 4.0]);
    truncated.pop();
    assert!(decode_vector_path(&truncated).is_none());
    assert!(decode_vector_path(&path_bytes(2, &[3.0, 4.0])).is_none());
    let mut empty = path_bytes(1, &[3.0, 4.0]);
    empty.truncate(36);
    empty[8..12].copy_from_slice(&0u32.to_le_bytes());
    empty[12..16].copy_from_slice(&36u32.to_le_bytes());
    assert!(decode_vector_path(&empty).is_none());
}
