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
