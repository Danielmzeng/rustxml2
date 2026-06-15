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

        let missing = XmlHandle::new(&doc, root)
            .first_child_element(Some("z"))
            .first_child_element(Some("c"))
            .id();
        assert!(missing.is_none());
    }
}
