# Attribtues

In XML, a node can have attributes. The way they are displayed currently
is confusing:

    ┌Tree View─────────────────────────────────────────┐┌Node Details────────────────────┐
    │>> ▼ root [root]                                  ││Label: root                     │
    │     ▼ project [element] = XTV                    ││Type: root                      │
    │       ▼ metadata [element]                       ││Children: 1                     │
    │         ▼ author [element]                       ││                                │
    │             text [text] = XTV Contributors       ││(No attributes)                 │
    │         ▶ created [element]                      ││                                │
    │         ▶ description [element]                  ││                                │
    │       ▶ features [element]                       ││                                │
    │       ▶ configuration [element]                  ││                                │
    │       ▶ tags [element]                           ││                                │
    │                                                  ││                                │
    │                                                  ││                                │
    ...

The user sees the value of the first attribute without context after an
equal sign.

Instead, the attributes should be contained in a special "virtual" node
before all other child nodes. When expanding that node, the user should
see the list of attributes. These virtual nodes should have a different
color and perhaps a different symbol, like maybe a hollow triangle
instead of a solid triangle.

In XML, this special node will just be a dictionary, mapping keys to
primitive values. In LDIF, we may have lists of primitive values instead
of just primitive values. Keep this in mind for later.
