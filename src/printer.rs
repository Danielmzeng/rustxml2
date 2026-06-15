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
    element_open: bool,
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
        let only_text = if let Some(fc) = n.first_child {
            doc.node(fc).is_text() && doc.node(fc).next_sibling.is_none()
        } else {
            false
        };

        if self.element_open {
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
