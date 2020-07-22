use prettify_js::*;

fn m(fl: u32, fc: u32, tl: u32, tc: u32) -> SourceMapping {
    SourceMapping {
        from: SourceCoord {
            line: SourceMapLine(fl),
            column: SourceMapColumn(fc),
        },
        to: SourceCoord {
            line: SourceMapLine(tl),
            column: SourceMapColumn(tc),
        },
    }
}

#[test]
fn let_token() {
    let (pretty, mappings) = prettyprint("let a = 1;");
    assert_eq!(pretty, "let a = 1;\n");
    assert_eq!(
        mappings,
        vec![
            m(0, 0, 0, 0),
            m(0, 4, 0, 4),
            m(0, 6, 0, 6),
            m(0, 8, 0, 8),
            m(0, 9, 0, 9)
        ]
    );
}

#[test]
fn indent() {
    let (pretty, mappings) = prettyprint("function non_simple_params(...rest) {\n  {\n    let kitty = 99;\n    console.log(rest[0] + kitty);\n  }\n}");
    assert_eq!(pretty, "function non_simple_params(...rest) {\n  {\n    let kitty = 99;\n    console.log(rest[0] + kitty);\n  }\n}\n");
    assert_eq!(
        mappings
            .iter()
            .filter(|v| v.from.line.0 == 3)
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            m(3, 4, 3, 4),
            m(3, 11, 3, 11),
            m(3, 12, 3, 12),
            m(3, 15, 3, 15),
            m(3, 16, 3, 16),
            m(3, 20, 3, 20),
            m(3, 21, 3, 21),
            m(3, 22, 3, 22),
            m(3, 24, 3, 24),
            m(3, 26, 3, 26),
            m(3, 31, 3, 31),
            m(3, 32, 3, 32)
        ]
    );
}

#[test]
fn print() {
    let (pretty, mappings) = prettyprint("/* ABC */\nfunction f(x) { return x; }");
    assert_eq!(pretty, "/* ABC */\nfunction f(x) {\n  return x;\n}\n");
    let map = generate_source_map("orig.name".to_string(), "<original>".to_string(), mappings);
    let v: serde_json::Value = serde_json::from_str(&map).unwrap();
    let mappings = v
        .as_object()
        .unwrap()
        .get("mappings")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(
        mappings,
        "AAAA;AACA,SAAS,CAAC,CAAC,CAAC,EAAE,EACZ,OAAO,CAAC,EACV"
    );
}
