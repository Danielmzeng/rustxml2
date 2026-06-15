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
