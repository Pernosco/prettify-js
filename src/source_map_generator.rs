use serde::Serialize;

/// Zero-based line number
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceMapLine(pub u32);

/// Zero-based column number, in UTF16 code units
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceMapColumn(pub u32);

/// Source coordinate, line+column
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceCoord {
    pub line: SourceMapLine,
    pub column: SourceMapColumn,
}

/// Points that correspond in the original/generated source code.
pub struct SourceMapping {
    /// Coordinate in the "original source code"
    pub from: SourceCoord,
    /// Coordinate in the "generated source code"
    pub to: SourceCoord,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceMapJson {
    version: i32,
    sources: Vec<String>,
    sources_content: Vec<String>,
    names: Vec<String>,
    mappings: String,
}

fn encode_digit(value: u8) -> char {
    let b = match value {
        0..=25 => b'A' + value,
        26..=51 => b'a' + (value - 26),
        52..=61 => b'0' + (value - 52),
        62 => b'+',
        63 => b'/',
        _ => panic!("Invalid digit"),
    };
    b as char
}

fn write_vlq(value: i64, output: &mut String) {
    let mut val = if value >= 0 {
        (value as u64) << 1
    } else if value == i64::MIN {
        output.push('g');
        1 << (63 - 4)
    } else {
        ((-value as u64) << 1) + 1
    };
    loop {
        let mut digit = val as u8 & ((1 << 5) - 1);
        val >>= 5;
        if val > 0 {
            digit |= 1 << 5;
        }
        output.push(encode_digit(digit));
        if val == 0 {
            break;
        }
    }
}

/// Generate a source-map as a string, given the original source file name,
/// file data, and a list of mappings from the original source to the generated
/// source.
///
/// `from_name` is the name of the "original source code" file.
/// `from_content` is the content of that file. We always insert the content inline
/// into the source map.
/// Currently we only support mappings where a single source file is mapped to a single
/// generated file.
pub fn generate_source_map(
    from_name: String,
    from_content: String,
    mappings: Vec<SourceMapping>,
) -> String {
    let mut map = SourceMapJson {
        version: 3,
        sources: vec![from_name],
        sources_content: vec![from_content],
        names: Vec::new(),
        mappings: String::new(),
    };
    let mut last_to_line = SourceMapLine(0);
    let mut last_to_column = SourceMapColumn(0);
    let mut last_from_line = SourceMapLine(0);
    let mut last_from_column = SourceMapColumn(0);
    for m in mappings {
        if last_to_line < m.to.line {
            while last_to_line < m.to.line {
                map.mappings.push(';');
                last_to_column = SourceMapColumn(0);
                last_to_line.0 += 1;
            }
        } else {
            if !map.mappings.is_empty() {
                map.mappings.push(',');
            }
        }
        write_vlq(
            m.to.column.0 as i64 - last_to_column.0 as i64,
            &mut map.mappings,
        );
        last_to_column = m.to.column;
        // Sources index is always 0 since we only support one "from" file currently.
        write_vlq(0, &mut map.mappings);
        write_vlq(
            m.from.line.0 as i64 - last_from_line.0 as i64,
            &mut map.mappings,
        );
        last_from_line = m.from.line;
        write_vlq(
            m.from.column.0 as i64 - last_from_column.0 as i64,
            &mut map.mappings,
        );
        last_from_column = m.from.column;
    }
    serde_json::to_string(&map).unwrap()
}
