use rustxml2::{XmlDocument, XmlError};

#[test]
fn typed_attribute_queries() {
    let mut doc = XmlDocument::new();
    doc.parse(r#"<e i="42" n="-7" f="3.5" b="true" big="10000000000"/>"#)
        .unwrap();
    let e = doc.root_element().unwrap();

    assert_eq!(doc.query_int_attribute(e, "i"), Ok(42));
    assert_eq!(doc.query_int_attribute(e, "n"), Ok(-7));
    assert_eq!(doc.query_double_attribute(e, "f"), Ok(3.5));
    assert_eq!(doc.query_bool_attribute(e, "b"), Ok(true));
    assert_eq!(doc.query_int64_attribute(e, "big"), Ok(10_000_000_000));

    assert_eq!(
        doc.query_int_attribute(e, "missing"),
        Err(XmlError::NoAttribute)
    );
    assert_eq!(
        doc.query_int_attribute(e, "f"),
        Err(XmlError::WrongAttributeType)
    );
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
