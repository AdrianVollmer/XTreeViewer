# Navigation

Add more navigation keys. Take inspiration from jless:

    Moving

        count j, DownArrow, Ctrl-n, Enter Move focus down one line (or N lines).
        count k, UpArrow, Ctrl-p, Backspace Move focus up one line (or N lines).
        h, LeftArrow When focused on an expanded object or array, collapse the object or array. Otherwise, move focus to the parent of the focused node.
        H Focus the parent of the focused node without collapsing the focused node.
        l, RightArrow When focused on a collapsed object or array, expand the object or array. When focused on an expanded object or array, move focus to the first child. When focused on non-container values, does nothing.
        count J Move to the focused node's next sibling 1 or N times.
        count K Move to the focused node's previous sibling 1 or N times.
        count w Move forward until the next change in depth 1 or N times.
        count b Move backwards until the next change in depth 1 or N times.
        count Ctrl-f, PageDown Move down by one window's height or N windows' heights.
        count Ctrl-b, PageUp Move up by one window's height or N windows' heights.
        0, ^ Move to the first sibling of the focused node's parent.
        $ Move to the last sibling of the focused node's parent.
        Home Focus the first line in the input
        End Focus the last line in the input
        count g Focus the first line in the input if no count is given. If a count is given, focus that line number. If the line isn't visible, focus the last visible line before it.
        count G Focus the last line in the input if no count is given. If a count is given, focus that line number, expanding any of its parent nodes if necessary.
        c Shallow collapse the focused node and all its siblings.
        C Deep collapse the focused node and all its siblings.
        e Shallow expand the focused node and all its siblings.
        E Deep expand the focused node and all its siblings.
        Space Toggle the collapsed state of the currently focused node.

    Scrolling

        count Ctrl-e Scroll down one line (or N lines).
        count Ctrl-y Scroll up one line (or N lines).
        count Ctrl-d Scroll down by half the height of the screen (or by N lines).
        count Ctrl-u Scroll up by half the height of the screen (or by N lines). For this command and Ctrl-d, focus is also moved by the specified number of lines. If no count is specified, the number of lines to scroll by is recalled from previous executions.
        zz Move the focused node to the center of the screen.
        zt Move the focused node to the top of the screen.
        zb Move the focused node to the bottom of the screen.
        count . Scroll a truncated value one character to the right (or N characters).
        count , Scroll a truncated value one character to the left (or N characters).
        ; Scroll a truncated value all the way to the end, or, if already at the end, back to the start.
        count < Decrease the indentation of every line by one (or N) tabs.
        count > Increase the indentation of every line by one (or N) tabs.

    Copying and Printing

    You can copy various parts of the JSON file to your clipboard using one of the following y commands.

    Alternatively, you can print out values using p. This is useful for viewing long string values all at once, or if the clipboard functionality is not working; mouse-tracking will be temporarily disabled, allowing you to use your terminal's native clipboard capabilities to select and copy the desired text.

        yy, pp Copy/print the value of the currently focused node, pretty printed
        yv, pv Copy/print the value of the currently focused node in a "nicely" printed one-line format
        ys, ps When the currently focused value is a string, copy/print the contents of the string unescaped (except control characters)
        yk, pk Copy/print the key of the current key/value pair
        yp, pP Copy/print the path from the root JSON element to the currently focused node, e.g., .foo[3].bar
        yb, pb Like yp, but always uses the bracket form for object keys, e.g., ["foo"][3]["bar"], which is useful if the environment where you'll paste the path doesn't support the .key format, like in Python
        yq, pq Copy/print a jq style path that will select the currently focused node, e.g., .foo[].bar 

