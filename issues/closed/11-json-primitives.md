# JSON primitives

When viewing a JSON file, primitives, i.e. properties whose values are neither
an array nor an object, should display their value just like attributes in the
HTML or XML case. So in the tree view it should be `<property>: <value>`, and in
the node detail view the value should be displayed without header or
indentation, just like a text node or comment node in HTML.
