//! Node storage and kind discriminants.

use crate::arena::NodeId;
use crate::attribute::Attribute;

/// Whitespace handling mode (tinyxml2 `Whitespace`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Whitespace {
    #[default]
    Preserve,
    Collapse,
    Pedantic,
}

/// How an element was closed (tinyxml2 `ElementClosingType`).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosingType {
    Open,   // <foo>
    Closed, // <foo/>
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
