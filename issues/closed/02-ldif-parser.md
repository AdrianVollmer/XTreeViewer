# LDIF Parser

Implement a custom parser for LDIF (LDAP Data Interchange Format) files
to enable viewing LDAP directory entries as tree structures. LDIF files
can be very large (up to 20GB), so the parser should support efficient
reading and potentially build an index for quick navigation. Each LDIF
entry should be represented as a tree node with the DN (Distinguished
Name) as the label. Attributes in LDIF can have multiple values, which
should be stored as a list in the node's attribute collection. Follow
the existing parser pattern and add comprehensive tests with sample LDIF
files.

If there is no good LDIF parser crate, consider implementing it from scratch. It
shouldn't be too complex. The definition is in RFC 2849.
