# Streaming Support for Large Files

Add streaming reader support for very large files (\>100MB) to enable
XTV to handle massive data files without loading everything into memory.
Implement streaming parsers that can progressively read and parse file
content on-demand. This may require building an index file first for
quick random access to different parts of the tree structure. The
streaming implementation should detect when a file exceeds the size
threshold (configurable, default ~100MB) and automatically switch to
streaming mode. Display a progress indicator during index building. The
user experience should remain smooth with lazy evaluation of tree nodes
as they're navigated.

Let's do this first for XML and LDIF. Try to use a SAX parser crate for
the XML case, and use our own implementation of a LDIF reader.

The code handeling the index file should be decoupled from the rest of
the code base, as we may experiment with different approaches.

Idea: the index stores positions of the tags (opening and ending) and
must itself be a tree structure. Work with recursion until the bytes
between opening tag and the ending tag is lower than the configured size
limit.
