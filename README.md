# rustxml2

An idiomatic Rust port of the [tinyxml2](https://github.com/leethomason/tinyxml2)
XML DOM library. It parses XML into an in-memory tree, lets you navigate and
mutate it, and serializes it back out — with the same feature set as tinyxml2
but a safe, Rust-native API.

- **Safe by construction** — nodes live in a document-owned *generational arena*
  and are referenced by lightweight `NodeId` handles. Stale handles are rejected
  rather than dangling; no `unsafe`.
- **`Result`-based errors** — an `XmlError` enum mirroring tinyxml2's `XMLError`.
- **UTF-8 throughout** — parse from `&str`, get back `&str`.
- **Zero dependencies** — std only.

## Quick start

Add the crate to your `Cargo.toml` (path or git dependency), then:

```rust
use rustxml2::XmlDocument;

let mut doc = XmlDocument::new();
doc.parse(r#"<note to="you" id="42">hello</note>"#).unwrap();

let note = doc.root_element().unwrap();
assert_eq!(doc.name(note), Some("note"));
assert_eq!(doc.attribute(note, "to"), Some("you"));
assert_eq!(doc.text(note), Some("hello"));

// Typed attribute access
assert_eq!(doc.query_int_attribute(note, "id"), Ok(42));
```

## Building a document and serializing

```rust
use rustxml2::XmlDocument;

let mut doc = XmlDocument::new();
let root = doc.new_element("library");
doc.insert_end_child(doc.root(), root);

let book = doc.new_element("book");
doc.insert_end_child(root, book);
doc.set_attribute(book, "id", 1i32);
doc.set_text(book, "The Rust Programming Language");

// Compact (default) or pretty-printed output:
assert_eq!(
    doc.print_to_string(true),
    r#"<library><book id="1">The Rust Programming Language</book></library>"#
);
println!("{}", doc.print_to_string(false)); // indented
```

## Navigating

```rust
use rustxml2::XmlDocument;

let mut doc = XmlDocument::new();
doc.parse("<list><i>a</i><i>b</i><i>c</i></list>").unwrap();
let list = doc.root_element().unwrap();

// First/next element, optionally filtered by name:
let first = doc.first_child_element(list, Some("i")).unwrap();
let second = doc.next_sibling_element(first, None).unwrap();

// Or iterate:
for item in doc.child_elements(list, Some("i")) {
    println!("{:?}", doc.text(item));
}

// Null-safe chained navigation via XmlHandle:
use rustxml2::handle::XmlHandle;
let third = XmlHandle::new(&doc, list)
    .first_child_element(Some("i"))
    .next_sibling_element(None)
    .next_sibling_element(None)
    .id();
```

## Features

- Parsing of elements, attributes, text, comments, CDATA, processing
  instructions/declarations, and unknown/`<!DOCTYPE ...>` nodes (with bracketed
  internal subsets).
- Predefined and numeric entity decoding; entity escaping on output.
- Whitespace modes: `Preserve` (default), `Collapse`, `Pedantic`.
- Optional leading BOM handling.
- Element-depth limit (500) to guard against stack-overflow attacks.
- Typed attribute accessors (`query_int/int64/unsigned/float/double/bool_attribute`),
  including `0x` hex integers.
- `XmlVisitor` trait for tree traversal; `XmlPrinter` for serialization.
- File I/O: `load_file` / `save_file`.

## File I/O

```rust
use rustxml2::XmlDocument;
use std::path::Path;

let mut doc = XmlDocument::new();
doc.load_file(Path::new("input.xml")).unwrap();
doc.save_file(Path::new("output.xml"), false).unwrap(); // false = pretty
```

## Building and testing

```sh
cargo build
cargo test      # unit + integration tests (ported from tinyxml2's xmltest.cpp)
```

On Windows, the repo includes a `.cargo/config.toml` pinning the MSVC linker for
this environment; adjust or remove it for other machines.

## Differences from tinyxml2

The behavior aims to match tinyxml2; the API is re-shaped for Rust. Notably,
navigation and mutation are *document-mediated* (`doc.method(id, ...)`) rather
than methods on node pointers, and input must be valid UTF-8. See
`docs/superpowers/specs/` for the full design.

## License

Licensed under the [Zlib license](LICENSE), the same terms as the original
tinyxml2.
