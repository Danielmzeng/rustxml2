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
    fn take_while<F: Fn(char) -> bool>(&mut self, pred: F) -> &'a str {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if pred(c)) {
            self.bump();
        }
        &self.s[start..self.pos]
    }
    fn take_until(&mut self, term: &str) -> Option<&'a str> {
        let rel = self.rest().find(term)?;
        let start = self.pos;
        let chunk = &self.s[start..start + rel];
        for _ in 0..chunk.chars().count() + term.chars().count() {
            self.bump();
        }
        Some(chunk)
    }
}

impl XmlDocument {
    /// Parse an XML document from a string, replacing any existing content.
    pub fn parse(&mut self, input: &str) -> Result<()> {
        let root = self.root();
        self.delete_children(root);
        self.error = None;

        let (body, had_bom) = strip_bom(input);
        self.write_bom = had_bom;

        let mut cur = Cursor::new(body);
        let process_entities = self.process_entities;

        parse_node_list(self, &mut cur, root, 0, process_entities, None)?;

        if self.root_element().is_none() {
            let line = cur.line;
            return Err(self.set_error(XmlError::ErrorEmptyDocument, line, "no root element"));
        }
        Ok(())
    }
}

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
        if !cur.starts_with("<") {
            let raw = cur.take_while(|c| c != '<');
            if !raw.is_empty() {
                let mut text = if process_entities {
                    decode_entities(raw)
                } else {
                    raw.to_string()
                };
                if doc.whitespace_mode == crate::node::Whitespace::Collapse {
                    text = crate::strutil::collapse_whitespace(&text);
                }
                if (open_name.is_some() || !text.trim().is_empty())
                    && !(doc.whitespace_mode == crate::node::Whitespace::Collapse
                        && text.is_empty())
                {
                    let t = doc.new_text(&text);
                    doc.insert_end_child(parent, t);
                }
            }
            if cur.peek().is_none() {
                return Ok(());
            }
        }

        if cur.consume("</") {
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
            let body_opt = cur.take_until("-->");
            let body = body_opt.ok_or_else(|| {
                doc.set_error(XmlError::ErrorParsingComment, line, "unterminated")
            })?;
            let c = doc.new_comment(body);
            doc.insert_end_child(parent, c);
        } else if cur.starts_with("<![CDATA[") {
            cur.consume("<![CDATA[");
            let line = cur.line;
            let body_opt = cur.take_until("]]>");
            let body = body_opt
                .ok_or_else(|| doc.set_error(XmlError::ErrorParsingCData, line, "unterminated"))?;
            let t = doc.nodes.insert(crate::node::NodeData::new(
                NodeKind::Text(TextData { cdata: true }),
                body.to_string(),
            ));
            doc.insert_end_child(parent, t);
        } else if cur.starts_with("<?") {
            cur.consume("<?");
            let line = cur.line;
            let body_opt = cur.take_until("?>");
            let body = body_opt.ok_or_else(|| {
                doc.set_error(XmlError::ErrorParsingDeclaration, line, "unterminated")
            })?;
            let d = doc.new_declaration(body);
            doc.insert_end_child(parent, d);
        } else if cur.starts_with("<!") {
            cur.consume("<!");
            let line = cur.line;
            let body_opt = cur.take_until(">");
            let body = body_opt.ok_or_else(|| {
                doc.set_error(XmlError::ErrorParsingUnknown, line, "unterminated")
            })?;
            let u = doc.new_unknown(body);
            doc.insert_end_child(parent, u);
        } else if cur.consume("<") {
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
                parse_node_list(doc, cur, el, depth + 1, process_entities, Some(name))?;
            } else {
                let l = cur.line;
                return Err(doc.set_error(XmlError::ErrorParsingElement, l, "bad open tag"));
            }
        } else {
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
            _ => {
                return Err(doc.set_error(
                    XmlError::ErrorParsingAttribute,
                    cur.line,
                    "missing quote",
                ))
            }
        };
        let term = if quote == '"' { "\"" } else { "'" };
        let attr_line = cur.line;
        let raw_opt = cur.take_until(term);
        let raw = raw_opt.ok_or_else(|| {
            doc.set_error(
                XmlError::ErrorParsingAttribute,
                attr_line,
                "unterminated value",
            )
        })?;
        let value = if process_entities {
            decode_entities(raw)
        } else {
            raw.to_string()
        };
        doc.set_attribute(el, name, value.as_str());
    }
}

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
        doc.parse(r#"<?xml version="1.0"?><!-- hi --><root/>"#)
            .unwrap();
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
