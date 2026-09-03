use sourcemap::DecodedMap;

pub(crate) struct StackSourceMap {
    map: DecodedMap,
}

impl StackSourceMap {
    pub(crate) fn parse(bytes: &[u8]) -> Option<Self> {
        sourcemap::decode_slice(bytes).ok().map(|map| Self { map })
    }

    pub(crate) fn map_stack(&self, stack: &str) -> String {
        stack
            .lines()
            .map(|line| self.map_line(line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn map_line(&self, line: &str) -> String {
        let Some((start, end, generated_line, generated_column)) = generated_location(line) else {
            return line.to_owned();
        };
        let Some(token) = self.map.lookup_token(
            generated_line.saturating_sub(1),
            generated_column.saturating_sub(1),
        ) else {
            return line.to_owned();
        };
        let Some(source) = token.get_source() else {
            return line.to_owned();
        };
        let original = format!(
            "{}:{}:{}",
            source,
            token.get_src_line() + 1,
            token.get_src_col() + 1
        );
        format!(
            "{}{} [generated {}:{}]{}",
            &line[..start],
            original,
            generated_line,
            generated_column,
            &line[end..]
        )
    }
}

fn generated_location(line: &str) -> Option<(usize, usize, u32, u32)> {
    for marker in ["bundle.js:", "eval_script:"] {
        let Some(start) = line.find(marker) else {
            continue;
        };
        let numbers = &line[start + marker.len()..];
        let (line_number, rest) = numbers.split_once(':')?;
        let column = rest
            .split(|character: char| !character.is_ascii_digit())
            .next()?;
        if let (Ok(line_number), Ok(column)) = (line_number.parse::<u32>(), column.parse::<u32>()) {
            let end =
                start + marker.len() + line_number.to_string().len() + 1 + column.to_string().len();
            return Some((start, end, line_number, column));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::StackSourceMap;

    #[test]
    fn maps_quickjs_bundle_frames_to_original_sources() {
        let map = StackSourceMap::parse(
            br#"{"version":3,"sources":["ui/app.tsx"],"sourcesContent":["throw new Error()"],"names":[],"mappings":"AAAA"}"#,
        )
        .unwrap();
        assert_eq!(
            map.map_stack("    at App (bundle.js:1:1)"),
            "    at App (ui/app.tsx:1:1 [generated 1:1])"
        );
    }
}
