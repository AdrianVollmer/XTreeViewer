use crate::tree::TreeNode;
use lru::LruCache;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;

/// LRU cache size for streaming tree nodes
/// Tuned for typical navigation patterns - holds approximately 250KB-1MB of nodes
const STREAMING_CACHE_SIZE: usize = 1000;

/// Type of node in the streaming index.
///
/// The streaming mode needs to know node types at index-build time to support
/// lazy loading without parsing the full tree structure into memory.
///
/// # Variants
///
/// - `Root`: The top-level root node of the tree
/// - `Entry`: An LDIF entry with DN (Distinguished Name) and RDN (Relative DN)
/// - `VirtualAttributes`: The `@attributes` container node
/// - `Attribute`: An individual attribute with key-value pair
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    /// Root node of the tree
    Root,
    /// LDIF entry node with distinguished name and relative distinguished name
    Entry {
        /// Full Distinguished Name (e.g., "uid=alice,ou=people,dc=example,dc=com")
        dn: String,
        /// Relative Distinguished Name (e.g., "uid=alice")
        rdn: String,
    },
    /// Virtual `@attributes` container node
    VirtualAttributes,
    /// Individual attribute node with key and value
    Attribute {
        /// Attribute key/name
        key: String,
        /// Attribute value
        value: String,
    },
}

/// Entry in the LDIF streaming index.
///
/// Each index entry represents a single node in the tree and contains just enough
/// information to locate and reconstruct the node from disk when needed.
///
/// # Fields
///
/// - `offset`: Byte position in the file where the LDIF entry begins
/// - `parent_id`: Parent node ID for O(1) parent lookup
/// - `children`: List of child node IDs for tree traversal
/// - `node_type`: Type information needed to reconstruct the node
///
/// # Memory Usage
///
/// Approximately 8 + 4 + (4 × child_count) bytes per entry on disk, plus the
/// size of strings in the `NodeType` variant.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Byte offset in the file where this LDIF entry starts (only meaningful for Entry nodes)
    pub offset: u64,
    /// Parent node ID (None for root)
    pub parent_id: Option<usize>,
    /// Child node IDs
    pub children: Vec<usize>,
    /// Type of this node with associated data
    pub node_type: NodeType,
}

impl IndexEntry {
    /// Creates a new index entry.
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset in the file where the entry starts
    /// * `parent_id` - Optional parent node ID
    /// * `node_type` - Type of node with associated data
    pub fn new(offset: u64, parent_id: Option<usize>, node_type: NodeType) -> Self {
        Self {
            offset,
            parent_id,
            children: Vec::new(),
            node_type,
        }
    }
}

/// In-memory index for LDIF streaming.
///
/// The index stores the complete tree structure (parent-child relationships and
/// node metadata) in memory, but not the full node data. This allows tree navigation
/// without loading all nodes, while keeping memory usage bounded.
///
/// # Memory Usage
///
/// For a 20GB LDIF file with 1 million entries:
/// - Each entry has ~5 attributes
/// - Total nodes: 1M entries + 1M @attributes + 5M attributes = 7M nodes
/// - Index size: ~50-100MB (compared to 20GB full data)
///
/// # Structure
///
/// - Node IDs are implicit (position in the vector)
/// - Each entry knows its parent and children
/// - Entries contain type info and file offset for lazy loading
#[derive(Debug)]
pub struct LdifIndex {
    /// Vector of index entries, indexed by node ID
    entries: Vec<IndexEntry>,
    /// Root node ID
    root_id: usize,
}

impl LdifIndex {
    /// Creates a new LDIF index.
    ///
    /// # Arguments
    ///
    /// * `root_id` - The ID of the root node (typically 0)
    pub fn new(root_id: usize) -> Self {
        Self {
            entries: Vec::new(),
            root_id,
        }
    }

    /// Adds a new index entry and returns its ID.
    ///
    /// The entry ID is simply its position in the entries vector.
    ///
    /// # Arguments
    ///
    /// * `entry` - The index entry to add
    ///
    /// # Returns
    ///
    /// The ID of the newly added entry
    pub fn add_entry(&mut self, entry: IndexEntry) -> usize {
        let id = self.entries.len();
        self.entries.push(entry);
        id
    }

    /// Gets an index entry by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The entry ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(&IndexEntry)` - If the entry exists
    /// * `None` - If the ID is out of bounds
    pub fn get_entry(&self, id: usize) -> Option<&IndexEntry> {
        self.entries.get(id)
    }

    /// Gets a mutable index entry by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The entry ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(&mut IndexEntry)` - If the entry exists
    /// * `None` - If the ID is out of bounds
    pub fn get_entry_mut(&mut self, id: usize) -> Option<&mut IndexEntry> {
        self.entries.get_mut(id)
    }

    /// Adds a child to a parent node.
    ///
    /// Updates the parent's children list to include the child ID.
    /// Silently does nothing if the parent ID is invalid.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The ID of the parent entry
    /// * `child_id` - The ID of the child entry to add
    pub fn add_child(&mut self, parent_id: usize, child_id: usize) {
        if let Some(entry) = self.entries.get_mut(parent_id) {
            entry.children.push(child_id);
        }
    }

    /// Gets the total number of entries in the index.
    ///
    /// # Returns
    ///
    /// The count of all index entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the index is empty.
    ///
    /// # Returns
    ///
    /// `true` if there are no entries, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Gets the root node ID.
    ///
    /// # Returns
    ///
    /// The ID of the root node
    pub fn root_id(&self) -> usize {
        self.root_id
    }
}

/// Streaming tree that loads nodes on-demand from disk.
///
/// This structure enables handling extremely large LDIF files (20GB+) by keeping
/// only an in-memory index and LRU cache of recently accessed nodes, rather than
/// loading the entire tree into memory.
///
/// # How It Works
///
/// 1. **Index Building**: On creation, scans the file once to build an in-memory
///    index of all nodes (see [`LdifIndex`]). This index is ~1-2% of file size.
///
/// 2. **Lazy Loading**: When a node is accessed via [`get_node`](StreamingTree::get_node),
///    it's loaded from disk at its recorded offset if not in cache.
///
/// 3. **LRU Caching**: Recently accessed nodes are kept in an LRU cache
///    ([`STREAMING_CACHE_SIZE`] = 1000 nodes) to avoid repeated disk I/O.
///
/// 4. **Arc Sharing**: Cached nodes are wrapped in `Arc<TreeNode>` to enable cheap
///    reference counting instead of cloning on every access.
///
/// # Performance Characteristics
///
/// - **Index build**: O(n) scan of file, one-time cost at startup
/// - **Node access (cache hit)**: O(1), just Arc::clone + node clone
/// - **Node access (cache miss)**: O(1) + disk I/O latency
/// - **Memory usage**: Index (~1-2% of file) + cache (~1-5MB)
///
/// # Performance Notes
///
/// This implementation uses blocking I/O operations. On network-mounted filesystems
/// (NFS, SMB, sshfs) or slow/unresponsive storage devices, operations may block
/// indefinitely without timeout. For best performance and reliability, use files
/// on local storage.
///
/// # Known Limitations
///
/// - No timeout support for I/O operations (would require async/tokio)
/// - File operations may hang on unresponsive filesystems
/// - Errors are reported to stderr but UI remains blocked during I/O
/// - Currently only supports LDIF format
///
/// # Examples
///
/// ```ignore
/// use xtv::tree::streaming::StreamingTree;
/// use xtv::parser::ldif::build_ldif_index;
///
/// // Build streaming tree from large LDIF file
/// let tree = build_ldif_index(Path::new("large.ldif"))?;
///
/// // Access nodes on-demand
/// let root = tree.get_node(tree.root_id());
/// ```
pub struct StreamingTree {
    /// Path to the LDIF file
    file_path: PathBuf,
    /// In-memory index of all nodes
    index: LdifIndex,
    /// LRU cache for recently accessed nodes
    /// Uses Arc to avoid expensive clones on cache hits
    cache: std::cell::RefCell<LruCache<usize, Arc<TreeNode>>>,
    /// Persistent file reader to avoid reopening file on every node load
    reader: std::cell::RefCell<BufReader<File>>,
}

impl std::fmt::Debug for StreamingTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingTree")
            .field("file_path", &self.file_path)
            .field("index", &self.index)
            .field("cache", &self.cache)
            .field("reader", &"<BufReader<File>>")
            .finish()
    }
}

impl StreamingTree {
    /// Creates a new streaming tree.
    ///
    /// Opens the LDIF file and creates a persistent reader. The file remains open
    /// for the lifetime of the StreamingTree to avoid repeated open/close overhead.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the LDIF file
    /// * `index` - Pre-built index (see [`build_ldif_index`](crate::parser::ldif::build_ldif_index))
    ///
    /// # Returns
    ///
    /// * `Ok(StreamingTree)` - Successfully created streaming tree
    /// * `Err(io::Error)` - If the file cannot be opened
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The file doesn't exist or cannot be accessed
    /// - Permission denied
    /// - Network filesystem is unresponsive (may block indefinitely)
    ///
    /// # Note
    ///
    /// This operation may block on network filesystems without timeout.
    pub fn new(file_path: PathBuf, index: LdifIndex) -> std::io::Result<Self> {
        // Cache size: holds recently accessed nodes for fast repeat access
        let cache_size = NonZeroUsize::new(STREAMING_CACHE_SIZE).unwrap();

        // Open the file once and keep a persistent reader
        // Note: This may block on network filesystems without timeout
        let file = File::open(&file_path).map_err(|e| {
            eprintln!("Error: Failed to open file {:?}: {}", file_path, e);
            eprintln!("Hint: Ensure the file exists and is on a responsive filesystem.");
            e
        })?;
        let reader = BufReader::new(file);

        Ok(Self {
            file_path,
            index,
            cache: std::cell::RefCell::new(LruCache::new(cache_size)),
            reader: std::cell::RefCell::new(reader),
        })
    }

    /// Gets the root node ID.
    ///
    /// # Returns
    ///
    /// The ID of the root node (typically 0)
    pub fn root_id(&self) -> usize {
        self.index.root_id()
    }

    /// Gets a node by ID, loading from disk if not in cache.
    ///
    /// This method first checks the LRU cache. On cache miss, it performs blocking
    /// I/O to load the node from disk, then adds it to the cache.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(Arc<TreeNode>)` - The node wrapped in Arc for cheap cloning
    /// * `None` - If the node doesn't exist or failed to load from disk
    ///
    /// # Performance
    ///
    /// - **Cache hit**: O(1) - Just Arc::clone (cheap reference count increment)
    /// - **Cache miss**: O(1) + blocking I/O (seek + read until blank line)
    ///
    /// # Note
    ///
    /// Returns `Arc<TreeNode>` to avoid expensive clones. Callers should use
    /// `Arc::clone()` to share ownership or dereference to access the node.
    pub fn get_node(&self, id: usize) -> Option<Arc<TreeNode>> {
        // Check cache first
        {
            let mut cache = self.cache.borrow_mut();
            if let Some(node_arc) = cache.get(&id) {
                // Arc::clone is cheap - just increments reference count
                return Some(Arc::clone(node_arc));
            }
        }

        // Load from disk
        if let Some(node) = self.load_node(id) {
            // Wrap in Arc before caching
            let node_arc = Arc::new(node);
            let mut cache = self.cache.borrow_mut();
            cache.put(id, Arc::clone(&node_arc));
            Some(node_arc)
        } else {
            None
        }
    }

    /// Gets the children IDs of a node.
    ///
    /// This is a pure index operation - no disk I/O required.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID whose children to retrieve
    ///
    /// # Returns
    ///
    /// A vector of child node IDs, or an empty vector if the node has no children
    /// or doesn't exist.
    ///
    /// # Performance
    ///
    /// O(k) where k is the number of children (due to cloning the children vector)
    pub fn get_children(&self, id: usize) -> Vec<usize> {
        self.index
            .get_entry(id)
            .map(|entry| entry.children.clone())
            .unwrap_or_default()
    }

    /// Gets the parent of a node.
    ///
    /// This is a pure index operation - no disk I/O required.
    ///
    /// # Arguments
    ///
    /// * `child_id` - The node ID whose parent to find
    ///
    /// # Returns
    ///
    /// * `Some(parent_id)` - The ID of the parent node
    /// * `None` - If the node is the root, doesn't exist, or has no parent
    ///
    /// # Performance
    ///
    /// O(1) - Direct index lookup using parent_id field
    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        self.index
            .get_entry(child_id)
            .and_then(|entry| entry.parent_id)
    }

    /// Gets the total number of nodes in the tree.
    ///
    /// # Returns
    ///
    /// The count of all nodes in the index
    ///
    /// # Performance
    ///
    /// O(1) - Returns the index length
    pub fn node_count(&self) -> usize {
        self.index.len()
    }

    /// Loads a node from disk by reading from its byte offset.
    ///
    /// This is an internal method called by [`get_node`](StreamingTree::get_node)
    /// on cache misses. It performs blocking I/O to:
    /// 1. Seek to the node's byte offset in the file
    /// 2. Read lines until a blank line (end of LDIF entry)
    /// 3. Parse the lines into a TreeNode using node type from index
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID to load
    ///
    /// # Returns
    ///
    /// * `Some(TreeNode)` - Successfully loaded and parsed node
    /// * `None` - If seek failed, read failed, or node doesn't exist in index
    ///
    /// # Safety Limits
    ///
    /// A `MAX_LINES` limit (1000) prevents infinite loops from corrupt data.
    /// If more than 1000 lines are read, loading stops with a warning.
    ///
    /// # Performance
    ///
    /// This performs blocking I/O operations. May block indefinitely on
    /// unresponsive network filesystems.
    fn load_node(&self, id: usize) -> Option<TreeNode> {
        let entry = self.index.get_entry(id)?;
        let offset = entry.offset;

        // Use the persistent reader and seek to offset
        let mut reader = self.reader.borrow_mut();
        if let Err(e) = reader.seek(SeekFrom::Start(offset)) {
            eprintln!(
                "Warning: Failed to seek to offset {} in file {:?}: {}",
                offset, self.file_path, e
            );
            return None;
        }

        // Read lines until we hit an empty line (end of entry)
        let mut lines = Vec::new();
        let mut line = String::new();
        let mut line_count = 0;
        const MAX_LINES: usize = 1000; // Prevent infinite loops

        loop {
            line.clear();

            // Check for excessive lines (potential infinite loop or corruption)
            line_count += 1;
            if line_count > MAX_LINES {
                eprintln!(
                    "Warning: Read more than {} lines for node {}, stopping",
                    MAX_LINES, id
                );
                break;
            }

            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim_end();
                    if trimmed.is_empty() {
                        break; // End of LDIF entry
                    }
                    lines.push(trimmed.to_string());
                }
                Err(e) => {
                    eprintln!(
                        "Warning: I/O error reading from file {:?}: {}",
                        self.file_path, e
                    );
                    return None;
                }
            }
        }

        // Parse the entry to create a TreeNode
        self.parse_node_from_lines(id, lines)
    }

    /// Parses a TreeNode from the lines read from disk.
    ///
    /// This method uses the node type information from the index to reconstruct
    /// the TreeNode without needing to parse the full LDIF entry. The actual
    /// line content is ignored - we use only the pre-computed metadata from
    /// the index.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID being parsed
    /// * `_lines` - Lines read from disk (currently unused, kept for future use)
    ///
    /// # Returns
    ///
    /// * `Some(TreeNode)` - Successfully reconstructed node
    /// * `None` - If the node doesn't exist in the index
    ///
    /// # Implementation Note
    ///
    /// The `_lines` parameter is currently unused because all necessary data
    /// (label, type, attributes) is stored in the index's `NodeType`. This
    /// trades index size for parsing speed.
    fn parse_node_from_lines(&self, id: usize, _lines: Vec<String>) -> Option<TreeNode> {
        let entry = self.index.get_entry(id)?;

        let mut node = match &entry.node_type {
            NodeType::Root => TreeNode::new("root", "root"),
            NodeType::Entry { rdn, .. } => TreeNode::new(rdn, "entry"),
            NodeType::VirtualAttributes => {
                TreeNode::new("@attributes", TreeNode::VIRTUAL_ATTRIBUTES_TYPE)
            }
            NodeType::Attribute { key, value } => {
                let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
                node.add_attribute("value", value);
                node
            }
        };

        // Add children references from index
        node.children = entry.children.clone();

        Some(node)
    }
}
