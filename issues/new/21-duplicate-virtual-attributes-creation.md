# Duplicate Virtual Attributes Node Creation

**Severity**: LOW (DRY Violation)
**Category**: Code Quality / Maintainability
**Location**: `src/parser/xml.rs:41-46` and `97-102`

## Problem

Identical code for inserting virtual attributes nodes appears twice in the XML parser:

### Event::Start handler (lines 41-46)
```rust
// Create virtual attributes node if there are attributes
if let Some(virtual_id) = create_virtual_attributes_node(&mut tree, &attributes) {
    tree.get_node_mut(node_id)
        .unwrap()
        .children
        .insert(0, virtual_id);
}
```

### Event::Empty handler (lines 97-102)
```rust
// Create virtual attributes node if there are attributes
if let Some(virtual_id) = create_virtual_attributes_node(&mut tree, &attributes) {
    tree.get_node_mut(node_id)
        .unwrap()
        .children
        .insert(0, virtual_id);
}
```

Exact same 5 lines of code in both places.

## Impact

- **Maintenance**: Changes must be made in two places
- **Code bloat**: Unnecessary duplication
- **Readability**: Adds visual clutter
- **Error prone**: Easy to update one but forget the other

## Context

This duplication exists because XML has two types of elements:
- `Event::Start` - Opening tag that may have children: `<element attr="value">...</element>`
- `Event::Empty` - Self-closing tag: `<element attr="value" />`

Both need the same attribute handling logic.

## Recommendation

**Extract into helper function**:

```rust
/// Add virtual attributes node to an element node if it has attributes
fn add_virtual_attributes_if_present(
    tree: &mut Tree,
    node_id: usize,
    attributes: &[crate::tree::node::Attribute],
) {
    if let Some(virtual_id) = create_virtual_attributes_node(tree, attributes) {
        tree.get_node_mut(node_id)
            .unwrap()
            .children
            .insert(0, virtual_id);
    }
}
```

Then use it in both places:
```rust
Ok(Event::Start(e)) => {
    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
    let mut node = TreeNode::new(name, "element");

    // Add XML attributes
    for attr in e.attributes() {
        let attr = attr.map_err(|e| XtvError::XmlParse(e.to_string()))?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let value = String::from_utf8_lossy(&attr.value).to_string();
        node.add_attribute(key, value);
    }

    let attributes = node.attributes.clone();
    let node_id = tree.add_node(node);

    // Use extracted helper
    add_virtual_attributes_if_present(&mut tree, node_id, &attributes);

    // ...
}
```

This eliminates the duplication and makes the intent clearer.
