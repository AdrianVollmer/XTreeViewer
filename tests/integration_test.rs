use std::fs;
use std::path::PathBuf;
use xtv::parser;

#[test]
fn test_parse_sample_json() {
    let path = PathBuf::from("examples/sample.json");
    let content = fs::read_to_string(&path).expect("Failed to read sample.json");

    let parser = parser::detect_parser(&path).expect("Failed to detect parser");
    let tree = parser.parse(&content).expect("Failed to parse JSON");

    // Verify tree structure
    assert!(tree.node_count() > 0);
    let root = tree.get_node(tree.root_id()).unwrap();
    assert_eq!(root.label, "root");
    assert!(root.has_children());
}

#[test]
fn test_parse_sample_xml() {
    let path = PathBuf::from("examples/sample.xml");
    let content = fs::read_to_string(&path).expect("Failed to read sample.xml");

    let parser = parser::detect_parser(&path).expect("Failed to detect parser");
    let tree = parser.parse(&content).expect("Failed to parse XML");

    // Verify tree structure
    assert!(tree.node_count() > 0);
    let root = tree.get_node(tree.root_id()).unwrap();
    assert_eq!(root.label, "root");
    assert!(root.has_children());
}

#[test]
fn test_unsupported_format() {
    let path = PathBuf::from("test.unsupported");
    let result = parser::detect_parser(&path);

    assert!(result.is_err());
}
