# rustxml2 — tinyxml2 → Rust Port Design

**Date:** 2026-06-14
**Source:** `D:\Workspace\Rust\Original\tinyxml2` (tinyxml2 v11.0.0 — `tinyxml2.h` ~2,384 LOC, `tinyxml2.cpp` ~3,029 LOC, `xmltest.cpp` ~2,865 LOC)
**Target:** `D:\Workspace\Rust\rustxml2`

## 1. Goal & Approach

A from-scratch idiomatic Rust DOM library reproducing tinyxml2's **behavior and full feature set** with a Rust-native API:

- Document-owned **generational arena** of nodes addressed by `NodeId` handles.
- `Result`-based error handling.
- UTF-8 `&str` / `String` throughout (input must be valid UTF-8).
- Iterators for child/sibling traversal.
- **Std-only, zero external dependencies** (honoring tinyxml2's dependency-free ethos).
- Tests ported into idiomatic `#[test]` cases run by `cargo test`.

### Decisions locked during brainstorming
| Decision | Choice |
|---|---|
| Port style | Idiomatic Rust (re-architect, same behavior) |
| Test porting | Idiomatic `#[test]` cases |
| Feature scope | **Full parity** with tinyxml2 |
| DOM model | Arena + `NodeId` handles (document-mediated API) |
| Encoding / strings | UTF-8 `&str` / `String` (valid UTF-8 required) |

## 2. Crate & Module Layout

```
rustxml2/
  Cargo.toml
  src/
    lib.rs        // crate root, re-exports, crate docs
    error.rs      // XmlError enum (mirrors XMLError) + Result alias + error location
    arena.rs      // generational arena (NodeId{index,generation}), alloc/free/get
    node.rs       // NodeData, NodeKind, navigation (parent/sibling/child)
    element.rs    // element ops: attributes, typed queries, child-element iteration
    attribute.rs  // Attribute storage + typed accessors
    parser.rs     // the parse state machine (entry: Document::parse)
    printer.rs    // XmlPrinter: serialize with whitespace/compact modes
    visitor.rs    // XmlVisitor trait + accept() traversal
    handle.rs     // XmlHandle: null-safe chained navigation
    strutil.rs    // XMLUtil port: entity encode/decode, num<->str, whitespace, BOM
    document.rs   // XmlDocument: arena owner, parse/print entry, options, error state
  tests/
    resources/    // copied XML fixtures from tinyxml2/resources
    *.rs          // ported xmltest.cpp cases grouped by area
```

## 3. Core Data Model

- `XmlDocument` owns `nodes: Arena<NodeData>` plus options (`whitespace_mode`, `process_entities`, `write_bom`) and error state (`error_id`, `error_str`, `error_line`).
- `NodeId { index: u32, generation: u32 }` — **generational** so that `delete_node` / `delete_children` cannot produce dangling-handle reuse bugs (tinyxml2 supports node deletion via its pool).
- `NodeData { kind, parent, first_child, last_child, prev_sibling, next_sibling, value }`.
- `NodeKind`: `Document | Element(ElementData) | Text(TextData) | Comment | Declaration | Unknown`.
  - `ElementData { attributes: Vec<Attribute>, closing_type }`
  - `TextData { cdata: bool }`
- `Attribute { name: String, value: String }`.
- `Whitespace`: `Preserve | Collapse | Pedantic`.

## 4. Public API (tinyxml2 → Rust mapping)

Navigation/mutation is **document-mediated** (`doc.method(id, ...)`).

- **Navigation:** `first_child_element(id, Some("book"))`, `next_sibling_element(id, None)`, `parent(id)`, `first_child(id)`, `next_sibling(id)`; iterator helpers `child_elements(id)`, `children(id)`.
- **Element data:** `name(id)`, `set_name(id, ..)`, `text(id) -> Option<&str>`, `set_text(id, ..)`.
- **Attributes:** `attribute(id, "x") -> Option<&str>`; typed `query_int_attribute(id, "x") -> Result<i32, XmlError>` (and i64/u32/u64/f32/f64/bool); `set_attribute(id, "x", v)` generic over an `XmlValue` trait (i32/i64/u32/u64/f32/f64/bool/&str), mirroring overloaded `SetAttribute`.
- **Typed value accessors** on attribute/text values: `as_i32/as_i64/as_u32/as_u64/as_f32/as_f64/as_bool` (port of `XMLUtil::ToVal`).
- **Construction:** `new_element("a")`, `new_text`, `new_comment`, `new_declaration`, `new_unknown`; `insert_end_child`, `insert_first_child`, `insert_after_child`, `delete_node`, `delete_children`.

## 5. Parsing

Port of tinyxml2's parse logic as a state machine over a UTF-8 `&str`:

- Recognizes declarations (`<?xml ... ?>`), comments (`<!-- -->`), CDATA (`<![CDATA[ ]]>`), unknowns (`<! ... >`, DOCTYPE), elements (open / close / self-closing), attributes, and text.
- Entity decode (`&lt; &gt; &amp; &apos; &quot;` + numeric `&#nn;` / `&#xnn;`) when `process_entities` is enabled.
- `MAX_ELEMENT_DEPTH = 500` guard → `XmlError::ElementDepthExceeded`.
- Optional leading BOM detection/handling.
- Whitespace handling per `Whitespace` mode (`Preserve` | `Collapse` | `Pedantic`).
- Errors carry a line number, matching tinyxml2's reporting.

## 6. Serialization (`XmlPrinter`)

- Implements `XmlVisitor`; writes to any `std::io::Write`, with a `to_string()` convenience.
- Compact vs. pretty (indented) output.
- Attribute-entity escaping with the `EscapeApos` option.
- BOM / declaration emission matching tinyxml2.
- Round-trips the `resources/` fixtures.

## 7. Visitor & Handle

- `XmlVisitor` trait: `visit_enter_document` / `visit_exit_document`, `visit_enter_element` / `visit_exit_element` (return `bool` to prune traversal as in C++), `visit_text` / `visit_comment` / `visit_declaration` / `visit_unknown`. Traversal via `doc.accept(id, &mut visitor)`.
- `XmlHandle` / `XmlConstHandle`: thin `Option<NodeId>`-carrying chainable wrapper for null-safe navigation, mirroring C++ handle ergonomics.

## 8. Error Handling

`XmlError` enum mirrors every `XMLError` variant (the C++ `XML_SUCCESS`/`NoError` collapses into `Ok`), including:
`WrongAttributeType`, `NoAttribute`, `FileNotFound`, `FileCouldNotBeOpened`, `FileReadError`, `ErrorParsingElement`, `ErrorParsingAttribute`, `ErrorParsingText`, `ErrorParsingCData`, `ErrorParsingComment`, `ErrorParsingDeclaration`, `ErrorParsingUnknown`, `ErrorEmptyDocument`, `ErrorMismatchedElement`, `ErrorParsing`, `CanNotConvertText`, `NoTextNode`, `ElementDepthExceeded`.

- `parse` / file ops return `Result<(), XmlError>`.
- Document retains last-error + line for `error_str()` / `error_line()` parity.
- `error_name()` ports `ErrorIDToName`.

## 9. Tests

- Port `xmltest.cpp` cases into `#[test]` functions grouped by topic: parsing, attributes, typed values, printing, entities, whitespace, error cases, deep-nesting, round-trip.
- `XMLTest(expected, found)` → `assert_eq!`.
- Copy `resources/` fixtures into `tests/resources/`; file-based tests use `std::fs`.

### Adaptations
- C++ `_CrtMemState` memory-leak checks are dropped — Rust ownership makes them unnecessary.
- The few raw-byte / invalid-UTF-8 cases are adapted to valid-UTF-8 equivalents (per the UTF-8 decision), noted in test comments.

## 10. Out of Scope

- No C++ ABI / FFI layer.
- No CMake / meson build (Cargo only).
- No Doxygen docs port (Rustdoc instead).
- No in-place mutable-buffer parsing (we own a parsed tree instead).
