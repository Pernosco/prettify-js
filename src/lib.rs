//! prettify-js is a tokenizer-based JS prettyprinter that generates source maps.
//!
//! Example:
//! ```
//! let (pretty, _) = prettify_js::prettyprint("function x(a){return a;}");
//! assert_eq!(pretty, "function x(a) {\n  return a;\n}\n");
//! ```

mod prettyprint;
mod source_map_generator;

pub use prettyprint::*;
pub use source_map_generator::*;

/// Uses a heuristic to decide if the source file needs prettyprinting:
/// whether its average line length is > 100.
///
/// We also prettyprint if the source string starts with "//PRETTYPRINT".
/// This is useful for testing.
pub fn should_prettyprint(source_str: &str) -> bool {
    if source_str.starts_with("//PRETTYPRINT") {
        return true;
    }
    let mut lines = 0;
    let mut line_lengths = 0;
    for line in source_str.lines() {
        lines += 1;
        line_lengths += line.len();
    }
    if lines == 0 {
        return false;
    }
    line_lengths / lines > 100
}

/// Convenience function to create a sourcemap for the prettyprinted version of the file
/// (if it needs prettyprinting), generate a URL for it and append that URL to the file text
/// so it gets used.
///
/// The `generate_file` closure takes a file name and file text and returns a URL which can
/// be used to load that file. This URL is injected into `source_str`.
///
/// Example:
/// ```
/// let mut generated = "//PRETTYPRINT\nfunction x(a){return a;}".to_string();
/// prettify_js::maybe_prettyprint("demo.js", &mut generated,
///   |name, text| {
///     assert_eq!(name, "demo.js.sourcemap");
///     let url = "https://example.com/demo.js.sourcemap".to_string();
///     println!("Serve {} with contents {}", &url, text);
///     url
///   });
/// assert_eq!(generated, "//PRETTYPRINT\nfunction x(a){return a;}\n//# sourceMappingURL=https://example.com/demo.js.sourcemap");
/// ```
pub fn maybe_prettyprint<G>(script_name: &str, source_str: &mut String, mut generate_file: G)
where
    G: FnMut(String, String) -> String,
{
    if !should_prettyprint(source_str) {
        return;
    }
    let (pretty_str, mappings) = prettyprint(&source_str);
    let source_map_name = format!("{}.sourcemap", script_name);
    let pretty_name = format!("{}.pretty", script_name);
    // The source map maps *from* prettyprinted source *to* the obfuscated/minified source
    let source_map = generate_source_map(pretty_name, pretty_str, mappings);
    let url = generate_file(source_map_name, source_map);
    source_str.push_str("\n//# sourceMappingURL=");
    source_str.push_str(&url);
}
