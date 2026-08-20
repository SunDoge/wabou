use std::{collections::BTreeMap, fs, path::Path};

use serde::Serialize;

type Dictionary = BTreeMap<Vec<u8>, BencodeValue>;

#[derive(Debug)]
enum BencodeValue {
    Bytes(Vec<u8>),
    Dict(Dictionary),
    Integer(i64),
    List(Vec<BencodeValue>),
}

const MAX_TORRENT_SIZE: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentPreview {
    pub name: String,
    pub total_length: u64,
    pub files: Vec<TorrentFilePreview>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentFilePreview {
    /// User-facing BitTorrent file index. These indices are one-based.
    pub index: u64,
    pub path: String,
    pub length: u64,
}

pub fn read_torrent(path: &Path) -> Result<(Vec<u8>, TorrentPreview), String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("cannot inspect torrent file: {error}"))?;
    if !metadata.is_file() {
        return Err("torrent path is not a regular file".to_owned());
    }
    if metadata.len() > MAX_TORRENT_SIZE {
        return Err("torrent file exceeds the 64 MB safety limit".to_owned());
    }
    let bytes = fs::read(path).map_err(|error| format!("cannot read torrent file: {error}"))?;
    let preview = parse_torrent(&bytes)?;
    Ok((bytes, preview))
}

pub fn parse_torrent(bytes: &[u8]) -> Result<TorrentPreview, String> {
    let root = BencodeParser::new(bytes).parse()?;
    let root = dictionary(&root, "torrent root")?;
    let info = dictionary(required(root, b"info", "torrent info")?, "torrent info")?;
    let name = text(
        info.get(b"name.utf-8".as_slice())
            .or_else(|| info.get(b"name".as_slice()))
            .ok_or_else(|| "torrent info is missing its name".to_owned())?,
        "torrent name",
    )?;

    let mut files = Vec::new();
    if let Some(BencodeValue::List(entries)) = info.get(b"files".as_slice()) {
        for entry in entries {
            let entry = dictionary(entry, "torrent file")?;
            let length = unsigned(
                required(entry, b"length", "torrent file length")?,
                "torrent file length",
            )?;
            let components = entry
                .get(b"path.utf-8".as_slice())
                .or_else(|| entry.get(b"path".as_slice()))
                .ok_or_else(|| "torrent file is missing its path".to_owned())?;
            let components = list(components, "torrent file path")?
                .iter()
                .map(|component| text(component, "torrent file path component"))
                .collect::<Result<Vec<_>, _>>()?;
            push_file(&mut files, components.join("/"), length)?;
        }
    } else if let Some(tree) = info.get(b"file tree".as_slice()) {
        walk_v2_tree(
            dictionary(tree, "torrent file tree")?,
            &mut Vec::new(),
            &mut files,
        )?;
    } else {
        let length = unsigned(
            required(info, b"length", "torrent length")?,
            "torrent length",
        )?;
        push_file(&mut files, name.clone(), length)?;
    }

    if files.is_empty() {
        return Err("torrent metadata contains no files".to_owned());
    }
    let total_length = files.iter().try_fold(0_u64, |total, file| {
        total
            .checked_add(file.length)
            .ok_or_else(|| "torrent total length overflows u64".to_owned())
    })?;
    Ok(TorrentPreview {
        name,
        total_length,
        files,
    })
}

fn walk_v2_tree(
    tree: &Dictionary,
    parents: &mut Vec<String>,
    files: &mut Vec<TorrentFilePreview>,
) -> Result<(), String> {
    let mut entries = tree.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(component, _)| *component);
    for (component, value) in entries {
        if component.is_empty() {
            let attributes = dictionary(value, "torrent v2 file attributes")?;
            let length = unsigned(
                required(attributes, b"length", "torrent v2 file length")?,
                "torrent v2 file length",
            )?;
            push_file(files, parents.join("/"), length)?;
            continue;
        }
        parents.push(String::from_utf8_lossy(component).into_owned());
        walk_v2_tree(
            dictionary(value, "torrent file tree entry")?,
            parents,
            files,
        )?;
        parents.pop();
    }
    Ok(())
}

fn push_file(files: &mut Vec<TorrentFilePreview>, path: String, length: u64) -> Result<(), String> {
    if path.is_empty() {
        return Err("torrent file path is empty".to_owned());
    }
    let index = u64::try_from(files.len())
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| "torrent contains too many files".to_owned())?;
    files.push(TorrentFilePreview {
        index,
        path,
        length,
    });
    Ok(())
}

fn required<'a>(dict: &'a Dictionary, key: &[u8], label: &str) -> Result<&'a BencodeValue, String> {
    dict.get(key).ok_or_else(|| format!("{label} is missing"))
}

fn dictionary<'a>(value: &'a BencodeValue, label: &str) -> Result<&'a Dictionary, String> {
    match value {
        BencodeValue::Dict(value) => Ok(value),
        _ => Err(format!("{label} must be a dictionary")),
    }
}

fn list<'a>(value: &'a BencodeValue, label: &str) -> Result<&'a [BencodeValue], String> {
    match value {
        BencodeValue::List(value) => Ok(value),
        _ => Err(format!("{label} must be a list")),
    }
}

fn text(value: &BencodeValue, label: &str) -> Result<String, String> {
    match value {
        BencodeValue::Bytes(value) if !value.is_empty() => {
            Ok(String::from_utf8_lossy(value).into_owned())
        }
        BencodeValue::Bytes(_) => Err(format!("{label} is empty")),
        _ => Err(format!("{label} must be a byte string")),
    }
}

fn unsigned(value: &BencodeValue, label: &str) -> Result<u64, String> {
    match value {
        BencodeValue::Integer(value) => {
            u64::try_from(*value).map_err(|_| format!("{label} must not be negative"))
        }
        _ => Err(format!("{label} must be an integer")),
    }
}

struct BencodeParser<'a> {
    bytes: &'a [u8],
    cursor: usize,
    values: usize,
}

impl<'a> BencodeParser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: 0,
            values: 0,
        }
    }

    fn parse(mut self) -> Result<BencodeValue, String> {
        if self.bytes.is_empty() {
            return Err("torrent metadata is empty".to_owned());
        }
        let value = self.value(0)?;
        if self.cursor != self.bytes.len() {
            return Err("torrent metadata contains trailing values".to_owned());
        }
        Ok(value)
    }

    fn value(&mut self, depth: usize) -> Result<BencodeValue, String> {
        if depth > 128 {
            return Err("torrent metadata nesting exceeds 128 levels".to_owned());
        }
        self.values = self
            .values
            .checked_add(1)
            .ok_or_else(|| "torrent metadata is too complex".to_owned())?;
        if self.values > 1_000_000 {
            return Err("torrent metadata contains too many values".to_owned());
        }
        match self.peek()? {
            b'd' => self.dictionary(depth + 1),
            b'l' => self.list(depth + 1),
            b'i' => self.integer(),
            b'0'..=b'9' => self.bytes().map(BencodeValue::Bytes),
            _ => Err(format!("invalid bencode token at byte {}", self.cursor)),
        }
    }

    fn dictionary(&mut self, depth: usize) -> Result<BencodeValue, String> {
        self.cursor += 1;
        let mut values = BTreeMap::new();
        while self.peek()? != b'e' {
            let key = self.bytes()?;
            let value = self.value(depth)?;
            if values.insert(key, value).is_some() {
                return Err("torrent metadata contains a duplicate dictionary key".to_owned());
            }
        }
        self.cursor += 1;
        Ok(BencodeValue::Dict(values))
    }

    fn list(&mut self, depth: usize) -> Result<BencodeValue, String> {
        self.cursor += 1;
        let mut values = Vec::new();
        while self.peek()? != b'e' {
            values.push(self.value(depth)?);
        }
        self.cursor += 1;
        Ok(BencodeValue::List(values))
    }

    fn integer(&mut self) -> Result<BencodeValue, String> {
        self.cursor += 1;
        let start = self.cursor;
        while self.peek()? != b'e' {
            self.cursor += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.cursor])
            .map_err(|_| "bencode integer is not ASCII".to_owned())?;
        if text.is_empty() || text == "-0" || (text.starts_with('0') && text.len() > 1) {
            return Err("bencode integer is not canonical".to_owned());
        }
        let value = text
            .parse::<i64>()
            .map_err(|_| "bencode integer is out of range".to_owned())?;
        self.cursor += 1;
        Ok(BencodeValue::Integer(value))
    }

    fn bytes(&mut self) -> Result<Vec<u8>, String> {
        let start = self.cursor;
        while self.peek()? != b':' {
            if !self.bytes[self.cursor].is_ascii_digit() {
                return Err(format!(
                    "invalid byte string length at byte {}",
                    self.cursor
                ));
            }
            self.cursor += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.cursor])
            .map_err(|_| "byte string length is not ASCII".to_owned())?;
        if text.is_empty() || (text.starts_with('0') && text.len() > 1) {
            return Err("byte string length is not canonical".to_owned());
        }
        let length = text
            .parse::<usize>()
            .map_err(|_| "byte string length is out of range".to_owned())?;
        self.cursor += 1;
        let end = self
            .cursor
            .checked_add(length)
            .ok_or_else(|| "byte string length overflows".to_owned())?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or_else(|| "byte string extends past end of torrent metadata".to_owned())?;
        self.cursor = end;
        Ok(value.to_vec())
    }

    fn peek(&self) -> Result<u8, String> {
        self.bytes
            .get(self.cursor)
            .copied()
            .ok_or_else(|| "unexpected end of torrent metadata".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v1_multi_file_torrent_with_one_based_indices() {
        let bytes = b"d4:infod5:filesld6:lengthi3e4:pathl5:a.txteed6:lengthi7e4:pathl3:dir5:b.bineee4:name4:demoee";
        let preview = parse_torrent(bytes).expect("valid torrent");
        assert_eq!(preview.name, "demo");
        assert_eq!(preview.total_length, 10);
        assert_eq!(
            preview.files[0],
            TorrentFilePreview {
                index: 1,
                path: "a.txt".to_owned(),
                length: 3
            }
        );
        assert_eq!(
            preview.files[1],
            TorrentFilePreview {
                index: 2,
                path: "dir/b.bin".to_owned(),
                length: 7
            }
        );
    }

    #[test]
    fn parses_many_file_fixture_used_by_behavior_tests() {
        let entries = (1..=30)
            .map(|index| {
                let path = format!("folder-{index:02}/file-{index:02}.bin");
                format!("d6:lengthi{index}e4:pathl{}:{path}ee", path.len())
            })
            .collect::<String>();
        let bytes = format!("d4:infod5:filesl{entries}e4:name12:fixture-packee");
        let preview = parse_torrent(bytes.as_bytes()).expect("valid fixture");
        assert_eq!(preview.files.len(), 30);
        assert_eq!(preview.total_length, 465);
    }

    #[test]
    fn parses_v2_file_tree() {
        let bytes = b"d4:infod9:file treed5:a.txtd0:d6:lengthi4eeee4:name4:demoee";
        let preview = parse_torrent(bytes).expect("valid v2 torrent");
        assert_eq!(preview.total_length, 4);
        assert_eq!(preview.files[0].path, "a.txt");
    }

    #[test]
    fn rejects_negative_lengths() {
        let bytes = b"d4:infod6:lengthi-1e4:name4:demoee";
        assert!(
            parse_torrent(bytes)
                .unwrap_err()
                .contains("must not be negative")
        );
    }
}
