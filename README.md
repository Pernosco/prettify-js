# prettify-js

A fast, robust but imperfect token-based JS code prettifier, written in Rust, that outputs JS source maps.

The code was mostly ported from Mozilla's [pretty-fast](https://github.com/mozilla/pretty-fast). Instead of using [Acorn](https://github.com/acornjs/acorn) to tokenize, we use [RESS](https://crates.io/crates/ress). Instead of using the [source-map](https://github.com/mozilla/source-map) package to generate source maps, we use our own very minimal handwritten source-map emitter. The original pretty-fast code tries to avoid emitting more than one source-map record per pretty line; instead we emit one source-map record per token, because we sometimes care about code offsets within a pretty line.
