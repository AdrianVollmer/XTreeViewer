# Magic Numbers Should Be Constants

**Severity**: LOW (Maintainability)
**Category**: Code Quality / Readability
**Locations**: Multiple

## Problem

Magic numbers are scattered throughout the code without clear meaning:

### Examples

1. **Cache size** - `src/tree/streaming.rs:111`
   ```rust
   let cache_size = NonZeroUsize::new(1000).unwrap();
   ```

2. **Page navigation amount** - `src/ui/app.rs:358, 362, 376, 380`
   ```rust
   for _ in 0..10 {
       self.tree_view.navigate_up();
   }
   ```

3. **Popup dimensions** - `src/ui/app.rs:621-622, 720-721`
   ```rust
   let popup_width = 80.min(area.width - 4);
   let popup_height = 30.min(area.height - 4);
   // ...
   let popup_width = (area.width * 4 / 5).min(100);
   ```

4. **Binary preview limit** - `src/parser/ldif.rs:192, 562`
   ```rust
   if bytes.len() <= 64 {
       Ok(format!("<binary: {}>", hex_preview(&bytes)))
   }
   ```

5. **Hex preview length** - `src/parser/ldif.rs:348`
   ```rust
   .take(32)
   ```

## Impact

- **Clarity**: Reader must infer meaning from context
- **Maintenance**: Changes require finding all occurrences
- **Tuning**: Difficult to adjust values experimentally
- **Documentation**: No place to explain why these values were chosen

## Recommendation

Define module-level or struct-level constants with descriptive names:

### For streaming.rs:
```rust
/// LRU cache size for streaming tree nodes
/// Tuned for typical navigation patterns - holds ~250KB-1MB of nodes
const STREAMING_CACHE_SIZE: usize = 1000;

impl StreamingTree {
    pub fn new(file_path: PathBuf, index: LdifIndex) -> Self {
        let cache_size = NonZeroUsize::new(STREAMING_CACHE_SIZE).unwrap();
        // ...
    }
}
```

### For app.rs:
```rust
/// Number of lines to scroll for page up/down operations
const PAGE_SCROLL_LINES: usize = 10;

/// Help popup dimensions
const HELP_POPUP_WIDTH: u16 = 80;
const HELP_POPUP_HEIGHT: u16 = 30;

/// Print popup dimensions (as fraction of screen)
const PRINT_POPUP_WIDTH_FRACTION: u16 = 4;  // 4/5 of screen
const PRINT_POPUP_WIDTH_DIVISOR: u16 = 5;
const PRINT_POPUP_MAX_WIDTH: u16 = 100;
const PRINT_POPUP_HEIGHT_FRACTION: u16 = 3;  // 3/4 of screen
const PRINT_POPUP_HEIGHT_DIVISOR: u16 = 4;
const PRINT_POPUP_MAX_HEIGHT: u16 = 30;
```

### For ldif.rs:
```rust
/// Maximum bytes to show in binary data hex preview
const BINARY_INLINE_PREVIEW_LIMIT: usize = 64;

/// Number of bytes to show in hex preview
const HEX_PREVIEW_BYTES: usize = 32;
```

This makes the code self-documenting and easier to tune.
