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
        "AAAA;AACA,SAAS,CAAC,CAAC,CAAC,EAAE,EACd,AAAE,OAAO,CAAC,EACV"
    );
}
