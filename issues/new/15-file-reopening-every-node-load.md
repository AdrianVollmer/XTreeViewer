# File Reopening on Every Node Load

**Severity**: MEDIUM (Performance)
**Category**: Performance / I/O
**Location**: `src/tree/streaming.rs:171-173`

## Problem

StreamingTree opens a new file handle for every single node loaded:

```rust
fn load_node(&self, id: usize) -> Option<TreeNode> {
    let entry = self.index.get_entry(id)?;
    let offset = entry.offset;

    // Opens file every time!
    let file = File::open(&self.file_path).ok()?;
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(offset)).ok()?;
    // ...
}
```

## Impact

- **Performance**: File opening involves kernel syscalls with significant overhead
- **Scalability**: Navigating through tree causes many file open/close operations
- **Resource usage**: Excessive file descriptor churn
- **Latency**: Noticeable delay when expanding nodes or navigating

## Benchmarking

Typical file open overhead:
- Local filesystem: ~10-50 microseconds
- Network filesystem (NFS): ~1-10 milliseconds
- For 100 node navigations: 0.1-1 second of overhead

## Recommendation

**Keep a persistent file handle or pool of readers**:

### Option 1: Single RefCell BufReader
```rust
pub struct StreamingTree {
    file_path: PathBuf,
    index: LdifIndex,
    cache: std::cell::RefCell<LruCache<usize, TreeNode>>,
    reader: std::cell::RefCell<BufReader<File>>,  // Add this
}

fn load_node(&self, id: usize) -> Option<TreeNode> {
    let entry = self.index.get_entry(id)?;
    let offset = entry.offset;

    let mut reader = self.reader.borrow_mut();
    reader.seek(SeekFrom::Start(offset)).ok()?;
    // ... read from existing reader
}
```

### Option 2: File Handle Pool
For concurrent access (if adding threading):
```rust
pub struct StreamingTree {
    file_path: PathBuf,
    index: LdifIndex,
    cache: std::cell::RefCell<LruCache<usize, TreeNode>>,
    reader_pool: Vec<RefCell<BufReader<File>>>,  // Pool of readers
}
```

This eliminates file opening overhead entirely for subsequent reads.
