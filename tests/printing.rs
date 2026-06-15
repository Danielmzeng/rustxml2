use rustxml2::XmlDocument;

fn roundtrip_compact(input: &str) -> String {
    let mut doc = XmlDocument::new();
    doc.parse(input).unwrap();
    doc.print_to_string(true)
}

#[test]
fn compact_roundtrip_is_stable() {
    let xml = r#"<?xml version="1.0"?><root a="1" b="2"><child>text</child><self/><!--c--></root>"#;
    let once = roundtrip_compact(xml);
    let twice = roundtrip_compact(&once);
    assert_eq!(once, twice, "round-trip must be idempotent");
}

#[test]
fn entities_are_escaped_on_output() {
    let out = roundtrip_compact(r#"<a x="&lt;&amp;">&gt;&quot;</a>"#);
    assert_eq!(out, r#"<a x="&lt;&amp;">&gt;&quot;</a>"#);
}

#[test]
fn cdata_is_preserved() {
    let out = roundtrip_compact("<a><![CDATA[<not parsed>]]></a>");
    assert_eq!(out, "<a><![CDATA[<not parsed>]]></a>");
}

#[test]
fn pretty_print_is_indented() {
    let mut doc = XmlDocument::new();
    doc.parse("<a><b><c/></b></a>").unwrap();
    let out = doc.print_to_string(false);
    assert_eq!(out, "<a>\n    <b>\n        <c/>\n    </b>\n</a>\n");
}
