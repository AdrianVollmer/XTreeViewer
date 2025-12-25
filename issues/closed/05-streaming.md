# Streaming Support for Large LDIF Files

Add streaming reader support for very large files (\>100MB) to enable
XTV to handle massive data files without loading everything into memory.
Implement streaming parsers that can progressively read and parse file
content on-demand. This may require building an index first for quick
random access to different parts of the tree structure. The streaming
implementation should detect when a file exceeds the size threshold
(configurable, default ~100MB) and automatically switch to streaming
mode. Display a progress indicator during index building. The user
experience should remain smooth with lazy evaluation of tree nodes as
they're navigated.

Let's do this first for LDIF. In LDIF, it is very common that nodes are
much larger than an index entry, so it makes most sense. The index
should be in memory, not on disk.

Idea: the index stores the offset of a node, an ID (can be sequential)
and a list of child nodes. Lookup should be fast. When accessing
attributes of a node lazily, the end of the node in LDIF can easily be
determined by an empty line.

In the UI, the path of a node is displayed. To determine parent nodes,
use the information of the tree view widget, not the index, because it
should be faster.
