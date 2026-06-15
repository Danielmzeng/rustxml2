use rustxml2::{XmlDocument, XmlError};
use std::path::Path;

#[test]
fn load_dream_fixture() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/dream.xml");
    let mut doc = XmlDocument::new();
    doc.load_file(&path).unwrap();
    // dream.xml is <?xml?> then <!DOCTYPE PLAY ...> then <PLAY>. The first element is PLAY.
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
