# rustxml2 (tinyxml2 → Rust) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the tinyxml2 C++ XML DOM library to an idiomatic, std-only Rust crate (`rustxml2`) with full feature parity, including a ported test suite.

**Architecture:** A document-owned generational arena holds all nodes, addressed by lightweight `NodeId` handles. All navigation/mutation is document-mediated (`doc.method(id, ...)`). Parsing is a state machine over a UTF-8 `&str` producing an owned tree; serialization uses an `XmlPrinter` implementing an `XmlVisitor` trait. Errors are `Result`-based via an `XmlError` enum mirroring tinyxml2's `XMLError`.

**Tech Stack:** Rust (edition 2021), `cargo test`, **zero external dependencies** (std only).

---

## File Structure

| File | Responsibility |
|---|---|
| `Cargo.toml` | Crate manifest, edition 2021, no deps |
| `src/lib.rs` | Crate root: module declarations, public re-exports, crate docs |
| `src/error.rs` | `XmlError` enum, `Result` alias, `error_name()` |
| `src/arena.rs` | Generational `Arena<T>` + `NodeId { index, generation }` |
| `src/node.rs` | `NodeData`, `NodeKind`, `TextData`, `ElementData`, `Whitespace`; navigation |
| `src/attribute.rs` | `Attribute`, `XmlValue` trait, typed value parsing (`as_i32` …) |
| `src/strutil.rs` | Entity encode/decode, number↔string, whitespace, BOM, name-char predicates |
| `src/document.rs` | `XmlDocument`: arena owner, options, error state, node construction, navigation API |
| `src/parser.rs` | Parse state machine (`XmlDocument::parse`) |
| `src/printer.rs` | `XmlPrinter`: serialization with compact/pretty + escaping |
| `src/visitor.rs` | `XmlVisitor` trait + `accept()` traversal |
| `src/handle.rs` | `XmlHandle` null-safe chained navigation |
| `tests/resources/` | XML fixtures copied from `tinyxml2/resources/` |
| `tests/*.rs` | Ported `xmltest.cpp` cases grouped by topic |

Source reference: `D:\Workspace\Rust\Original\tinyxml2` (`tinyxml2.h`, `tinyxml2.cpp`, `xmltest.cpp`).

---

## Task 1: Crate scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "rustxml2"
version = "0.1.0"
edition = "2021"
description = "An idiomatic Rust port of the tinyxml2 XML DOM library"
license = "Zlib"

[dependencies]
```

- [ ] **Step 2: Create minimal `src/lib.rs`**

```rust
//! rustxml2 — an idiomatic Rust port of the tinyxml2 XML DOM library.

pub mod arena;
pub mod attribute;
pub mod document;
pub mod error;
pub mod handle;
pub mod node;
pub mod parser;
pub mod printer;
pub mod strutil;
pub mod visitor;

pub use document::XmlDocument;
pub use error::{Result, XmlError};
pub use node::{NodeKind, Whitespace};
```

- [ ] **Step 3: Create empty module files so the crate compiles**

Create each of `src/arena.rs`, `src/attribute.rs`, `src/document.rs`, `src/error.rs`, `src/handle.rs`, `src/node.rs`, `src/parser.rs`, `src/printer.rs`, `src/strutil.rs`, `src/visitor.rs` containing a single line:

```rust
// implemented in a later task
```

- [ ] **Step 4: Verify it does not compile yet (re-exports reference empty modules)**

Run: `cargo build`
Expected: FAIL — unresolved imports (e.g. `document::XmlDocument`). This confirms the scaffold and that later tasks supply the names.

- [ ] **Step 5: Temporarily comment the re-exports to get a green baseline**

In `src/lib.rs`, comment out the three `pub use` lines (re-enable in Task 8).

Run: `cargo build`
Expected: PASS (empty modules compile).

- [ ] **Step 6: Commit**

```bash
git init
git add Cargo.toml src/
git commit -m "chore: scaffold rustxml2 crate"
```

---

## Task 2: Error type

**Files:**
- Modify: `src/error.rs`
- Test: inline `#[cfg(test)]` in `src/error.rs`

- [ ] **Step 1: Write the failing test**

In `src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_names_match_tinyxml2() {
        assert_eq!(XmlError::NoAttribute.name(), "XML_NO_ATTRIBUTE");
        assert_eq!(XmlError::ElementDepthExceeded.name(), "XML_ELEMENT_DEPTH_EXCEEDED");
        assert_eq!(XmlError::ErrorMismatchedElement.name(), "XML_ERROR_MISMATCHED_ELEMENT");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib error`
Expected: FAIL — `XmlError` not defined.

- [ ] **Step 3: Implement the error type**

Replace the placeholder in `src/error.rs` (keep the test module):

```rust
//! Error type mirroring tinyxml2's `XMLError`.

use std::fmt;

/// Result alias used across the crate.
pub type Result<T> = std::result::Result<T, XmlError>;

/// Mirrors tinyxml2's `XMLError`. `XML_SUCCESS` is represented by `Ok(_)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlError {
    NoAttribute,
    WrongAttributeType,
    FileNotFound,
    FileCouldNotBeOpened,
    FileReadError,
    ErrorParsingElement,
    ErrorParsingAttribute,
    ErrorParsingText,
    ErrorParsingCData,
    ErrorParsingComment,
    ErrorParsingDeclaration,
    ErrorParsingUnknown,
    ErrorEmptyDocument,
    ErrorMismatchedElement,
    ErrorParsing,
    CanNotConvertText,
    NoTextNode,
    ElementDepthExceeded,
}

impl XmlError {
    /// Equivalent of tinyxml2 `XMLDocument::ErrorIDToName`.
    pub fn name(self) -> &'static str {
        match self {
            XmlError::NoAttribute => "XML_NO_ATTRIBUTE",
            XmlError::WrongAttributeType => "XML_WRONG_ATTRIBUTE_TYPE",
            XmlError::FileNotFound => "XML_ERROR_FILE_NOT_FOUND",
            XmlError::FileCouldNotBeOpened => "XML_ERROR_FILE_COULD_NOT_BE_OPENED",
            XmlError::FileReadError => "XML_ERROR_FILE_READ_ERROR",
            XmlError::ErrorParsingElement => "XML_ERROR_PARSING_ELEMENT",
            XmlError::ErrorParsingAttribute => "XML_ERROR_PARSING_ATTRIBUTE",
            XmlError::ErrorParsingText => "XML_ERROR_PARSING_TEXT",
            XmlError::ErrorParsingCData => "XML_ERROR_PARSING_CDATA",
            XmlError::ErrorParsingComment => "XML_ERROR_PARSING_COMMENT",
            XmlError::ErrorParsingDeclaration => "XML_ERROR_PARSING_DECLARATION",
            XmlError::ErrorParsingUnknown => "XML_ERROR_PARSING_UNKNOWN",
            XmlError::ErrorEmptyDocument => "XML_ERROR_EMPTY_DOCUMENT",
            XmlError::ErrorMismatchedElement => "XML_ERROR_MISMATCHED_ELEMENT",
            XmlError::ErrorParsing => "XML_ERROR_PARSING",
            XmlError::CanNotConvertText => "XML_CAN_NOT_CONVERT_TEXT",
            XmlError::NoTextNode => "XML_NO_TEXT_NODE",
            XmlError::ElementDepthExceeded => "XML_ELEMENT_DEPTH_EXCEEDED",
        }
    }
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::error::Error for XmlError {}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib error`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/error.rs
git commit -m "feat: add XmlError type mirroring tinyxml2 XMLError"
```

---

## Task 3: Generational arena

**Files:**
- Modify: `src/arena.rs`
- Test: inline `#[cfg(test)]` in `src/arena.rs`

- [ ] **Step 1: Write the failing test**

In `src/arena.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get_remove_with_generations() {
        let mut a: Arena<i32> = Arena::new();
        let id = a.insert(42);
        assert_eq!(a.get(id), Some(&42));

        a.remove(id);
        assert_eq!(a.get(id), None); // stale handle rejected

        let id2 = a.insert(7); // reuses the slot, new generation
        assert_eq!(a.get(id2), Some(&7));
        assert_eq!(a.get(id), None); // old handle still rejected
        assert_ne!(id, id2);
    }

    #[test]
    fn get_mut_mutates() {
        let mut a: Arena<i32> = Arena::new();
        let id = a.insert(1);
        *a.get_mut(id).unwrap() = 99;
        assert_eq!(a.get(id), Some(&99));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib arena`
Expected: FAIL — `Arena` not defined.

- [ ] **Step 3: Implement the arena**

Replace the placeholder in `src/arena.rs` (keep the test module):

```rust
//! A simple generational arena. Stale `NodeId`s (after removal) are rejected.

/// Handle into an [`Arena`]. Cheap to copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    index: u32,
    generation: u32,
}

struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

/// Generational arena storage.
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena { slots: Vec::new(), free: Vec::new() }
    }

    pub fn insert(&mut self, value: T) -> NodeId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.value = Some(value);
            NodeId { index, generation: slot.generation }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot { generation: 0, value: Some(value) });
            NodeId { index, generation: 0 }
        }
    }

    fn slot(&self, id: NodeId) -> Option<&Slot<T>> {
        self.slots
            .get(id.index as usize)
            .filter(|s| s.generation == id.generation && s.value.is_some())
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.slot(id).and_then(|s| s.value.as_ref())
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation == id.generation {
            slot.value.as_mut()
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: NodeId) -> Option<T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation || slot.value.is_none() {
            return None;
        }
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(id.index);
        slot.value.take()
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.slot(id).is_some()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib arena`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src/arena.rs
git commit -m "feat: add generational arena with NodeId handles"
```

---

## Task 4: String utilities (entities, numbers, whitespace, BOM)

**Files:**
- Modify: `src/strutil.rs`
- Test: inline `#[cfg(test)]` in `src/strutil.rs`

Reference: `XMLUtil` in `tinyxml2.cpp` (`ToStr`, `ToVal`, entity handling), and the `entities[]` table / `PrintString` for encode.

- [ ] **Step 1: Write the failing tests**

In `src/strutil.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_named_and_numeric_entities() {
        assert_eq!(decode_entities("a &lt; b &amp; c &gt; &apos;&quot;"), "a < b & c > '\"");
        assert_eq!(decode_entities("&#65;&#x42;"), "AB");
        assert_eq!(decode_entities("no entities"), "no entities");
        // Unrecognized sequences pass through unchanged.
        assert_eq!(decode_entities("&unknown;"), "&unknown;");
    }

    #[test]
    fn encode_escapes_required_chars() {
        assert_eq!(encode_text("a < b & c > \"x\" 'y'"), "a &lt; b &amp; c &gt; &quot;x&quot; 'y'");
    }

    #[test]
    fn parse_bool_accepts_tinyxml2_forms() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("True"), Some(true));
        assert_eq!(parse_bool("maybe"), None);
    }

    #[test]
    fn strip_bom_removes_utf8_bom() {
        let with = "\u{feff}<a/>";
        assert_eq!(strip_bom(with), ("<a/>", true));
        assert_eq!(strip_bom("<a/>"), ("<a/>", false));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib strutil`
Expected: FAIL — functions not defined.

- [ ] **Step 3: Implement the utilities**

Replace the placeholder in `src/strutil.rs` (keep the test module):

```rust
//! Port of tinyxml2 `XMLUtil`: entity encode/decode, bool parsing, BOM, predicates.

/// UTF-8 BOM byte sequence.
pub const BOM: &str = "\u{feff}";

/// Strip a leading UTF-8 BOM. Returns the remainder and whether a BOM was present.
pub fn strip_bom(input: &str) -> (&str, bool) {
    match input.strip_prefix(BOM) {
        Some(rest) => (rest, true),
        None => (input, false),
    }
}

/// Decode the five predefined XML entities plus numeric character references.
/// Unrecognized `&...;` sequences are passed through verbatim (tinyxml2 behavior).
pub fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some((decoded, consumed)) = decode_one_entity(&input[i..]) {
                out.push(decoded);
                i += consumed;
                continue;
            }
        }
        // Push the next full UTF-8 char.
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Try to decode a single entity starting at `&`. Returns (char, bytes_consumed).
fn decode_one_entity(s: &str) -> Option<(char, usize)> {
    let end = s.find(';')?;
    let body = &s[1..end]; // between '&' and ';'
    let consumed = end + 1;
    let ch = match body {
        "lt" => '<',
        "gt" => '>',
        "amp" => '&',
        "apos" => '\'',
        "quot" => '"',
        _ if body.starts_with("#x") || body.starts_with("#X") => {
            let code = u32::from_str_radix(&body[2..], 16).ok()?;
            char::from_u32(code)?
        }
        _ if body.starts_with('#') => {
            let code = body[1..].parse::<u32>().ok()?;
            char::from_u32(code)?
        }
        _ => return None,
    };
    Some((ch, consumed))
}

/// Escape characters that must be escaped in element text / attribute values.
/// Note: apostrophe is NOT escaped (matches tinyxml2 default text behavior).
pub fn encode_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Parse a bool the way tinyxml2 does: accepts true/false/1/0, case-insensitive.
pub fn parse_bool(s: &str) -> Option<bool> {
    match s.trim() {
        _ if s.eq_ignore_ascii_case("true") => Some(true),
        _ if s.eq_ignore_ascii_case("false") => Some(false),
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

/// tinyxml2 `IsWhiteSpace`: space, tab, CR, LF.
pub fn is_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n')
}

/// tinyxml2 `IsNameStartChar`.
pub fn is_name_start_char(ch: char) -> bool {
    ch as u32 >= 128 || ch.is_ascii_alphabetic() || ch == ':' || ch == '_'
}

/// tinyxml2 `IsNameChar`.
pub fn is_name_char(ch: char) -> bool {
    is_name_start_char(ch) || ch.is_ascii_digit() || ch == '.' || ch == '-'
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib strutil`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add src/strutil.rs
git commit -m "feat: add string utilities (entities, bool parse, BOM, name predicates)"
```

---

## Task 5: Attribute type and typed value parsing

**Files:**
- Modify: `src/attribute.rs`
- Test: inline `#[cfg(test)]` in `src/attribute.rs`

Reference: `XMLAttribute` typed `QueryIntValue` etc. and `XMLUtil::ToVal` in `tinyxml2.cpp`.

- [ ] **Step 1: Write the failing tests**

In `src/attribute.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_accessors() {
        let a = Attribute { name: "x".into(), value: "42".into() };
        assert_eq!(a.as_i32(), Ok(42));
        assert_eq!(a.as_i64(), Ok(42));
        assert_eq!(a.as_f64(), Ok(42.0));

        let b = Attribute { name: "b".into(), value: "true".into() };
        assert_eq!(b.as_bool(), Ok(true));

        let bad = Attribute { name: "n".into(), value: "abc".into() };
        assert_eq!(bad.as_i32(), Err(XmlError::WrongAttributeType));
    }

    #[test]
    fn xml_value_to_string() {
        assert_eq!(42i32.to_xml_string(), "42");
        assert_eq!(true.to_xml_string(), "true");
        assert_eq!("hi".to_xml_string(), "hi");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib attribute`
Expected: FAIL — `Attribute` not defined.

- [ ] **Step 3: Implement attribute + value conversions**

Replace the placeholder in `src/attribute.rs` (keep the test module):

```rust
//! XML attribute storage, typed accessors, and the `XmlValue` set-conversion trait.

use crate::error::{Result, XmlError};
use crate::strutil::parse_bool;

/// A single `name="value"` attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

macro_rules! typed_accessor {
    ($method:ident, $ty:ty) => {
        pub fn $method(&self) -> Result<$ty> {
            self.value.trim().parse::<$ty>().map_err(|_| XmlError::WrongAttributeType)
        }
    };
}

impl Attribute {
    pub fn value(&self) -> &str {
        &self.value
    }

    typed_accessor!(as_i32, i32);
    typed_accessor!(as_i64, i64);
    typed_accessor!(as_u32, u32);
    typed_accessor!(as_u64, u64);
    typed_accessor!(as_f32, f32);
    typed_accessor!(as_f64, f64);

    pub fn as_bool(&self) -> Result<bool> {
        parse_bool(&self.value).ok_or(XmlError::WrongAttributeType)
    }
}

/// Types that can be stored as an attribute or text value (port of overloaded
/// `SetAttribute` / `SetText` / `PushText`).
pub trait XmlValue {
    fn to_xml_string(&self) -> String;
}

macro_rules! xml_value_display {
    ($ty:ty) => {
        impl XmlValue for $ty {
            fn to_xml_string(&self) -> String {
                self.to_string()
            }
        }
    };
}

xml_value_display!(i32);
xml_value_display!(i64);
xml_value_display!(u32);
xml_value_display!(u64);
xml_value_display!(f32);
xml_value_display!(f64);

impl XmlValue for bool {
    fn to_xml_string(&self) -> String {
        if *self { "true".into() } else { "false".into() }
    }
}

impl XmlValue for &str {
    fn to_xml_string(&self) -> String {
        (*self).to_string()
    }
}

impl XmlValue for String {
    fn to_xml_string(&self) -> String {
        self.clone()
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib attribute`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src/attribute.rs
git commit -m "feat: add Attribute type, typed accessors, and XmlValue trait"
```

---

## Task 6: Node data model

**Files:**
- Modify: `src/node.rs`
- Test: inline `#[cfg(test)]` in `src/node.rs`

Reference: `XMLNode` and subclasses; `Whitespace` enum (`tinyxml2.h:1703`); `ElementClosingType` (`tinyxml2.h:1669`).

- [ ] **Step 1: Write the failing test**

In `src/node.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_kind_predicates() {
        let el = NodeData::new(NodeKind::Element(ElementData::default()), "div".into());
        assert!(el.is_element());
        assert_eq!(el.value, "div");

        let txt = NodeData::new(NodeKind::Text(TextData { cdata: false }), "hi".into());
        assert!(txt.is_text());
        assert!(!txt.is_element());
    }

    #[test]
    fn whitespace_default_is_preserve() {
        assert_eq!(Whitespace::default(), Whitespace::Preserve);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib node`
Expected: FAIL — types not defined.

- [ ] **Step 3: Implement the node model**

Replace the placeholder in `src/node.rs` (keep the test module):

```rust
//! Node storage and kind discriminants.

use crate::arena::NodeId;
use crate::attribute::Attribute;

/// Whitespace handling mode (tinyxml2 `Whitespace`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whitespace {
    Preserve,
    Collapse,
    Pedantic,
}

impl Default for Whitespace {
    fn default() -> Self {
        Whitespace::Preserve
    }
}

/// How an element was closed (tinyxml2 `ElementClosingType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosingType {
    Open,    // <foo>
    Closed,  // <foo/>
}

/// Element-specific payload.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ElementData {
    pub attributes: Vec<Attribute>,
}

/// Text-specific payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextData {
    pub cdata: bool,
}

/// Node variant. `value` on the owning `NodeData` holds:
/// element name, text content, comment body, declaration body, or unknown body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Document,
    Element(ElementData),
    Text(TextData),
    Comment,
    Declaration,
    Unknown,
}

/// A node in the arena. Links are arena handles.
#[derive(Debug, Clone)]
pub struct NodeData {
    pub kind: NodeKind,
    pub value: String,
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
}

impl NodeData {
    pub fn new(kind: NodeKind, value: String) -> Self {
        NodeData {
            kind,
            value,
            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        }
    }

    pub fn is_element(&self) -> bool {
        matches!(self.kind, NodeKind::Element(_))
    }
    pub fn is_text(&self) -> bool {
        matches!(self.kind, NodeKind::Text(_))
    }
    pub fn is_document(&self) -> bool {
        matches!(self.kind, NodeKind::Document)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib node`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src/node.rs
git commit -m "feat: add node data model (NodeData, NodeKind, Whitespace)"
```

---

## Task 7: XmlDocument — construction & navigation

**Files:**
- Modify: `src/document.rs`
- Test: inline `#[cfg(test)]` in `src/document.rs`

Reference: `XMLDocument` and `XMLNode` navigation (`FirstChildElement`, `NextSiblingElement`, `InsertEndChild`, etc.).

- [ ] **Step 1: Write the failing tests**

In `src/document.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_tree_and_navigate() {
        let mut doc = XmlDocument::new();
        let root = doc.new_element("root");
        doc.insert_end_child(doc.root(), root);

        let a = doc.new_element("a");
        let b = doc.new_element("b");
        doc.insert_end_child(root, a);
        doc.insert_end_child(root, b);

        assert_eq!(doc.name(root), Some("root"));
        assert_eq!(doc.first_child_element(root, None), Some(a));
        assert_eq!(doc.next_sibling_element(a, None), Some(b));
        assert_eq!(doc.first_child_element(root, Some("b")), Some(b));
        assert_eq!(doc.parent(a), Some(root));
    }

    #[test]
    fn attributes_roundtrip() {
        let mut doc = XmlDocument::new();
        let e = doc.new_element("e");
        doc.set_attribute(e, "id", 5i32);
        doc.set_attribute(e, "ok", true);
        assert_eq!(doc.attribute(e, "id"), Some("5"));
        assert_eq!(doc.query_int_attribute(e, "id"), Ok(5));
        assert_eq!(doc.query_bool_attribute(e, "ok"), Ok(true));
        assert_eq!(doc.query_int_attribute(e, "missing"), Err(XmlError::NoAttribute));
    }

    #[test]
    fn set_and_get_text() {
        let mut doc = XmlDocument::new();
        let e = doc.new_element("e");
        doc.set_text(e, "hello");
        assert_eq!(doc.text(e), Some("hello"));
    }

    #[test]
    fn delete_node_detaches_and_invalidates() {
        let mut doc = XmlDocument::new();
        let root = doc.new_element("root");
        doc.insert_end_child(doc.root(), root);
        let a = doc.new_element("a");
        doc.insert_end_child(root, a);
        doc.delete_node(a);
        assert_eq!(doc.first_child_element(root, None), None);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib document`
Expected: FAIL — `XmlDocument` not defined.

- [ ] **Step 3: Implement the document core**

Replace the placeholder in `src/document.rs` (keep the test module). This is the central type; implement construction, linking, navigation, attributes, text, and deletion:

```rust
//! The document: owns the arena and exposes navigation/mutation.

use crate::arena::{Arena, NodeId};
use crate::attribute::{Attribute, XmlValue};
use crate::error::{Result, XmlError};
use crate::node::{ElementData, NodeData, NodeKind, TextData, Whitespace};

pub const MAX_ELEMENT_DEPTH: i32 = 500;

pub struct XmlDocument {
    pub(crate) nodes: Arena<NodeData>,
    root: NodeId,
    pub(crate) whitespace_mode: Whitespace,
    pub(crate) process_entities: bool,
    pub(crate) write_bom: bool,
    pub(crate) error: Option<XmlError>,
    pub(crate) error_str: String,
    pub(crate) error_line: i32,
}

impl XmlDocument {
    pub fn new() -> Self {
        let mut nodes = Arena::new();
        let root = nodes.insert(NodeData::new(NodeKind::Document, String::new()));
        XmlDocument {
            nodes,
            root,
            whitespace_mode: Whitespace::Preserve,
            process_entities: true,
            write_bom: false,
            error: None,
            error_str: String::new(),
            error_line: 0,
        }
    }

    // ---- options ----
    pub fn set_whitespace_mode(&mut self, mode: Whitespace) {
        self.whitespace_mode = mode;
    }
    pub fn set_process_entities(&mut self, on: bool) {
        self.process_entities = on;
    }

    // ---- error state ----
    pub fn error(&self) -> Option<XmlError> {
        self.error
    }
    pub fn error_str(&self) -> &str {
        &self.error_str
    }
    pub fn error_line(&self) -> i32 {
        self.error_line
    }
    pub(crate) fn set_error(&mut self, err: XmlError, line: i32, detail: &str) -> XmlError {
        self.error = Some(err);
        self.error_line = line;
        self.error_str = format!("{} (line {}): {}", err.name(), line, detail);
        err
    }

    // ---- node access ----
    pub fn root(&self) -> NodeId {
        self.root
    }
    pub(crate) fn node(&self, id: NodeId) -> &NodeData {
        self.nodes.get(id).expect("stale NodeId")
    }
    pub(crate) fn node_mut(&mut self, id: NodeId) -> &mut NodeData {
        self.nodes.get_mut(id).expect("stale NodeId")
    }
    pub fn is_valid(&self, id: NodeId) -> bool {
        self.nodes.contains(id)
    }

    // ---- construction ----
    pub fn new_element(&mut self, name: &str) -> NodeId {
        self.nodes.insert(NodeData::new(
            NodeKind::Element(ElementData::default()),
            name.to_string(),
        ))
    }
    pub fn new_text(&mut self, text: &str) -> NodeId {
        self.nodes
            .insert(NodeData::new(NodeKind::Text(TextData { cdata: false }), text.to_string()))
    }
    pub fn new_comment(&mut self, text: &str) -> NodeId {
        self.nodes.insert(NodeData::new(NodeKind::Comment, text.to_string()))
    }
    pub fn new_declaration(&mut self, text: &str) -> NodeId {
        self.nodes.insert(NodeData::new(NodeKind::Declaration, text.to_string()))
    }
    pub fn new_unknown(&mut self, text: &str) -> NodeId {
        self.nodes.insert(NodeData::new(NodeKind::Unknown, text.to_string()))
    }

    // ---- linking ----
    pub fn insert_end_child(&mut self, parent: NodeId, child: NodeId) -> NodeId {
        self.node_mut(child).parent = Some(parent);
        match self.node(parent).last_child {
            Some(last) => {
                self.node_mut(last).next_sibling = Some(child);
                self.node_mut(child).prev_sibling = Some(last);
                self.node_mut(parent).last_child = Some(child);
            }
            None => {
                self.node_mut(parent).first_child = Some(child);
                self.node_mut(parent).last_child = Some(child);
            }
        }
        child
    }

    pub fn insert_first_child(&mut self, parent: NodeId, child: NodeId) -> NodeId {
        self.node_mut(child).parent = Some(parent);
        match self.node(parent).first_child {
            Some(first) => {
                self.node_mut(first).prev_sibling = Some(child);
                self.node_mut(child).next_sibling = Some(first);
                self.node_mut(parent).first_child = Some(child);
            }
            None => {
                self.node_mut(parent).first_child = Some(child);
                self.node_mut(parent).last_child = Some(child);
            }
        }
        child
    }

    fn detach(&mut self, id: NodeId) {
        let (parent, prev, next) = {
            let n = self.node(id);
            (n.parent, n.prev_sibling, n.next_sibling)
        };
        match prev {
            Some(p) => self.node_mut(p).next_sibling = next,
            None => {
                if let Some(par) = parent {
                    self.node_mut(par).first_child = next;
                }
            }
        }
        match next {
            Some(n) => self.node_mut(n).prev_sibling = prev,
            None => {
                if let Some(par) = parent {
                    self.node_mut(par).last_child = prev;
                }
            }
        }
        let n = self.node_mut(id);
        n.parent = None;
        n.prev_sibling = None;
        n.next_sibling = None;
    }

    /// Detach `id` from its parent and recursively remove it and its subtree.
    pub fn delete_node(&mut self, id: NodeId) {
        self.detach(id);
        self.delete_children(id);
        self.nodes.remove(id);
    }

    /// Recursively remove all descendants of `id` (but not `id` itself).
    pub fn delete_children(&mut self, id: NodeId) {
        let mut child = self.node(id).first_child;
        while let Some(c) = child {
            let next = self.node(c).next_sibling;
            self.delete_children(c);
            self.nodes.remove(c);
            child = next;
        }
        let n = self.node_mut(id);
        n.first_child = None;
        n.last_child = None;
    }

    // ---- navigation ----
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).parent
    }
    pub fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).first_child
    }
    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.node(id).next_sibling
    }

    fn matches_element(&self, id: NodeId, name: Option<&str>) -> bool {
        let n = self.node(id);
        n.is_element() && name.map_or(true, |want| n.value == want)
    }

    pub fn first_child_element(&self, id: NodeId, name: Option<&str>) -> Option<NodeId> {
        let mut child = self.node(id).first_child;
        while let Some(c) = child {
            if self.matches_element(c, name) {
                return Some(c);
            }
            child = self.node(c).next_sibling;
        }
        None
    }

    pub fn next_sibling_element(&self, id: NodeId, name: Option<&str>) -> Option<NodeId> {
        let mut sib = self.node(id).next_sibling;
        while let Some(s) = sib {
            if self.matches_element(s, name) {
                return Some(s);
            }
            sib = self.node(s).next_sibling;
        }
        None
    }

    /// The first child element of the document (the document root element).
    pub fn root_element(&self) -> Option<NodeId> {
        self.first_child_element(self.root, None)
    }

    // ---- element name & text ----
    pub fn name(&self, id: NodeId) -> Option<&str> {
        let n = self.node(id);
        if n.is_element() {
            Some(&n.value)
        } else {
            None
        }
    }
    pub fn set_name(&mut self, id: NodeId, name: &str) {
        self.node_mut(id).value = name.to_string();
    }

    /// Text of an element: the value of its first child text node.
    pub fn text(&self, id: NodeId) -> Option<&str> {
        let child = self.node(id).first_child?;
        let n = self.node(child);
        if n.is_text() {
            Some(&n.value)
        } else {
            None
        }
    }

    pub fn set_text(&mut self, id: NodeId, text: &str) {
        if let Some(child) = self.node(id).first_child {
            if self.node(child).is_text() {
                self.node_mut(child).value = text.to_string();
                return;
            }
        }
        let t = self.new_text(text);
        self.insert_first_child(id, t);
    }

    // ---- attributes ----
    fn element_data(&self, id: NodeId) -> Option<&ElementData> {
        match &self.node(id).kind {
            NodeKind::Element(d) => Some(d),
            _ => None,
        }
    }
    fn element_data_mut(&mut self, id: NodeId) -> Option<&mut ElementData> {
        match &mut self.node_mut(id).kind {
            NodeKind::Element(d) => Some(d),
            _ => None,
        }
    }

    pub fn attribute(&self, id: NodeId, name: &str) -> Option<&str> {
        self.element_data(id)?
            .attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.as_str())
    }

    pub fn set_attribute<V: XmlValue>(&mut self, id: NodeId, name: &str, value: V) {
        let value = value.to_xml_string();
        if let Some(data) = self.element_data_mut(id) {
            if let Some(a) = data.attributes.iter_mut().find(|a| a.name == name) {
                a.value = value;
            } else {
                data.attributes.push(Attribute { name: name.to_string(), value });
            }
        }
    }

    pub fn delete_attribute(&mut self, id: NodeId, name: &str) {
        if let Some(data) = self.element_data_mut(id) {
            data.attributes.retain(|a| a.name != name);
        }
    }

    fn find_attribute(&self, id: NodeId, name: &str) -> Result<&Attribute> {
        self.element_data(id)
            .and_then(|d| d.attributes.iter().find(|a| a.name == name))
            .ok_or(XmlError::NoAttribute)
    }

    pub fn query_int_attribute(&self, id: NodeId, name: &str) -> Result<i32> {
        self.find_attribute(id, name)?.as_i32()
    }
    pub fn query_int64_attribute(&self, id: NodeId, name: &str) -> Result<i64> {
        self.find_attribute(id, name)?.as_i64()
    }
    pub fn query_unsigned_attribute(&self, id: NodeId, name: &str) -> Result<u32> {
        self.find_attribute(id, name)?.as_u32()
    }
    pub fn query_float_attribute(&self, id: NodeId, name: &str) -> Result<f32> {
        self.find_attribute(id, name)?.as_f32()
    }
    pub fn query_double_attribute(&self, id: NodeId, name: &str) -> Result<f64> {
        self.find_attribute(id, name)?.as_f64()
    }
    pub fn query_bool_attribute(&self, id: NodeId, name: &str) -> Result<bool> {
        self.find_attribute(id, name)?.as_bool()
    }
}

impl Default for XmlDocument {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib document`
Expected: PASS (4 tests).

- [ ] **Step 5: Re-enable the re-exports in `src/lib.rs`**

Uncomment the three `pub use` lines from Task 1, Step 5.

Run: `cargo build`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/document.rs src/lib.rs
git commit -m "feat: add XmlDocument with tree construction, navigation, attributes"
```

---

## Task 8: Child iterators

**Files:**
- Modify: `src/document.rs` (add iterator methods + a `ChildElements` struct)
- Test: inline `#[cfg(test)]` in `src/document.rs`

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `src/document.rs`:

```rust
#[test]
fn iterate_child_elements() {
    let mut doc = XmlDocument::new();
    let root = doc.new_element("root");
    doc.insert_end_child(doc.root(), root);
    for n in ["a", "b", "c"] {
        let e = doc.new_element(n);
        doc.insert_end_child(root, e);
    }
    let names: Vec<String> =
        doc.child_elements(root, None).map(|id| doc.name(id).unwrap().to_string()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib iterate_child_elements`
Expected: FAIL — `child_elements` not defined.

- [ ] **Step 3: Implement the iterator**

Add to `src/document.rs` (outside the `impl XmlDocument` block for the struct, and a method inside it):

```rust
/// Iterator over child elements of a node, optionally filtered by name.
pub struct ChildElements<'a> {
    doc: &'a XmlDocument,
    next: Option<NodeId>,
    name: Option<&'a str>,
}

impl<'a> Iterator for ChildElements<'a> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        while let Some(cur) = self.next {
            self.next = self.doc.node(cur).next_sibling;
            if self.doc.matches_element(cur, self.name) {
                return Some(cur);
            }
        }
        None
    }
}
```

Add this method inside `impl XmlDocument` (and change `matches_element` from `fn` to `pub(crate) fn`):

```rust
    pub fn child_elements<'a>(&'a self, id: NodeId, name: Option<&'a str>) -> ChildElements<'a> {
        ChildElements { doc: self, next: self.node(id).first_child, name }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib iterate_child_elements`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/document.rs
git commit -m "feat: add child element iterator"
```

---

## Task 9: Parser — declarations, elements, attributes, text

**Files:**
- Modify: `src/parser.rs`
- Modify: `src/document.rs` (add `pub fn parse`)
- Test: inline `#[cfg(test)]` in `src/parser.rs`

Reference: `XMLDocument::Parse`, `XMLNode::ParseDeep`, `XMLElement::ParseAttributes`, `StrPair` parsing in `tinyxml2.cpp`.

- [ ] **Step 1: Write the failing tests**

In `src/parser.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::XmlDocument;

    #[test]
    fn parse_simple_element() {
        let mut doc = XmlDocument::new();
        doc.parse("<root/>").unwrap();
        let root = doc.root_element().unwrap();
        assert_eq!(doc.name(root), Some("root"));
    }

    #[test]
    fn parse_nested_with_text_and_attrs() {
        let mut doc = XmlDocument::new();
        doc.parse(r#"<a id="1"><b>hi</b></a>"#).unwrap();
        let a = doc.root_element().unwrap();
        assert_eq!(doc.attribute(a, "id"), Some("1"));
        let b = doc.first_child_element(a, Some("b")).unwrap();
        assert_eq!(doc.text(b), Some("hi"));
    }

    #[test]
    fn parse_decodes_entities_in_text_and_attrs() {
        let mut doc = XmlDocument::new();
        doc.parse(r#"<a x="a&amp;b">1&lt;2</a>"#).unwrap();
        let a = doc.root_element().unwrap();
        assert_eq!(doc.attribute(a, "x"), Some("a&b"));
        assert_eq!(doc.text(a), Some("1<2"));
    }

    #[test]
    fn parse_declaration_and_comment() {
        let mut doc = XmlDocument::new();
        doc.parse(r#"<?xml version="1.0"?><!-- hi --><root/>"#).unwrap();
        assert_eq!(doc.name(doc.root_element().unwrap()), Some("root"));
    }

    #[test]
    fn parse_empty_document_errors() {
        let mut doc = XmlDocument::new();
        assert!(doc.parse("   ").is_err());
    }

    #[test]
    fn parse_mismatched_element_errors() {
        let mut doc = XmlDocument::new();
        let err = doc.parse("<a></b>").unwrap_err();
        assert_eq!(err, crate::XmlError::ErrorMismatchedElement);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib parser`
Expected: FAIL — `parse` not defined.

- [ ] **Step 3: Implement the parser**

Replace the placeholder in `src/parser.rs` (keep the test module). Implement a recursive-descent / cursor parser. Key structure:

```rust
//! Parse state machine: turns a UTF-8 string into the document tree.

use crate::arena::NodeId;
use crate::document::{XmlDocument, MAX_ELEMENT_DEPTH};
use crate::error::{Result, XmlError};
use crate::node::{NodeKind, TextData};
use crate::strutil::{decode_entities, is_name_char, is_name_start_char, is_whitespace, strip_bom};

/// Cursor over the input with line tracking.
struct Cursor<'a> {
    s: &'a str,
    pos: usize,
    line: i32,
}

impl<'a> Cursor<'a> {
    fn new(s: &'a str) -> Self {
        Cursor { s, pos: 0, line: 1 }
    }
    fn rest(&self) -> &'a str {
        &self.s[self.pos..]
    }
    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }
    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        if ch == '\n' {
            self.line += 1;
        }
        self.pos += ch.len_utf8();
        Some(ch)
    }
    fn starts_with(&self, p: &str) -> bool {
        self.rest().starts_with(p)
    }
    fn consume(&mut self, p: &str) -> bool {
        if self.starts_with(p) {
            for _ in 0..p.chars().count() {
                self.bump();
            }
            true
        } else {
            false
        }
    }
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if is_whitespace(c)) {
            self.bump();
        }
    }
    /// Take chars while predicate holds; returns the slice.
    fn take_while<F: Fn(char) -> bool>(&mut self, pred: F) -> &'a str {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if pred(c)) {
            self.bump();
        }
        &self.s[start..self.pos]
    }
    /// Find a literal terminator; returns the slice up to it and advances past it.
    fn take_until(&mut self, term: &str) -> Option<&'a str> {
        let rel = self.rest().find(term)?;
        let start = self.pos;
        let chunk = &self.s[start..start + rel];
        // advance over chunk + terminator, tracking lines
        for _ in 0..chunk.chars().count() + term.chars().count() {
            self.bump();
        }
        Some(chunk)
    }
}

impl XmlDocument {
    /// Parse an XML document from a string, replacing any existing content.
    pub fn parse(&mut self, input: &str) -> Result<()> {
        // reset
        let root = self.root();
        self.delete_children(root);
        self.error = None;

        let (body, had_bom) = strip_bom(input);
        self.write_bom = had_bom;

        let mut cur = Cursor::new(body);
        let process_entities = self.process_entities;

        // Parse top-level nodes into the document root.
        parse_node_list(self, &mut cur, root, 0, process_entities, None)?;

        if self.root_element().is_none() {
            let line = cur.line;
            return Err(self.set_error(XmlError::ErrorEmptyDocument, line, "no root element"));
        }
        Ok(())
    }
}

/// Parse a sequence of nodes as children of `parent` until EOF or the matching
/// close tag for `open_name` (when inside an element).
fn parse_node_list(
    doc: &mut XmlDocument,
    cur: &mut Cursor,
    parent: NodeId,
    depth: i32,
    process_entities: bool,
    open_name: Option<&str>,
) -> Result<()> {
    if depth > MAX_ELEMENT_DEPTH {
        let line = cur.line;
        return Err(doc.set_error(XmlError::ElementDepthExceeded, line, "too deep"));
    }
    loop {
        // Text up to next '<'
        if !cur.starts_with("<") {
            let raw = cur.take_while(|c| c != '<');
            if !raw.is_empty() {
                let text = if process_entities { decode_entities(raw) } else { raw.to_string() };
                // Only emit non-pure-whitespace text, or any text inside an element.
                if open_name.is_some() || !text.trim().is_empty() {
                    let t = doc.new_text(&text);
                    doc.insert_end_child(parent, t);
                }
            }
            if cur.peek().is_none() {
                return Ok(());
            }
        }

        // Now positioned at '<'
        if cur.consume("</") {
            // closing tag
            cur.skip_ws();
            let name = cur.take_while(is_name_char);
            cur.skip_ws();
            let line = cur.line;
            if !cur.consume(">") {
                return Err(doc.set_error(XmlError::ErrorParsingElement, line, "bad close tag"));
            }
            match open_name {
                Some(expected) if expected == name => return Ok(()),
                _ => return Err(doc.set_error(XmlError::ErrorMismatchedElement, line, name)),
            }
        } else if cur.starts_with("<!--") {
            cur.consume("<!--");
            let line = cur.line;
            let body = cur
                .take_until("-->")
                .ok_or_else(|| doc.set_error(XmlError::ErrorParsingComment, line, "unterminated"))?;
            let c = doc.new_comment(body);
            doc.insert_end_child(parent, c);
        } else if cur.starts_with("<![CDATA[") {
            cur.consume("<![CDATA[");
            let line = cur.line;
            let body = cur
                .take_until("]]>")
                .ok_or_else(|| doc.set_error(XmlError::ErrorParsingCData, line, "unterminated"))?;
            let t = doc.nodes.insert(crate::node::NodeData::new(
                NodeKind::Text(TextData { cdata: true }),
                body.to_string(),
            ));
            doc.insert_end_child(parent, t);
        } else if cur.starts_with("<?") {
            cur.consume("<?");
            let line = cur.line;
            let body = cur
                .take_until("?>")
                .ok_or_else(|| doc.set_error(XmlError::ErrorParsingDeclaration, line, "unterminated"))?;
            let d = doc.new_declaration(body);
            doc.insert_end_child(parent, d);
        } else if cur.starts_with("<!") {
            cur.consume("<!");
            let line = cur.line;
            let body = cur
                .take_until(">")
                .ok_or_else(|| doc.set_error(XmlError::ErrorParsingUnknown, line, "unterminated"))?;
            let u = doc.new_unknown(body);
            doc.insert_end_child(parent, u);
        } else if cur.consume("<") {
            // opening element
            let line = cur.line;
            let name = cur.take_while(is_name_char);
            if name.is_empty() || !name.starts_with(is_name_start_char) {
                return Err(doc.set_error(XmlError::ErrorParsingElement, line, "bad name"));
            }
            let el = doc.new_element(name);
            doc.insert_end_child(parent, el);
            parse_attributes(doc, cur, el, process_entities)?;
            cur.skip_ws();
            if cur.consume("/>") {
                // self-closing
            } else if cur.consume(">") {
                // recurse into children until matching close
                parse_node_list(doc, cur, el, depth + 1, process_entities, Some(name))?;
            } else {
                let l = cur.line;
                return Err(doc.set_error(XmlError::ErrorParsingElement, l, "bad open tag"));
            }
        } else {
            // shouldn't happen
            return Ok(());
        }
    }
}

fn parse_attributes(
    doc: &mut XmlDocument,
    cur: &mut Cursor,
    el: NodeId,
    process_entities: bool,
) -> Result<()> {
    loop {
        cur.skip_ws();
        match cur.peek() {
            Some('>') | Some('/') | None => return Ok(()),
            _ => {}
        }
        let line = cur.line;
        let name = cur.take_while(is_name_char);
        if name.is_empty() {
            return Err(doc.set_error(XmlError::ErrorParsingAttribute, line, "bad attr name"));
        }
        cur.skip_ws();
        if !cur.consume("=") {
            return Err(doc.set_error(XmlError::ErrorParsingAttribute, cur.line, "missing ="));
        }
        cur.skip_ws();
        let quote = match cur.bump() {
            Some(q @ '"') | Some(q @ '\'') => q,
            _ => return Err(doc.set_error(XmlError::ErrorParsingAttribute, cur.line, "missing quote")),
        };
        let term = if quote == '"' { "\"" } else { "'" };
        let raw = cur
            .take_until(term)
            .ok_or_else(|| doc.set_error(XmlError::ErrorParsingAttribute, line, "unterminated value"))?;
        let value = if process_entities { decode_entities(raw) } else { raw.to_string() };
        doc.set_attribute(el, name, value.as_str());
    }
}
```

Note: this requires `nodes` and `new_*`/`insert_end_child`/`set_error`/`delete_children`/`root` to be accessible from `parser.rs`; they are `pub(crate)` or `pub` per Task 7. Add `use crate::node::NodeData;` is covered via the fully-qualified path used above.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib parser`
Expected: PASS (6 tests).

- [ ] **Step 5: Commit**

```bash
git add src/parser.rs src/document.rs
git commit -m "feat: add XML parser (elements, attributes, text, comments, CDATA, decl)"
```

---

## Task 10: Visitor trait & traversal

**Files:**
- Modify: `src/visitor.rs`
- Modify: `src/document.rs` (add `accept`)
- Test: inline `#[cfg(test)]` in `src/visitor.rs`

Reference: `XMLVisitor`, `XMLNode::Accept` in `tinyxml2.cpp`.

- [ ] **Step 1: Write the failing test**

In `src/visitor.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::NodeId;
    use crate::XmlDocument;

    #[derive(Default)]
    struct Counter {
        elements: usize,
        texts: usize,
    }
    impl XmlVisitor for Counter {
        fn visit_enter_element(&mut self, _doc: &XmlDocument, _id: NodeId) -> bool {
            self.elements += 1;
            true
        }
        fn visit_text(&mut self, _doc: &XmlDocument, _id: NodeId) -> bool {
            self.texts += 1;
            true
        }
    }

    #[test]
    fn visitor_counts_nodes() {
        let mut doc = XmlDocument::new();
        doc.parse("<a><b>hi</b><c/></a>").unwrap();
        let mut v = Counter::default();
        doc.accept(doc.root(), &mut v);
        assert_eq!(v.elements, 3); // a, b, c
        assert_eq!(v.texts, 1); // hi
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib visitor`
Expected: FAIL — `XmlVisitor` / `accept` not defined.

- [ ] **Step 3: Implement the visitor**

Replace the placeholder in `src/visitor.rs` (keep the test module):

```rust
//! Visitor pattern over the document tree (port of `XMLVisitor`).

use crate::arena::NodeId;
use crate::document::XmlDocument;

/// Returning `false` from an `enter` method prunes that subtree.
#[allow(unused_variables)]
pub trait XmlVisitor {
    fn visit_enter_document(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        true
    }
    fn visit_exit_document(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        true
    }
    fn visit_enter_element(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        true
    }
    fn visit_exit_element(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        true
    }
    fn visit_text(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        true
    }
    fn visit_comment(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        true
    }
    fn visit_declaration(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        true
    }
    fn visit_unknown(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        true
    }
}
```

Add to `impl XmlDocument` in `src/document.rs`:

```rust
    /// Traverse the subtree rooted at `id`, dispatching to `visitor`.
    pub fn accept(&self, id: NodeId, visitor: &mut dyn crate::visitor::XmlVisitor) {
        use crate::node::NodeKind;
        match &self.node(id).kind {
            NodeKind::Document => {
                if visitor.visit_enter_document(self, id) {
                    self.accept_children(id, visitor);
                }
                visitor.visit_exit_document(self, id);
            }
            NodeKind::Element(_) => {
                if visitor.visit_enter_element(self, id) {
                    self.accept_children(id, visitor);
                }
                visitor.visit_exit_element(self, id);
            }
            NodeKind::Text(_) => {
                visitor.visit_text(self, id);
            }
            NodeKind::Comment => {
                visitor.visit_comment(self, id);
            }
            NodeKind::Declaration => {
                visitor.visit_declaration(self, id);
            }
            NodeKind::Unknown => {
                visitor.visit_unknown(self, id);
            }
        }
    }

    fn accept_children(&self, id: NodeId, visitor: &mut dyn crate::visitor::XmlVisitor) {
        let mut child = self.node(id).first_child;
        while let Some(c) = child {
            self.accept(c, visitor);
            child = self.node(c).next_sibling;
        }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib visitor`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/visitor.rs src/document.rs
git commit -m "feat: add XmlVisitor trait and accept() traversal"
```

---

## Task 11: Printer (serialization)

**Files:**
- Modify: `src/printer.rs`
- Modify: `src/document.rs` (add `print_to_string` / `print`)
- Test: inline `#[cfg(test)]` in `src/printer.rs`

Reference: `XMLPrinter` in `tinyxml2.cpp`; options at `tinyxml2.h:2236`.

- [ ] **Step 1: Write the failing tests**

In `src/printer.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::XmlDocument;

    #[test]
    fn roundtrip_compact() {
        let mut doc = XmlDocument::new();
        doc.parse(r#"<a id="1"><b>hi&amp;bye</b><c/></a>"#).unwrap();
        let out = doc.print_to_string(true);
        assert_eq!(out, r#"<a id="1"><b>hi&amp;bye</b><c/></a>"#);
    }

    #[test]
    fn pretty_indents() {
        let mut doc = XmlDocument::new();
        doc.parse("<a><b/></a>").unwrap();
        let out = doc.print_to_string(false);
        assert_eq!(out, "<a>\n    <b/>\n</a>\n");
    }

    #[test]
    fn escapes_attributes_and_text() {
        let mut doc = XmlDocument::new();
        let e = doc.new_element("e");
        doc.insert_end_child(doc.root(), e);
        doc.set_attribute(e, "x", "a<b&c");
        doc.set_text(e, "1<2&3");
        let out = doc.print_to_string(true);
        assert_eq!(out, r#"<e x="a&lt;b&amp;c">1&lt;2&amp;3</e>"#);
    }

    #[test]
    fn prints_comment_and_declaration() {
        let mut doc = XmlDocument::new();
        doc.parse(r#"<?xml version="1.0"?><!--c--><a/>"#).unwrap();
        let out = doc.print_to_string(true);
        assert_eq!(out, r#"<?xml version="1.0"?><!--c--><a/>"#);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib printer`
Expected: FAIL — `print_to_string` not defined.

- [ ] **Step 3: Implement the printer**

Replace the placeholder in `src/printer.rs` (keep the test module). Implement an `XmlPrinter` as a visitor that writes to a `String`:

```rust
//! Serialization to XML text (port of `XMLPrinter`).

use crate::arena::NodeId;
use crate::document::XmlDocument;
use crate::node::NodeKind;
use crate::strutil::encode_text;
use crate::visitor::XmlVisitor;

/// Accumulates serialized XML. `compact == false` produces indented output.
pub struct XmlPrinter {
    out: String,
    compact: bool,
    depth: usize,
    /// True while an element's open tag is written but not yet sealed (`>` or `/>`).
    element_open: bool,
    /// Track whether the just-opened element has element children (for pretty mode).
    first_element: bool,
}

impl XmlPrinter {
    pub fn new(compact: bool) -> Self {
        XmlPrinter {
            out: String::new(),
            compact,
            depth: 0,
            element_open: false,
            first_element: true,
        }
    }

    pub fn into_string(self) -> String {
        self.out
    }

    fn indent(&mut self) {
        if !self.compact {
            for _ in 0..self.depth {
                self.out.push_str("    ");
            }
        }
    }

    fn newline(&mut self) {
        if !self.compact {
            self.out.push('\n');
        }
    }

    /// Close a pending open tag with `>`.
    fn seal(&mut self) {
        if self.element_open {
            self.out.push('>');
            self.element_open = false;
        }
    }
}

impl XmlVisitor for XmlPrinter {
    fn visit_enter_element(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        self.seal();
        if !self.first_element {
            self.newline();
        }
        self.indent();
        self.first_element = false;

        let n = doc.node(id);
        self.out.push('<');
        self.out.push_str(&n.value);
        if let NodeKind::Element(data) = &n.kind {
            for attr in &data.attributes {
                self.out.push(' ');
                self.out.push_str(&attr.name);
                self.out.push_str("=\"");
                self.out.push_str(&encode_text(&attr.value));
                self.out.push('"');
            }
        }
        self.element_open = true;
        self.depth += 1;
        true
    }

    fn visit_exit_element(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        self.depth -= 1;
        let n = doc.node(id);
        let has_children = n.first_child.is_some();
        let only_text = n.first_child.is_some()
            && doc.node(n.first_child.unwrap()).is_text()
            && doc.node(n.first_child.unwrap()).next_sibling.is_none();

        if self.element_open {
            // no children at all → self-close
            self.out.push_str("/>");
            self.element_open = false;
        } else {
            if has_children && !only_text {
                self.newline();
                self.indent();
            }
            self.out.push_str("</");
            self.out.push_str(&n.value);
            self.out.push('>');
        }
        true
    }

    fn visit_text(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        self.seal();
        let n = doc.node(id);
        if let NodeKind::Text(t) = &n.kind {
            if t.cdata {
                self.out.push_str("<![CDATA[");
                self.out.push_str(&n.value);
                self.out.push_str("]]>");
            } else {
                self.out.push_str(&encode_text(&n.value));
            }
        }
        true
    }

    fn visit_comment(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        self.seal();
        if !self.first_element {
            self.newline();
            self.indent();
        }
        self.first_element = false;
        self.out.push_str("<!--");
        self.out.push_str(&doc.node(id).value);
        self.out.push_str("-->");
        true
    }

    fn visit_declaration(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        self.seal();
        if !self.first_element {
            self.newline();
            self.indent();
        }
        self.first_element = false;
        self.out.push_str("<?");
        self.out.push_str(&doc.node(id).value);
        self.out.push_str("?>");
        true
    }

    fn visit_unknown(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
        self.seal();
        self.out.push_str("<!");
        self.out.push_str(&doc.node(id).value);
        self.out.push('>');
        true
    }
}
```

Add to `impl XmlDocument` in `src/document.rs`:

```rust
    /// Serialize the whole document to a string. `compact` removes indentation.
    pub fn print_to_string(&self, compact: bool) -> String {
        let mut printer = crate::printer::XmlPrinter::new(compact);
        // Visit each top-level node under the document root.
        let mut child = self.node(self.root()).first_child;
        while let Some(c) = child {
            self.accept(c, &mut printer);
            child = self.node(c).next_sibling;
        }
        let mut s = printer.into_string();
        if !compact && !s.ends_with('\n') {
            s.push('\n');
        }
        s
    }
```

Note on `pretty_indents` test: the expected `"<a>\n    <b/>\n</a>\n"` defines the canonical pretty layout — element children go on indented new lines; a self-closing child renders as `<b/>`. Implement `visit_exit_element` to match: when an element has element children, place the close tag on a new indented line. Adjust spacing logic until the four printer tests pass; the tests are the contract.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib printer`
Expected: PASS (4 tests). If pretty-mode spacing differs, refine `visit_enter_element`/`visit_exit_element` newline/indent placement until the `pretty_indents` and `roundtrip_compact` tests both pass.

- [ ] **Step 5: Commit**

```bash
git add src/printer.rs src/document.rs
git commit -m "feat: add XmlPrinter serialization (compact + pretty)"
```

---

## Task 12: XmlHandle (null-safe navigation)

**Files:**
- Modify: `src/handle.rs`
- Test: inline `#[cfg(test)]` in `src/handle.rs`

Reference: `XMLHandle` / `XMLConstHandle` in `tinyxml2.h:2051`.

- [ ] **Step 1: Write the failing test**

In `src/handle.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::XmlDocument;

    #[test]
    fn chained_navigation_is_null_safe() {
        let mut doc = XmlDocument::new();
        doc.parse("<a><b><c/></b></a>").unwrap();
        let root = doc.root_element().unwrap();

        let c = XmlHandle::new(&doc, root)
            .first_child_element(Some("b"))
            .first_child_element(Some("c"))
            .id();
        assert!(c.is_some());

        // A path that doesn't exist yields None instead of panicking.
        let missing = XmlHandle::new(&doc, root)
            .first_child_element(Some("z"))
            .first_child_element(Some("c"))
            .id();
        assert!(missing.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib handle`
Expected: FAIL — `XmlHandle` not defined.

- [ ] **Step 3: Implement the handle**

Replace the placeholder in `src/handle.rs` (keep the test module):

```rust
//! Null-safe chained navigation (port of `XMLHandle`).

use crate::arena::NodeId;
use crate::document::XmlDocument;

/// A cheap, copyable cursor that yields `None` once navigation leaves the tree.
#[derive(Clone, Copy)]
pub struct XmlHandle<'a> {
    doc: &'a XmlDocument,
    node: Option<NodeId>,
}

impl<'a> XmlHandle<'a> {
    pub fn new(doc: &'a XmlDocument, id: NodeId) -> Self {
        XmlHandle { doc, node: Some(id) }
    }

    fn wrap(self, node: Option<NodeId>) -> Self {
        XmlHandle { doc: self.doc, node }
    }

    pub fn id(self) -> Option<NodeId> {
        self.node
    }

    pub fn first_child(self) -> Self {
        let next = self.node.and_then(|id| self.doc.first_child(id));
        self.wrap(next)
    }

    pub fn first_child_element(self, name: Option<&str>) -> Self {
        let next = self.node.and_then(|id| self.doc.first_child_element(id, name));
        self.wrap(next)
    }

    pub fn next_sibling(self) -> Self {
        let next = self.node.and_then(|id| self.doc.next_sibling(id));
        self.wrap(next)
    }

    pub fn next_sibling_element(self, name: Option<&str>) -> Self {
        let next = self.node.and_then(|id| self.doc.next_sibling_element(id, name));
        self.wrap(next)
    }

    pub fn parent(self) -> Self {
        let next = self.node.and_then(|id| self.doc.parent(id));
        self.wrap(next)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib handle`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/handle.rs
git commit -m "feat: add XmlHandle for null-safe chained navigation"
```

---

## Task 13: Document file I/O + whitespace collapse

**Files:**
- Modify: `src/document.rs` (add `load_file`, `save_file`, apply whitespace mode in parse)
- Modify: `src/parser.rs` (collapse logic hook)
- Test: inline `#[cfg(test)]` in `src/document.rs`

Reference: `XMLDocument::LoadFile`/`SaveFile`; `XMLUtil` whitespace collapse.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `src/document.rs`:

```rust
#[test]
fn collapse_whitespace_mode() {
    let mut doc = XmlDocument::new();
    doc.set_whitespace_mode(crate::Whitespace::Collapse);
    doc.parse("<a>   hello     world   </a>").unwrap();
    let a = doc.root_element().unwrap();
    assert_eq!(doc.text(a), Some("hello world"));
}

#[test]
fn load_and_save_file_roundtrip() {
    let dir = std::env::temp_dir();
    let path = dir.join("rustxml2_io_test.xml");
    std::fs::write(&path, r#"<root a="1"><child/></root>"#).unwrap();

    let mut doc = XmlDocument::new();
    doc.load_file(&path).unwrap();
    assert_eq!(doc.name(doc.root_element().unwrap()), Some("root"));

    let out = dir.join("rustxml2_io_out.xml");
    doc.save_file(&out, true).unwrap();
    let written = std::fs::read_to_string(&out).unwrap();
    assert_eq!(written, r#"<root a="1"><child/></root>"#);

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&out);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib collapse_whitespace_mode load_and_save_file_roundtrip`
Expected: FAIL — `load_file` not defined / collapse not applied.

- [ ] **Step 3: Implement collapse + file I/O**

Add a collapse helper to `src/strutil.rs`:

```rust
/// Collapse runs of whitespace to a single space and trim ends (tinyxml2 COLLAPSE_WHITESPACE).
pub fn collapse_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_ws = false;
    for ch in input.trim().chars() {
        if is_whitespace(ch) {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(ch);
            in_ws = false;
        }
    }
    out
}
```

In `src/parser.rs`, in the text-handling branch of `parse_node_list`, apply collapse when enabled. Change the text construction block to:

```rust
            if !raw.is_empty() {
                let mut text =
                    if process_entities { decode_entities(raw) } else { raw.to_string() };
                if doc.whitespace_mode == crate::node::Whitespace::Collapse {
                    text = crate::strutil::collapse_whitespace(&text);
                }
                if open_name.is_some() || !text.trim().is_empty() {
                    if !(doc.whitespace_mode == crate::node::Whitespace::Collapse && text.is_empty())
                    {
                        let t = doc.new_text(&text);
                        doc.insert_end_child(parent, t);
                    }
                }
            }
```

Add to `impl XmlDocument` in `src/document.rs`:

```rust
    /// Load and parse an XML file (UTF-8).
    pub fn load_file(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                self.set_error(XmlError::FileNotFound, 0, "file not found")
            }
            std::io::ErrorKind::PermissionDenied => {
                self.set_error(XmlError::FileCouldNotBeOpened, 0, "permission denied")
            }
            _ => self.set_error(XmlError::FileReadError, 0, "read error"),
        })?;
        self.parse(&content)
    }

    /// Serialize and write the document to a file.
    pub fn save_file(&self, path: &std::path::Path, compact: bool) -> Result<()> {
        std::fs::write(path, self.print_to_string(compact))
            .map_err(|_| XmlError::FileCouldNotBeOpened)
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib collapse_whitespace_mode load_and_save_file_roundtrip`
Expected: PASS (2 tests).

- [ ] **Step 5: Run the whole lib test suite**

Run: `cargo test --lib`
Expected: PASS (all unit tests across modules).

- [ ] **Step 6: Commit**

```bash
git add src/document.rs src/parser.rs src/strutil.rs
git commit -m "feat: add file I/O and whitespace collapse mode"
```

---

## Task 14: Copy test fixtures

**Files:**
- Create: `tests/resources/` (copied from `D:\Workspace\Rust\Original\tinyxml2\resources\`)

- [ ] **Step 1: Copy the resources directory**

Run (PowerShell):

```powershell
New-Item -ItemType Directory -Force "D:\Workspace\Rust\rustxml2\tests\resources"
Copy-Item -Recurse -Force "D:\Workspace\Rust\Original\tinyxml2\resources\*" "D:\Workspace\Rust\rustxml2\tests\resources\"
```

- [ ] **Step 2: Verify fixtures are present**

Run: `ls tests/resources`
Expected: lists `dream.xml`, `utf8test.xml`, and the other fixtures used by `xmltest.cpp`.

- [ ] **Step 3: Commit**

```bash
git add tests/resources
git commit -m "test: add XML fixtures from tinyxml2 resources"
```

---

## Task 15: Ported integration tests — parsing & navigation

**Files:**
- Create: `tests/parsing.rs`

Reference: the parsing/navigation cases in `xmltest.cpp` (the `XMLTest(...)` calls around element/attribute/text walking).

- [ ] **Step 1: Write the integration tests**

Create `tests/parsing.rs`:

```rust
use rustxml2::XmlDocument;

#[test]
fn walk_known_document() {
    let mut doc = XmlDocument::new();
    doc.parse(
        r#"<?xml version="1.0"?>
<doc>
  <element attr="value">text</element>
  <empty/>
  <nested><inner>deep</inner></nested>
</doc>"#,
    )
    .unwrap();

    let root = doc.root_element().unwrap();
    assert_eq!(doc.name(root), Some("doc"));

    let element = doc.first_child_element(root, Some("element")).unwrap();
    assert_eq!(doc.attribute(element, "attr"), Some("value"));
    assert_eq!(doc.text(element), Some("text"));

    let empty = doc.first_child_element(root, Some("empty")).unwrap();
    assert!(doc.first_child(empty).is_none());

    let nested = doc.first_child_element(root, Some("nested")).unwrap();
    let inner = doc.first_child_element(nested, Some("inner")).unwrap();
    assert_eq!(doc.text(inner), Some("deep"));
}

#[test]
fn sibling_iteration_counts_elements() {
    let mut doc = XmlDocument::new();
    doc.parse("<list><i/><i/><i/><i/></list>").unwrap();
    let list = doc.root_element().unwrap();
    let count = doc.child_elements(list, Some("i")).count();
    assert_eq!(count, 4);
}

#[test]
fn error_cases_report_expected_codes() {
    let mut doc = XmlDocument::new();
    assert_eq!(doc.parse("<a></b>").unwrap_err(), rustxml2::XmlError::ErrorMismatchedElement);

    let mut doc2 = XmlDocument::new();
    assert_eq!(doc2.parse("").unwrap_err(), rustxml2::XmlError::ErrorEmptyDocument);
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test --test parsing`
Expected: PASS (3 tests).

- [ ] **Step 3: Commit**

```bash
git add tests/parsing.rs
git commit -m "test: port parsing & navigation cases from xmltest.cpp"
```

---

## Task 16: Ported integration tests — attributes & typed values

**Files:**
- Create: `tests/attributes.rs`

Reference: the attribute / typed-value cases in `xmltest.cpp` (`QueryIntAttribute`, `IntAttribute`, `SetAttribute`, bool/double conversions).

- [ ] **Step 1: Write the integration tests**

Create `tests/attributes.rs`:

```rust
use rustxml2::{XmlDocument, XmlError};

#[test]
fn typed_attribute_queries() {
    let mut doc = XmlDocument::new();
    doc.parse(r#"<e i="42" n="-7" f="3.5" b="true" big="10000000000"/>"#).unwrap();
    let e = doc.root_element().unwrap();

    assert_eq!(doc.query_int_attribute(e, "i"), Ok(42));
    assert_eq!(doc.query_int_attribute(e, "n"), Ok(-7));
    assert_eq!(doc.query_double_attribute(e, "f"), Ok(3.5));
    assert_eq!(doc.query_bool_attribute(e, "b"), Ok(true));
    assert_eq!(doc.query_int64_attribute(e, "big"), Ok(10_000_000_000));

    assert_eq!(doc.query_int_attribute(e, "missing"), Err(XmlError::NoAttribute));
    assert_eq!(doc.query_int_attribute(e, "f"), Err(XmlError::WrongAttributeType));
}

#[test]
fn set_attributes_of_each_type() {
    let mut doc = XmlDocument::new();
    let e = doc.new_element("e");
    doc.insert_end_child(doc.root(), e);

    doc.set_attribute(e, "i", 5i32);
    doc.set_attribute(e, "f", 2.25f64);
    doc.set_attribute(e, "b", false);
    doc.set_attribute(e, "s", "txt");

    assert_eq!(doc.attribute(e, "i"), Some("5"));
    assert_eq!(doc.attribute(e, "f"), Some("2.25"));
    assert_eq!(doc.attribute(e, "b"), Some("false"));
    assert_eq!(doc.attribute(e, "s"), Some("txt"));
}

#[test]
fn overwriting_attribute_replaces_value() {
    let mut doc = XmlDocument::new();
    let e = doc.new_element("e");
    doc.insert_end_child(doc.root(), e);
    doc.set_attribute(e, "x", 1i32);
    doc.set_attribute(e, "x", 2i32);
    assert_eq!(doc.attribute(e, "x"), Some("2"));
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test --test attributes`
Expected: PASS (3 tests).

- [ ] **Step 3: Commit**

```bash
git add tests/attributes.rs
git commit -m "test: port attribute & typed-value cases from xmltest.cpp"
```

---

## Task 17: Ported integration tests — printing & round-trip

**Files:**
- Create: `tests/printing.rs`

Reference: the print / round-trip cases in `xmltest.cpp` (`XMLPrinter`, `Print`, compact mode, entity escaping).

- [ ] **Step 1: Write the integration tests**

Create `tests/printing.rs`:

```rust
use rustxml2::XmlDocument;

fn roundtrip_compact(input: &str) -> String {
    let mut doc = XmlDocument::new();
    doc.parse(input).unwrap();
    doc.print_to_string(true)
}

#[test]
fn compact_roundtrip_is_stable() {
    let xml = r#"<?xml version="1.0"?><root a="1" b="2"><child>text</child><self/><!--c--></root>"#;
    let once = roundtrip_compact(xml);
    let twice = roundtrip_compact(&once);
    assert_eq!(once, twice, "round-trip must be idempotent");
}

#[test]
fn entities_are_escaped_on_output() {
    let out = roundtrip_compact(r#"<a x="&lt;&amp;">&gt;&quot;</a>"#);
    assert_eq!(out, r#"<a x="&lt;&amp;">&gt;&quot;</a>"#);
}

#[test]
fn cdata_is_preserved() {
    let out = roundtrip_compact("<a><![CDATA[<not parsed>]]></a>");
    assert_eq!(out, "<a><![CDATA[<not parsed>]]></a>");
}

#[test]
fn pretty_print_is_indented() {
    let mut doc = XmlDocument::new();
    doc.parse("<a><b><c/></b></a>").unwrap();
    let out = doc.print_to_string(false);
    assert_eq!(out, "<a>\n    <b>\n        <c/>\n    </b>\n</a>\n");
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test --test printing`
Expected: PASS (4 tests). If `pretty_print_is_indented` fails, adjust the printer's newline/indent placement (Task 11) — these tests are the canonical pretty-print contract.

- [ ] **Step 3: Commit**

```bash
git add tests/printing.rs
git commit -m "test: port printing & round-trip cases from xmltest.cpp"
```

---

## Task 18: Ported integration tests — fixtures, entities, deep nesting, visitor

**Files:**
- Create: `tests/fixtures.rs`

Reference: `xmltest.cpp` file-loading cases (`dream.xml`, `utf8test.xml`), deep-recursion guard, and visitor usage.

- [ ] **Step 1: Write the integration tests**

Create `tests/fixtures.rs`:

```rust
use rustxml2::{XmlDocument, XmlError};
use std::path::Path;

#[test]
fn load_dream_fixture() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/dream.xml");
    let mut doc = XmlDocument::new();
    doc.load_file(&path).unwrap();
    // dream.xml's root element is <!DOCTYPE ...> then <PLAY>. The first element is PLAY.
    let root = doc.root_element().unwrap();
    assert_eq!(doc.name(root), Some("PLAY"));
}

#[test]
fn utf8_content_roundtrips() {
    let mut doc = XmlDocument::new();
    doc.parse("<a>café — naïve — 日本語</a>").unwrap();
    let a = doc.root_element().unwrap();
    assert_eq!(doc.text(a), Some("café — naïve — 日本語"));
    let out = doc.print_to_string(true);
    assert_eq!(out, "<a>café — naïve — 日本語</a>");
}

#[test]
fn deep_nesting_is_rejected() {
    // Build input deeper than MAX_ELEMENT_DEPTH (500).
    let depth = 600;
    let mut s = String::new();
    for _ in 0..depth {
        s.push_str("<a>");
    }
    for _ in 0..depth {
        s.push_str("</a>");
    }
    let mut doc = XmlDocument::new();
    assert_eq!(doc.parse(&s).unwrap_err(), XmlError::ElementDepthExceeded);
}

#[test]
fn visitor_collects_element_names() {
    use rustxml2::arena::NodeId;
    use rustxml2::visitor::XmlVisitor;

    #[derive(Default)]
    struct Collector {
        names: Vec<String>,
    }
    impl XmlVisitor for Collector {
        fn visit_enter_element(&mut self, doc: &XmlDocument, id: NodeId) -> bool {
            self.names.push(doc.name(id).unwrap().to_string());
            true
        }
    }

    let mut doc = XmlDocument::new();
    doc.parse("<a><b/><c><d/></c></a>").unwrap();
    let mut v = Collector::default();
    doc.accept(doc.root(), &mut v);
    assert_eq!(v.names, vec!["a", "b", "c", "d"]);
}
```

Note: if `dream.xml`'s first element is not `PLAY`, open the fixture, find the actual root element name, and update the assertion to match. The fixture is the source of truth.

- [ ] **Step 2: Run the tests**

Run: `cargo test --test fixtures`
Expected: PASS (4 tests). Adjust the `dream.xml` root-name assertion if the fixture differs.

- [ ] **Step 3: Commit**

```bash
git add tests/fixtures.rs
git commit -m "test: port fixture, utf8, deep-nesting, and visitor cases"
```

---

## Task 19: Final verification & docs

**Files:**
- Modify: `src/lib.rs` (crate-level docs + example)

- [ ] **Step 1: Run the entire suite**

Run: `cargo test`
Expected: PASS — all unit + integration tests.

- [ ] **Step 2: Run clippy and fmt**

Run: `cargo clippy --all-targets -- -D warnings` then `cargo fmt`
Expected: no warnings; formatting clean. Fix any clippy findings.

- [ ] **Step 3: Add a crate-level doctest example to `src/lib.rs`**

Add at the top of `src/lib.rs` (after the existing `//!` line):

```rust
//!
//! ```
//! use rustxml2::XmlDocument;
//! let mut doc = XmlDocument::new();
//! doc.parse(r#"<note to="you">hello</note>"#).unwrap();
//! let note = doc.root_element().unwrap();
//! assert_eq!(doc.attribute(note, "to"), Some("you"));
//! assert_eq!(doc.text(note), Some("hello"));
//! ```
```

- [ ] **Step 4: Run the doctest**

Run: `cargo test --doc`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "docs: add crate-level example and finalize"
```

---

## Self-Review Notes (coverage map)

| Spec section | Covered by |
|---|---|
| Crate & module layout | Task 1 |
| Error handling (§8) | Task 2 |
| Arena + NodeId (§3) | Task 3 |
| String utils / entities / BOM (§5, §6) | Task 4 |
| Attribute + typed values (§4) | Task 5 |
| Node data model (§3) | Task 6 |
| Document, navigation, attributes, text, deletion (§4) | Task 7 |
| Iterators (§4) | Task 8 |
| Parsing (§5) | Task 9 |
| Visitor (§7) | Task 10 |
| Printer / serialization (§6) | Task 11 |
| Handle (§7) | Task 12 |
| File I/O + whitespace modes (§4, §5) | Task 13 |
| Test fixtures (§9) | Task 14 |
| Ported tests (§9) | Tasks 15–18 |
| Adaptations: no leak checks, UTF-8 cases (§9) | Tasks 18, 19 |

**Adaptation note:** Per the spec, `_CrtMemState` leak checks are not ported (Rust ownership), and invalid-UTF-8 cases are replaced with valid-UTF-8 equivalents (Task 18 `utf8_content_roundtrips`).
