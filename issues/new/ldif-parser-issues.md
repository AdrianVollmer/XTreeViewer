- There are several `line: 0` statements in `ldif.rs` in errors. Try to report
  the actual line number in case of errors.
- The `dn` attribute can also be base64 encoded, which is currently not handled.
- The separator detection on line 606 seems unreliable: `if let Some(pos) = line.find("::") {`
  What if the attribute contains a `::`?
