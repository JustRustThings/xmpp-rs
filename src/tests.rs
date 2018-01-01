use std::iter::Iterator;

use quick_xml::reader::Reader;

use element::Element;

const TEST_STRING: &'static str = r#"<?xml version="1.0" encoding="utf-8"?><root xmlns="root_ns" a="b" xml:lang="en">meow<child c="d" /><child xmlns="child_ns" d="e" xml:lang="fr" />nya</root>"#;

fn build_test_tree() -> Element {
    let mut root = Element::builder("root")
                           .ns("root_ns")
                           .attr("xml:lang", "en")
                           .attr("a", "b")
                           .build();
    root.append_text_node("meow");
    let child = Element::builder("child")
                        .attr("c", "d")
                        .build();
    root.append_child(child);
    let other_child = Element::builder("child")
                              .ns("child_ns")
                              .attr("d", "e")
                              .attr("xml:lang", "fr")
                              .build();
    root.append_child(other_child);
    root.append_text_node("nya");
    root
}

#[test]
fn reader_works() {
    let mut reader = Reader::from_str(TEST_STRING);
    assert_eq!(Element::from_reader(&mut reader).unwrap(), build_test_tree());
}

#[test]
fn writer_works() {
    let root = build_test_tree();
    let mut writer = Vec::new();
    {
        root.write_to(&mut writer).unwrap();
    }
    assert_eq!(String::from_utf8(writer).unwrap(), TEST_STRING);
}

#[test]
fn writer_escapes_attributes() {
    let root = Element::builder("root")
        .attr("a", "\"Air\" quotes")
        .build();
    let mut writer = Vec::new();
    {
        root.write_to(&mut writer).unwrap();
    }
    assert_eq!(String::from_utf8(writer).unwrap(),
               r#"<?xml version="1.0" encoding="utf-8"?><root a="&quot;Air&quot; quotes" />"#
    );
}

#[test]
fn writer_escapes_text() {
    let root = Element::builder("root")
        .append("<3")
        .build();
    let mut writer = Vec::new();
    {
        root.write_to(&mut writer).unwrap();
    }
    assert_eq!(String::from_utf8(writer).unwrap(),
               r#"<?xml version="1.0" encoding="utf-8"?><root>&lt;3</root>"#
    );
}

#[test]
fn builder_works() {
    let elem = Element::builder("a")
                       .ns("b")
                       .attr("c", "d")
                       .append(Element::builder("child"))
                       .append("e")
                       .build();
    assert_eq!(elem.name(), "a");
    assert_eq!(elem.ns(), Some("b".to_owned()));
    assert_eq!(elem.attr("c"), Some("d"));
    assert_eq!(elem.attr("x"), None);
    assert_eq!(elem.text(), "e");
    assert!(elem.has_child("child", "b"));
    assert!(elem.is("a", "b"));
}

#[test]
fn children_iter_works() {
    let root = build_test_tree();
    let mut iter = root.children();
    assert!(iter.next().unwrap().is("child", "root_ns"));
    assert!(iter.next().unwrap().is("child", "child_ns"));
    assert_eq!(iter.next(), None);
}

#[test]
fn get_child_works() {
    let root = build_test_tree();
    assert_eq!(root.get_child("child", "inexistent_ns"), None);
    assert_eq!(root.get_child("not_a_child", "root_ns"), None);
    assert!(root.get_child("child", "root_ns").unwrap().is("child", "root_ns"));
    assert!(root.get_child("child", "child_ns").unwrap().is("child", "child_ns"));
    assert_eq!(root.get_child("child", "root_ns").unwrap().attr("c"), Some("d"));
    assert_eq!(root.get_child("child", "child_ns").unwrap().attr("d"), Some("e"));
}

#[test]
fn namespace_propagation_works() {
    let mut root = Element::builder("root").ns("root_ns").build();
    let mut child = Element::bare("child");
    let grandchild = Element::bare("grandchild");
    child.append_child(grandchild);
    root.append_child(child);

    assert_eq!(root.get_child("child", "root_ns").unwrap().ns(), root.ns());
    assert_eq!(root.get_child("child", "root_ns").unwrap()
                   .get_child("grandchild", "root_ns").unwrap()
                   .ns(), root.ns());
}

#[test]
fn two_elements_with_same_arguments_different_order_are_equal() {
    let elem1: Element = "<a b='a' c=''/>".parse().unwrap();
    let elem2: Element = "<a c='' b='a'/>".parse().unwrap();
    assert_eq!(elem1, elem2);

    let elem1: Element = "<a b='a' c=''/>".parse().unwrap();
    let elem2: Element = "<a c='d' b='a'/>".parse().unwrap();
    assert_ne!(elem1, elem2);
}

#[test]
fn namespace_attributes_works() {
    let mut reader = Reader::from_str(TEST_STRING);
    let root = Element::from_reader(&mut reader).unwrap();
    assert_eq!("en", root.attr("xml:lang").unwrap());
    assert_eq!("fr", root.get_child("child", "child_ns").unwrap().attr("xml:lang").unwrap());
}

#[test]
fn wrongly_closed_elements_error() {
    let elem1 = "<a></b>".parse::<Element>();
    assert!(elem1.is_err());
    let elem1 = "<a></c></a>".parse::<Element>();
    assert!(elem1.is_err());
    let elem1 = "<a><c><d/></c></a>".parse::<Element>();
    assert!(elem1.is_ok());
}

#[test]
fn namespace_simple() {
    let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
    assert_eq!(elem.name(), "message");
    assert_eq!(elem.ns(), Some("jabber:client".to_owned()));
}

#[test]
fn namespace_prefixed() {
    let elem: Element = "<stream:features xmlns:stream='http://etherx.jabber.org/streams'/>"
        .parse().unwrap();
    assert_eq!(elem.name(), "features");
    assert_eq!(elem.ns(), Some("http://etherx.jabber.org/streams".to_owned()));
}

#[test]
fn namespace_inherited_simple() {
    let elem: Element = "<stream xmlns='jabber:client'><message/></stream>".parse().unwrap();
    assert_eq!(elem.name(), "stream");
    assert_eq!(elem.ns(), Some("jabber:client".to_owned()));
    let child = elem.children().next().unwrap();
    assert_eq!(child.name(), "message");
    assert_eq!(child.ns(), Some("jabber:client".to_owned()));
}

#[test]
fn namespace_inherited_prefixed1() {
    let elem: Element = "<stream:features xmlns:stream='http://etherx.jabber.org/streams' xmlns='jabber:client'><message/></stream:features>"
        .parse().unwrap();
    assert_eq!(elem.name(), "features");
    assert_eq!(elem.ns(), Some("http://etherx.jabber.org/streams".to_owned()));
    let child = elem.children().next().unwrap();
    assert_eq!(child.name(), "message");
    assert_eq!(child.ns(), Some("jabber:client".to_owned()));
}

#[test]
fn namespace_inherited_prefixed2() {
    let elem: Element = "<stream xmlns='http://etherx.jabber.org/streams' xmlns:jabber='jabber:client'><jabber:message/></stream>"
        .parse().unwrap();
    assert_eq!(elem.name(), "stream");
    assert_eq!(elem.ns(), Some("http://etherx.jabber.org/streams".to_owned()));
    let child = elem.children().next().unwrap();
    assert_eq!(child.name(), "message");
    assert_eq!(child.ns(), Some("jabber:client".to_owned()));
}