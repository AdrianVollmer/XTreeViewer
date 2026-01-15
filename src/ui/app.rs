use crate::error::{Result, XtvError};
use crate::tree::TreeVariant;
use crate::ui::tree_view::TreeView;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::Paragraph,
};
use std::io;

/// Maximum number of nodes to search through
/// Prevents memory exhaustion when searching large streaming trees
const MAX_SEARCH_NODES: usize = 100_000;

/// Number of lines to scroll for page up/down operations
const PAGE_SCROLL_LINES: usize = 10;

/// Help popup width
const HELP_POPUP_WIDTH: u16 = 80;

/// Help popup height
const HELP_POPUP_HEIGHT: u16 = 30;

/// Print popup width as fraction of screen width (4/5)
const PRINT_POPUP_WIDTH_FRACTION: u16 = 4;
const PRINT_POPUP_WIDTH_DIVISOR: u16 = 5;
const PRINT_POPUP_MAX_WIDTH: u16 = 100;

/// Print popup height as fraction of screen height (3/4)
const PRINT_POPUP_HEIGHT_FRACTION: u16 = 3;
const PRINT_POPUP_HEIGHT_DIVISOR: u16 = 4;
const PRINT_POPUP_MAX_HEIGHT: u16 = 30;

/// Decode menu options
#[derive(Debug, Clone, Copy, PartialEq)]
enum DecodeOption {
    Base64ToText,
    Base64ToHex,
    BinaryToHex,
    UnixTimestamp,
    RawValue,
}

impl DecodeOption {
    fn all() -> Vec<DecodeOption> {
        vec![
            DecodeOption::Base64ToText,
            DecodeOption::Base64ToHex,
            DecodeOption::BinaryToHex,
            DecodeOption::UnixTimestamp,
            DecodeOption::RawValue,
        ]
    }

    fn label(&self) -> &str {
        match self {
            DecodeOption::Base64ToText => "Decode Base64 → Text",
            DecodeOption::Base64ToHex => "Decode Base64 → Hexdump",
            DecodeOption::BinaryToHex => "View as Hexdump",
            DecodeOption::UnixTimestamp => "Parse as Unix Timestamp",
            DecodeOption::RawValue => "Show Raw Value",
        }
    }
}

pub struct App {
    tree: TreeVariant,
    tree_view: TreeView,
    should_quit: bool,
    show_help: bool,
    last_key_was_y: bool,
    last_key_was_p: bool,
    print_content: Option<String>,
    search_mode: bool,
    search_query: String,
    search_matches: Vec<usize>,
    current_match_index: Option<usize>,
    case_sensitive: bool,
    cached_path: String,
    last_selected_id: Option<usize>,
    show_decode_menu: bool,
    decode_menu_selected: usize,
}

impl App {
    pub fn new(tree: TreeVariant) -> Self {
        let tree_view = TreeView::new(tree.root_id());

        Self {
            tree,
            tree_view,
            should_quit: false,
            show_help: false,
            last_key_was_y: false,
            last_key_was_p: false,
            print_content: None,
            search_mode: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match_index: None,
            case_sensitive: false,
            cached_path: String::new(),
            last_selected_id: None,
            show_decode_menu: false,
            decode_menu_selected: 0,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode().map_err(|e| XtvError::Tui(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| XtvError::Tui(e.to_string()))?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|e| XtvError::Tui(e.to_string()))?;

        // Main loop
        let result = self.main_loop(&mut terminal);

        // Cleanup
        disable_raw_mode().map_err(|e| XtvError::Tui(e.to_string()))?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|e| XtvError::Tui(e.to_string()))?;
        terminal
            .show_cursor()
            .map_err(|e| XtvError::Tui(e.to_string()))?;

        result
    }

    fn main_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        while !self.should_quit {
            terminal
                .draw(|f| self.render(f))
                .map_err(|e| XtvError::Tui(e.to_string()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Tree view
                Constraint::Length(1), // Path bar
                Constraint::Length(1), // Footer
            ])
            .split(frame.size());

        // Render tree view (full width, no border)
        self.tree_view.render(
            frame,
            main_chunks[0],
            &self.tree,
            &self.search_matches,
            self.current_match_index,
        );

        // Update path cache only if selection changed
        let current_selected_id = self.tree_view.get_selected_node_id();
        if current_selected_id != self.last_selected_id {
            self.cached_path = self.compute_node_path();
            self.last_selected_id = current_selected_id;
        }

        // Render path bar using cached path
        let path_bar =
            Paragraph::new(self.cached_path.as_str()).style(Style::default().fg(Color::Gray));
        frame.render_widget(path_bar, main_chunks[1]);

        // Render footer or search bar
        if self.search_mode {
            let search_text = format!("Search: {}", self.search_query);
            let search_bar = Paragraph::new(search_text);
            frame.render_widget(search_bar, main_chunks[2]);
        } else if !self.search_matches.is_empty() {
            let match_info = if let Some(idx) = self.current_match_index {
                format!(
                    " Search: {} ({}/{}) | n: Next | N: Previous | /: New search | Esc: Clear ",
                    self.search_query,
                    idx + 1,
                    self.search_matches.len()
                )
            } else {
                format!(
                    " Search: {} (0/{}) ",
                    self.search_query,
                    self.search_matches.len()
                )
            };
            let status_bar = Paragraph::new(match_info);
            frame.render_widget(status_bar, main_chunks[2]);
        } else {
            let help_text =
                " ↑/↓/j/k: Move | h/l: Smart nav | Space: Toggle | /: Search | ?: Help | q: Quit ";
            let status_bar = Paragraph::new(help_text);
            frame.render_widget(status_bar, main_chunks[2]);
        }

        // Render help popup if shown
        if self.show_help {
            self.render_help_popup(frame);
        }

        // Render decode menu if shown
        if self.show_decode_menu {
            self.render_decode_menu(frame);
        }

        // Render print popup if content is set
        if self.print_content.is_some() {
            self.render_print_popup(frame);
        }
    }

    fn compute_node_path(&self) -> String {
        let selected_id = match self.tree_view.get_selected_node_id() {
            Some(id) => id,
            None => return String::new(),
        };

        // Build path from root to selected node
        let mut path_parts = Vec::new();
        let mut current_id = selected_id;

        // Walk up the tree to build the path
        loop {
            if let Some(node) = self.tree.get_node(current_id) {
                path_parts.push(node.label.clone());
            }

            // Find parent
            match self.tree.get_parent(current_id) {
                Some(parent_id) => current_id = parent_id,
                None => break,
            }
        }

        // Reverse to get root-to-leaf order
        path_parts.reverse();

        // Join with " > " separator
        format!(" {}", path_parts.join(" > "))
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))
            .map_err(|e| XtvError::Tui(e.to_string()))?
        {
            if let Event::Key(key) = event::read().map_err(|e| XtvError::Tui(e.to_string()))? {
                self.handle_key(key)?;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle modal states first
        if self.print_content.is_some() {
            return self.handle_print_popup_key();
        }

        if self.show_help {
            return self.handle_help_key(key);
        }

        if self.show_decode_menu {
            return self.handle_decode_menu_key(key);
        }

        if self.search_mode {
            return self.handle_search_input_key(key);
        }

        // Handle prefix keys
        if self.last_key_was_y {
            return self.handle_yank_command(key);
        }

        if self.last_key_was_p {
            return self.handle_print_command(key);
        }

        // Handle normal navigation/command keys
        self.handle_normal_key(key)
    }

    /// Handle key press when print popup is visible
    /// Any key closes the popup
    fn handle_print_popup_key(&mut self) -> Result<()> {
        self.print_content = None;
        Ok(())
    }

    /// Handle key press when help screen is visible
    fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                self.show_help = false;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle key press during search input mode
    fn handle_search_input_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Exit search mode
                self.search_mode = false;
                self.search_query.clear();
                self.search_matches.clear();
                self.current_match_index = None;
            }
            KeyCode::Enter => {
                // Exit search mode but keep search active
                self.search_mode = false;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.perform_search();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.perform_search();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle key press when decode menu is visible
    fn handle_decode_menu_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.show_decode_menu = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.decode_menu_selected > 0 {
                    self.decode_menu_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = DecodeOption::all().len() - 1;
                if self.decode_menu_selected < max {
                    self.decode_menu_selected += 1;
                }
            }
            KeyCode::Enter => {
                // Execute the selected decode option
                self.execute_decode_option();
                self.show_decode_menu = false;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle 'y' prefix commands (yank/copy to clipboard)
    fn handle_yank_command(&mut self, key: KeyEvent) -> Result<()> {
        self.last_key_was_y = false;
        match key.code {
            KeyCode::Char('y') => {
                if let Some(text) = self.get_node_value_pretty() {
                    let _ = self.copy_to_clipboard(&text);
                }
                return Ok(());
            }
            KeyCode::Char('v') => {
                if let Some(text) = self.get_node_value_compact() {
                    let _ = self.copy_to_clipboard(&text);
                }
                return Ok(());
            }
            KeyCode::Char('s') => {
                if let Some(text) = self.get_node_string_value() {
                    let _ = self.copy_to_clipboard(&text);
                }
                return Ok(());
            }
            KeyCode::Char('k') => {
                if let Some(text) = self.get_node_key() {
                    let _ = self.copy_to_clipboard(&text);
                }
                return Ok(());
            }
            _ => {
                // Fall through to normal handling
            }
        }
        // If we didn't handle it, process as normal key
        self.handle_normal_key(key)
    }

    /// Handle 'p' prefix commands (print to popup)
    fn handle_print_command(&mut self, key: KeyEvent) -> Result<()> {
        self.last_key_was_p = false;
        match key.code {
            KeyCode::Char('p') => {
                self.print_content = self.get_node_value_pretty();
                return Ok(());
            }
            KeyCode::Char('v') => {
                self.print_content = self.get_node_value_compact();
                return Ok(());
            }
            KeyCode::Char('s') => {
                self.print_content = self.get_node_string_value();
                return Ok(());
            }
            KeyCode::Char('k') => {
                self.print_content = self.get_node_key();
                return Ok(());
            }
            _ => {
                // Fall through to normal handling
            }
        }
        // If we didn't handle it, process as normal key
        self.handle_normal_key(key)
    }

    /// Handle normal navigation and command keys
    fn handle_normal_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Esc => {
                // Clear search if active, otherwise quit
                if !self.search_matches.is_empty() {
                    self.search_query.clear();
                    self.search_matches.clear();
                    self.current_match_index = None;
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.tree_view.navigate_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.tree_view.navigate_down(&self.tree);
            }
            KeyCode::Enter => {
                self.tree_view.toggle_expand(&self.tree);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.tree_view.smart_right(&self.tree);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.tree_view.smart_left(&self.tree);
            }
            KeyCode::Char('H') => {
                self.tree_view.navigate_to_parent(&self.tree);
            }
            KeyCode::Char(' ') => {
                self.tree_view.toggle_expand(&self.tree);
            }
            KeyCode::Char('J') => {
                self.tree_view.navigate_to_next_sibling(&self.tree);
            }
            KeyCode::Char('K') => {
                self.tree_view.navigate_to_previous_sibling(&self.tree);
            }
            KeyCode::Char('0') => {
                self.tree_view.navigate_to_first_sibling(&self.tree);
            }
            KeyCode::Char('$') => {
                self.tree_view.navigate_to_last_sibling(&self.tree);
            }
            KeyCode::Char('g') => {
                self.tree_view.navigate_to_first_line();
            }
            KeyCode::Char('G') => {
                self.tree_view.navigate_to_last_line(&self.tree);
            }
            KeyCode::Char('e') => {
                self.tree_view.expand_all_siblings(&self.tree);
            }
            KeyCode::Char('E') => {
                self.tree_view.expand_all_siblings_deep(&self.tree);
            }
            KeyCode::Char('c') => {
                self.tree_view.collapse_all_siblings(&self.tree);
            }
            KeyCode::Char('C') => {
                self.tree_view.collapse_all_siblings_deep(&self.tree);
            }
            KeyCode::PageUp | KeyCode::Char('[') => {
                for _ in 0..PAGE_SCROLL_LINES {
                    self.tree_view.navigate_up();
                }
            }
            KeyCode::PageDown | KeyCode::Char(']') => {
                for _ in 0..PAGE_SCROLL_LINES {
                    self.tree_view.navigate_down(&self.tree);
                }
            }
            KeyCode::Char('y') => {
                self.last_key_was_y = true;
                return Ok(());
            }
            KeyCode::Char('p') => {
                self.last_key_was_p = true;
                return Ok(());
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
                self.search_matches.clear();
                self.current_match_index = None;
            }
            KeyCode::Char('n') => {
                self.next_match();
            }
            KeyCode::Char('N') => {
                self.previous_match();
            }
            KeyCode::Char('d') => {
                // Only show decode menu if we're on a node with a value
                if self.get_node_string_value().is_some() {
                    self.show_decode_menu = true;
                    self.decode_menu_selected = 0;
                }
            }
            _ => {}
        }

        // Reset all prefix flags
        self.last_key_was_y = false;
        self.last_key_was_p = false;

        Ok(())
    }

    // Get the node value as pretty-printed JSON
    fn get_node_value_pretty(&self) -> Option<String> {
        let node_id = self.tree_view.get_selected_node_id()?;
        let node = self.tree.get_node(node_id)?;

        // Convert node to JSON value and pretty print
        let json_value = self.node_to_json(node_id, &node)?;
        serde_json::to_string_pretty(&json_value).ok()
    }

    // Get the node value as compact one-line JSON
    fn get_node_value_compact(&self) -> Option<String> {
        let node_id = self.tree_view.get_selected_node_id()?;
        let node = self.tree.get_node(node_id)?;

        let json_value = self.node_to_json(node_id, &node)?;
        serde_json::to_string(&json_value).ok()
    }

    // Get the string value if the node is a string
    fn get_node_string_value(&self) -> Option<String> {
        let node_id = self.tree_view.get_selected_node_id()?;
        let node = self.tree.get_node(node_id)?;

        // For attribute nodes, get the value
        if node.is_attribute() || node.node_type == "text" || node.node_type == "comment" {
            node.attributes.first().map(|attr| attr.value.clone())
        } else {
            None
        }
    }

    // Get the key/label of the current node
    fn get_node_key(&self) -> Option<String> {
        let node_id = self.tree_view.get_selected_node_id()?;
        let node = self.tree.get_node(node_id)?;
        Some(node.label.clone())
    }

    /// Execute the selected decode option
    fn execute_decode_option(&mut self) {
        let value = match self.get_node_string_value() {
            Some(v) => v,
            None => return,
        };

        let options = DecodeOption::all();
        if self.decode_menu_selected >= options.len() {
            return;
        }

        let option = options[self.decode_menu_selected];
        let decoded = match option {
            DecodeOption::Base64ToText => self.decode_base64_to_text(&value),
            DecodeOption::Base64ToHex => self.decode_base64_to_hex(&value),
            DecodeOption::BinaryToHex => Some(self.format_hexdump(value.as_bytes())),
            DecodeOption::UnixTimestamp => self.decode_unix_timestamp(&value),
            DecodeOption::RawValue => Some(value),
        };

        if let Some(result) = decoded {
            self.print_content = Some(result);
        }
    }

    /// Decode base64 string to UTF-8 text
    fn decode_base64_to_text(&self, value: &str) -> Option<String> {
        use base64::{Engine as _, engine::general_purpose};

        let decoded = general_purpose::STANDARD.decode(value.trim()).ok()?;
        String::from_utf8(decoded).ok()
    }

    /// Decode base64 string to hexdump
    fn decode_base64_to_hex(&self, value: &str) -> Option<String> {
        use base64::{Engine as _, engine::general_purpose};

        let decoded = general_purpose::STANDARD.decode(value.trim()).ok()?;
        Some(self.format_hexdump(&decoded))
    }

    /// Format bytes as hexdump with ASCII preview
    fn format_hexdump(&self, bytes: &[u8]) -> String {
        let mut result = String::new();
        const BYTES_PER_LINE: usize = 16;

        for (line_num, chunk) in bytes.chunks(BYTES_PER_LINE).enumerate() {
            // Offset
            result.push_str(&format!("{:08x}  ", line_num * BYTES_PER_LINE));

            // Hex bytes
            for (i, byte) in chunk.iter().enumerate() {
                if i == 8 {
                    result.push(' '); // Extra space in middle
                }
                result.push_str(&format!("{:02x} ", byte));
            }

            // Padding if last line is short
            if chunk.len() < BYTES_PER_LINE {
                for i in chunk.len()..BYTES_PER_LINE {
                    if i == 8 {
                        result.push(' ');
                    }
                    result.push_str("   ");
                }
            }

            // ASCII preview
            result.push_str(" |");
            for byte in chunk {
                let ch = if byte.is_ascii_graphic() || *byte == b' ' {
                    *byte as char
                } else {
                    '.'
                };
                result.push(ch);
            }
            result.push_str("|\n");
        }

        result
    }

    /// Try to parse value as unix timestamp and convert to human-readable date
    fn decode_unix_timestamp(&self, value: &str) -> Option<String> {
        use chrono::{TimeZone, Utc};

        // Try parsing as integer (seconds)
        if let Ok(timestamp) = value.trim().parse::<i64>() {
            let dt = Utc.timestamp_opt(timestamp, 0).single()?;
            return Some(format!(
                "Unix Timestamp: {}\nUTC: {}\nLocal: {}",
                timestamp,
                dt.format("%Y-%m-%d %H:%M:%S UTC"),
                dt.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S %Z")
            ));
        }

        // Try parsing as float (seconds with fractional part)
        if let Ok(timestamp_f) = value.trim().parse::<f64>() {
            let secs = timestamp_f.floor() as i64;
            let nanos = ((timestamp_f - timestamp_f.floor()) * 1_000_000_000.0) as u32;
            let dt = Utc.timestamp_opt(secs, nanos).single()?;
            return Some(format!(
                "Unix Timestamp: {}\nUTC: {}\nLocal: {}",
                timestamp_f,
                dt.format("%Y-%m-%d %H:%M:%S%.3f UTC"),
                dt.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S%.3f %Z")
            ));
        }

        None
    }

    /// Convert a tree node to a JSON value
    /// Recursively builds JSON objects and arrays from tree structure
    fn node_to_json(
        &self,
        node_id: usize,
        node: &crate::tree::TreeNode,
    ) -> Option<serde_json::Value> {
        use serde_json::{Map, Value};

        // For attribute nodes, return the value directly
        if node.is_attribute() {
            if let Some(attr) = node.attributes.first() {
                // Try to parse as JSON, otherwise return as string
                return serde_json::from_str(&attr.value)
                    .unwrap_or_else(|_| Value::String(attr.value.clone()))
                    .into();
            }
        }

        // For text/comment nodes, return the content
        if node.node_type == "text" || node.node_type == "comment" {
            if let Some(content) = node.attributes.iter().find(|a| a.key == "content") {
                return Some(Value::String(content.value.clone()));
            }
        }

        // For container nodes, recursively build object or array
        if node.node_type == "object" {
            let mut map = Map::new();
            let children = self.tree.get_children(node_id);
            for child_id in children {
                if let Some(child_node) = self.tree.get_node(child_id) {
                    // Skip virtual attributes node
                    if child_node.node_type == crate::tree::TreeNode::VIRTUAL_ATTRIBUTES_TYPE {
                        continue;
                    }
                    if let Some(value) = self.node_to_json(child_id, &child_node) {
                        map.insert(child_node.label.clone(), value);
                    }
                }
            }
            return Some(Value::Object(map));
        } else if node.node_type == "array" {
            let mut arr = Vec::new();
            let children = self.tree.get_children(node_id);
            for child_id in children {
                if let Some(child_node) = self.tree.get_node(child_id) {
                    // Skip virtual attributes node
                    if child_node.node_type == crate::tree::TreeNode::VIRTUAL_ATTRIBUTES_TYPE {
                        continue;
                    }
                    if let Some(value) = self.node_to_json(child_id, &child_node) {
                        arr.push(value);
                    }
                }
            }
            return Some(Value::Array(arr));
        }

        // Default: return label as string
        Some(Value::String(node.label.clone()))
    }

    // Copy text to clipboard
    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        use arboard::Clipboard;
        let mut clipboard =
            Clipboard::new().map_err(|e| XtvError::Tui(format!("Clipboard error: {}", e)))?;
        clipboard
            .set_text(text.to_string())
            .map_err(|e| XtvError::Tui(format!("Failed to copy to clipboard: {}", e)))?;
        Ok(())
    }

    // Perform search and update matches
    fn perform_search(&mut self) {
        self.search_matches.clear();
        self.current_match_index = None;

        if self.search_query.is_empty() {
            return;
        }

        let query = if self.case_sensitive {
            self.search_query.clone()
        } else {
            self.search_query.to_lowercase()
        };

        // Use iterative depth-first traversal with a stack to avoid collecting all nodes
        // This prevents memory exhaustion on large streaming trees
        let mut stack = vec![self.tree.root_id()];
        let mut nodes_searched = 0;

        while let Some(node_id) = stack.pop() {
            // Enforce search limit to prevent memory exhaustion
            nodes_searched += 1;
            if nodes_searched > MAX_SEARCH_NODES {
                eprintln!(
                    "Warning: Search stopped after examining {} nodes (limit reached)",
                    MAX_SEARCH_NODES
                );
                break;
            }

            if let Some(node) = self.tree.get_node(node_id) {
                // Check if this node matches
                let matches = if self.case_sensitive {
                    node.label.contains(&query)
                        || node.node_type.contains(&query)
                        || node
                            .attributes
                            .iter()
                            .any(|attr| attr.key.contains(&query) || attr.value.contains(&query))
                } else {
                    node.label.to_lowercase().contains(&query)
                        || node.node_type.to_lowercase().contains(&query)
                        || node.attributes.iter().any(|attr| {
                            attr.key.to_lowercase().contains(&query)
                                || attr.value.to_lowercase().contains(&query)
                        })
                };

                if matches {
                    self.search_matches.push(node_id);
                }

                // Add children to stack for depth-first traversal
                // Push in reverse order so we process them in original order
                let children = self.tree.get_children(node_id);
                for child_id in children.iter().rev() {
                    stack.push(*child_id);
                }
            }
        }

        // Set current match to first result if any
        if !self.search_matches.is_empty() {
            self.current_match_index = Some(0);
            self.jump_to_current_match();
        }
    }

    // Jump to the current search match
    fn jump_to_current_match(&mut self) {
        if let Some(index) = self.current_match_index {
            if let Some(&node_id) = self.search_matches.get(index) {
                // Expand all parents of the matched node
                self.expand_to_node(node_id);
                // Navigate to the matched node
                self.tree_view.navigate_to_node(&self.tree, node_id);
            }
        }
    }

    // Expand all parent nodes to make a node visible
    fn expand_to_node(&mut self, node_id: usize) {
        let mut path = Vec::new();
        let mut current = node_id;

        // Build path from node to root
        while let Some(parent_id) = self.tree.get_parent(current) {
            path.push(parent_id);
            current = parent_id;
        }

        // Expand all nodes in the path
        for &id in &path {
            self.tree_view.expand_node(id);
        }
    }

    // Navigate to next search match
    fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        if let Some(index) = self.current_match_index {
            self.current_match_index = Some((index + 1) % self.search_matches.len());
        } else {
            self.current_match_index = Some(0);
        }

        self.jump_to_current_match();
    }

    // Navigate to previous search match
    fn previous_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        if let Some(index) = self.current_match_index {
            self.current_match_index = if index == 0 {
                Some(self.search_matches.len() - 1)
            } else {
                Some(index - 1)
            };
        } else {
            self.current_match_index = Some(0);
        }

        self.jump_to_current_match();
    }

    fn render_help_popup(&self, frame: &mut ratatui::Frame) {
        use ratatui::{
            layout::Alignment,
            style::{Color, Modifier, Style},
            text::{Line, Span},
            widgets::{Block, Borders, Clear, Paragraph},
        };

        // Create centered popup area
        let area = frame.size();
        let popup_width = HELP_POPUP_WIDTH.min(area.width - 4);
        let popup_height = HELP_POPUP_HEIGHT.min(area.height - 4);
        let popup_x = (area.width - popup_width) / 2;
        let popup_y = (area.height - popup_height) / 2;

        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Clear the area
        frame.render_widget(Clear, popup_area);

        // Create help text
        let help_lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Navigation",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ↑/k       Move up              ↓/j       Move down"),
            Line::from("  PgUp/[    Move up 10 items     PgDn/]    Move down 10 items"),
            Line::from("  g         First line           G         Last line"),
            Line::from("  J         Next sibling         K         Previous sibling"),
            Line::from("  0         First sibling        $         Last sibling"),
            Line::from("  H         Navigate to parent"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Tree Manipulation",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  →/l       Smart right: expand or move to first child"),
            Line::from("  ←/h       Smart left: collapse or move to parent"),
            Line::from("  Space     Toggle expand/collapse current node"),
            Line::from("  Enter     Toggle expand/collapse current node"),
            Line::from("  e         Expand siblings      E         Expand siblings (deep)"),
            Line::from("  c         Collapse siblings    C         Collapse siblings (deep)"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Copy/Print/Decode",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  yy        Copy value (pretty)  pp        Print value (pretty)"),
            Line::from("  yv        Copy value (compact) pv        Print value (compact)"),
            Line::from("  ys        Copy string value    ps        Print string value"),
            Line::from("  yk        Copy key/label       pk        Print key/label"),
            Line::from("  d         Decode value (base64, hex, timestamp)"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Search",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  /         Start search (case-insensitive)"),
            Line::from("  n         Jump to next match"),
            Line::from("  N         Jump to previous match"),
            Line::from("  Esc       Clear search / Quit"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Other",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ?         Toggle this help"),
            Line::from("  q         Quit"),
        ];

        let help_paragraph = Paragraph::new(help_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Keyboard Shortcuts ")
                    .title_alignment(Alignment::Center)
                    .style(Style::default().bg(Color::Black)),
            )
            .alignment(Alignment::Left);

        frame.render_widget(help_paragraph, popup_area);
    }

    fn render_decode_menu(&self, frame: &mut ratatui::Frame) {
        use ratatui::{
            layout::Alignment,
            style::{Color, Modifier, Style},
            text::{Line, Span},
            widgets::{Block, Borders, Clear, List, ListItem},
        };

        // Create centered popup area
        let area = frame.size();
        let popup_width = 60;
        let popup_height = 10;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = ratatui::layout::Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Clear the area
        frame.render_widget(Clear, popup_area);

        // Create menu items
        let options = DecodeOption::all();
        let items: Vec<ListItem> = options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let content = if i == self.decode_menu_selected {
                    Line::from(vec![
                        Span::styled(
                            " → ",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            option.label(),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(option.label(), Style::default().fg(Color::White)),
                    ])
                };
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Decode Value ")
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::Black)),
        );

        frame.render_widget(list, popup_area);
    }

    fn render_print_popup(&self, frame: &mut ratatui::Frame) {
        use ratatui::{
            layout::Alignment,
            style::{Color, Style},
            widgets::{Block, Borders, Clear, Paragraph, Wrap},
        };

        if let Some(content) = &self.print_content {
            // Create centered popup area
            let area = frame.size();
            let popup_width = (area.width * PRINT_POPUP_WIDTH_FRACTION / PRINT_POPUP_WIDTH_DIVISOR)
                .min(PRINT_POPUP_MAX_WIDTH);
            let popup_height = (area.height * PRINT_POPUP_HEIGHT_FRACTION
                / PRINT_POPUP_HEIGHT_DIVISOR)
                .min(PRINT_POPUP_MAX_HEIGHT);
            let popup_x = (area.width - popup_width) / 2;
            let popup_y = (area.height - popup_height) / 2;

            let popup_area = ratatui::layout::Rect {
                x: popup_x,
                y: popup_y,
                width: popup_width,
                height: popup_height,
            };

            // Clear the area
            frame.render_widget(Clear, popup_area);

            // Create paragraph with content
            let paragraph = Paragraph::new(content.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Printed Content (press any key to close) ")
                        .title_alignment(Alignment::Center)
                        .style(Style::default().bg(Color::Black)),
                )
                .wrap(Wrap { trim: false })
                .alignment(Alignment::Left);

            frame.render_widget(paragraph, popup_area);
        }
    }
}
