use crate::tree::TreeNode;
use lru::LruCache;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// LRU cache size for streaming tree nodes
/// Tuned for typical navigation patterns - holds approximately 250KB-1MB of nodes
const STREAMING_CACHE_SIZE: usize = 1000;

/// Type of node in the index
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Root,
    Entry { dn: String, rdn: String },
    VirtualAttributes,
    Attribute { key: String, value: String },
}

/// Entry in the LDIF index
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Byte offset in the file where this LDIF entry starts (only for Entry nodes)
    pub offset: u64,
    /// Parent node ID (None for root)
    pub parent_id: Option<usize>,
    /// Child node IDs
    pub children: Vec<usize>,
    /// Type of this node
    pub node_type: NodeType,
}

impl IndexEntry {
    pub fn new(offset: u64, parent_id: Option<usize>, node_type: NodeType) -> Self {
        Self {
            offset,
            parent_id,
            children: Vec::new(),
            node_type,
        }
    }
}

/// In-memory index for LDIF streaming
#[derive(Debug)]
pub struct LdifIndex {
    /// Vector of index entries, indexed by node ID
    entries: Vec<IndexEntry>,
    /// Root node ID
    root_id: usize,
}

impl LdifIndex {
    pub fn new(root_id: usize) -> Self {
        Self {
            entries: Vec::new(),
            root_id,
        }
    }

    /// Add a new index entry and return its ID
    pub fn add_entry(&mut self, entry: IndexEntry) -> usize {
        let id = self.entries.len();
        self.entries.push(entry);
        id
    }

    /// Get an index entry by ID
    pub fn get_entry(&self, id: usize) -> Option<&IndexEntry> {
        self.entries.get(id)
    }

    /// Get a mutable index entry by ID
    pub fn get_entry_mut(&mut self, id: usize) -> Option<&mut IndexEntry> {
        self.entries.get_mut(id)
    }

    /// Add a child to a parent node
    pub fn add_child(&mut self, parent_id: usize, child_id: usize) {
        if let Some(entry) = self.entries.get_mut(parent_id) {
            entry.children.push(child_id);
        }
    }

    /// Get the total number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn root_id(&self) -> usize {
        self.root_id
    }
}

/// Streaming tree that loads nodes on-demand from disk
///
/// # Performance Note
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
pub struct StreamingTree {
    /// Path to the LDIF file
    file_path: PathBuf,
    /// In-memory index
    index: LdifIndex,
    /// LRU cache for recently accessed nodes
    cache: std::cell::RefCell<LruCache<usize, TreeNode>>,
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
    /// Create a new streaming tree
    ///
    /// Opens the LDIF file and creates a persistent reader. This operation may block
    /// on network filesystems. Returns an error if the file cannot be opened.
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

    /// Get the root node ID
    pub fn root_id(&self) -> usize {
        self.index.root_id()
    }

    /// Get a node by ID (loads from disk if not in cache)
    pub fn get_node(&self, id: usize) -> Option<TreeNode> {
        // Check cache first
        {
            let mut cache = self.cache.borrow_mut();
            if let Some(node) = cache.get(&id) {
                return Some(node.clone());
            }
        }

        // Load from disk
        if let Some(node) = self.load_node(id) {
            // Store in cache
            let mut cache = self.cache.borrow_mut();
            cache.put(id, node.clone());
            Some(node)
        } else {
            None
        }
    }

    /// Get the children IDs of a node
    pub fn get_children(&self, id: usize) -> Vec<usize> {
        self.index
            .get_entry(id)
            .map(|entry| entry.children.clone())
            .unwrap_or_default()
    }

    /// Get the parent of a node
    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        self.index
            .get_entry(child_id)
            .and_then(|entry| entry.parent_id)
    }

    /// Get the total number of nodes
    pub fn node_count(&self) -> usize {
        self.index.len()
    }

    /// Load a node from disk by reading from its offset
    ///
    /// This performs blocking I/O operations. Returns None if the operation fails.
    /// A MAX_LINES limit prevents infinite loops from corrupt data.
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

    /// Parse a TreeNode from the lines read from disk
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
