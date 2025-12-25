# Memory Exhaustion in LDIF Index Building

**Severity**: CRITICAL (Security)
**Category**: Security / Resource Limits
**Location**: `src/parser/ldif.rs:355-546`

## Problem

The `build_ldif_index()` function builds an in-memory index containing ALL attribute key-value pairs for every LDIF entry (lines 454-535):

```rust
// Parse attributes from this entry
let mut attr_map: HashMap<String, Vec<String>> = HashMap::new();
attr_map.insert("dn".to_string(), vec![dn]);

// ... reads ALL attributes into memory for EVERY entry
for key in sorted_keys {
    let values = &attr_map[key];
    // Creates index nodes for all attributes
}
```

## Impact

- **Security**: DoS attack vector - maliciously crafted LDIF files with extremely long attribute values or many multi-valued attributes can cause unbounded memory allocations
- **Stability**: Out-of-memory crashes on legitimate large files
- **Resource exhaustion**: Even with streaming mode enabled, the index itself can consume excessive memory

## Recommendation

Add resource limits:

1. **Attribute size limits**:
   - Maximum size per attribute value (e.g., 1MB)
   - Truncate or skip extremely large values in index

2. **Attribute count limits**:
   - Maximum number of attributes per entry (e.g., 100)
   - Maximum number of values for multi-valued attributes (e.g., 1000)

3. **Total index size limits**:
   - Track total memory consumed during indexing
   - Abort if index exceeds reasonable size (e.g., 100MB)

4. **Consider alternative index format**:
   - Store only offsets and basic metadata in index
   - Parse attributes on-demand when nodes are accessed
