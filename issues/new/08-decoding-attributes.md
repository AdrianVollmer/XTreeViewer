# Decoding attributes

Attributes might come in encoded form. Especially LDIF will have binary
values that will be encoded in base64 or something else. We need to
offer an option to decode a value.

By default, the UI should simply show the ASCII representation, but when
pressing a hotkey (like `d`) the user should be presented with a list of
options (base64 at a minimum, base64+hexdump, also perhaps a unix time stamp
converter),
that will convert the binary value of the ASCII reprsentation into
something human readable.
