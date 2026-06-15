//! rustxml2 — an idiomatic Rust port of the tinyxml2 XML DOM library.
//!
//! ```
//! use rustxml2::XmlDocument;
//! let mut doc = XmlDocument::new();
//! doc.parse(r#"<note to="you">hello</note>"#).unwrap();
//! let note = doc.root_element().unwrap();
//! assert_eq!(doc.attribute(note, "to"), Some("you"));
//! assert_eq!(doc.text(note), Some("hello"));
//! ```

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
