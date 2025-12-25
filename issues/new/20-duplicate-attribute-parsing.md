# Duplicate Attribute Parsing Code

**Severity**: LOW (DRY Violation)
**Category**: Code Quality / Maintainability
**Locations**: `src/parser/ldif.rs:155-179` and `src/parser/ldif.rs:550-596`

## Problem

Nearly identical code exists for parsing LDIF attribute lines in two places:

### Location 1: Method in LdifFileParser (lines 155-179)
```rust
fn parse_attribute_line(&self, line: &str) -> Result<(String, String)> {
    // Handle three separators: :, ::, :<
    if let Some(pos) = line.find("::") {
        // Base64 encoded
        let key = line[..pos].trim();
        let encoded = line[pos + 2..].trim();
        let decoded = self.decode_base64(encoded)?;
        Ok((key.to_string(), decoded))
    } else if let Some(pos) = line.find(":<") {
        // URL reference
        let key = line[..pos].trim();
        let url = line[pos + 2..].trim();
        Ok((key.to_string(), format!("<URL reference: {}>", url)))
    } else if let Some(pos) = line.find(':') {
        // Plain value
        let key = line[..pos].trim();
        let value = line[pos + 1..].trim();
        Ok((key.to_string(), value.to_string()))
    } else {
        Err(XtvError::LdifParse { ... })
    }
}
```

### Location 2: Free function (lines 550-596)
```rust
fn parse_attribute_line(line: &str) -> Result<(String, String)> {
    // Handle three separators: :, ::, :<
    if let Some(pos) = line.find("::") {
        // Base64 encoded
        let key = line[..pos].trim();
        let encoded = line[pos + 2..].trim();
        match general_purpose::STANDARD.decode(encoded) {
            Ok(bytes) => match String::from_utf8(bytes.clone()) {
                Ok(s) => Ok((key.to_string(), s)),
                Err(_) => {
                    if bytes.len() <= 64 {
                        Ok((key.to_string(), format!("<binary: {}>", hex_preview(&bytes))))
                    } else {
                        Ok((key.to_string(), format!("<binary data, {} bytes>", bytes.len())))
                    }
                }
            },
            Err(e) => Err(XtvError::LdifParse { ... }),
        }
    } else if let Some(pos) = line.find(":<") {
        // URL reference - identical code
        // ...
    } else if let Some(pos) = line.find(':') {
        // Plain value - identical code
        // ...
    }
}
```

## Impact

- **Maintenance burden**: Bug fixes must be applied in two places
- **Code bloat**: ~80 lines of duplicated logic
- **Inconsistency risk**: The implementations already differ slightly in Base64 error handling
- **Testing**: Need duplicate tests for same functionality

## Differences

The main difference is base64 decoding:
- Method version calls `self.decode_base64()` which returns `Result<String>`
- Free function version has inline base64 decoding with `general_purpose::STANDARD.decode()`

## Recommendation

**Consolidate into single implementation**:

1. Keep the free function `parse_attribute_line()` (it's more complete)
2. Make the method version call the free function:

```rust
impl<'a> LdifFileParser<'a> {
    fn parse_attribute_line(&self, line: &str) -> Result<(String, String)> {
        // Just delegate to the module-level function
        parse_attribute_line(line)
    }
}
```

3. Remove the now-unused `decode_base64()` method

Alternatively, if the method is only called once, inline it at the call site and use the free function directly.

This eliminates ~40 lines of duplicate code and ensures consistency.
