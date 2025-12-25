# No File Operation Timeouts

**Severity**: MEDIUM (Security)
**Category**: Security / Reliability
**Location**: `src/tree/streaming.rs:171`

## Problem

File operations in streaming mode have no timeout and could hang indefinitely:

```rust
fn load_node(&self, id: usize) -> Option<TreeNode> {
    let entry = self.index.get_entry(id)?;
    let offset = entry.offset;

    // Opens file with no timeout
    let file = File::open(&self.file_path).ok()?;
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(offset)).ok()?;
    // ...
}
```

## Impact

- **Availability**: Application can hang if file is on slow/unresponsive filesystem (NFS, network mount, FUSE filesystem)
- **User Experience**: UI becomes completely unresponsive with no feedback
- **Resource leaks**: Hung file operations may hold locks or file descriptors

## Scenarios

- Network-mounted files (NFS, SMB, sshfs)
- Files on failing/slow storage devices
- Files accessed through FUSE filesystems
- Concurrent access conflicts with file locks

## Recommendation

1. **Add timeout wrappers** for file operations:
   - Use platform-specific timeout mechanisms
   - Consider using `tokio` or async I/O with timeouts

2. **Implement background loading**:
   - Load nodes in background thread with timeout
   - Show loading indicator in UI
   - Allow user to cancel hung operations

3. **Add error handling**:
   - Detect and report slow file operations
   - Provide user feedback when operations are taking too long
   - Allow graceful degradation (e.g., show error node instead of crashing)
