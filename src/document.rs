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

    pub(crate) fn matches_element(&self, id: NodeId, name: Option<&str>) -> bool {
        let n = self.node(id);
        n.is_element() && name.is_none_or(|want| n.value == want)
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

    pub fn child_elements<'a>(&'a self, id: NodeId, name: Option<&'a str>) -> ChildElements<'a> {
        ChildElements { doc: self, next: self.node(id).first_child, name }
    }
}

impl Default for XmlDocument {
    fn default() -> Self {
        Self::new()
    }
}

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
}
