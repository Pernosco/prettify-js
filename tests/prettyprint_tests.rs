use prettify_js::*;

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
        "AAAA;AACA,AAAQ,SAAC,CAAC,CAAC,CAAC,AAAC,EAAC,AAAC,EACf,AAAE,AAAM,OAAC,CAAC,AAAC,EACX"
    );
}
