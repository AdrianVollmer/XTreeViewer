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

pub struct App {
    tree: TreeVariant,
    tree_view: TreeView,
    should_quit: bool,
    show_help: bool,
    last_key_was_y: bool,
    last_key_was_p: bool,
    print_content: Option<String>,
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
        self.tree_view.render(frame, main_chunks[0], &self.tree);

        // Render path bar
        let path = self.get_node_path();
        let path_bar = Paragraph::new(path).style(Style::default().fg(Color::Gray));
        frame.render_widget(path_bar, main_chunks[1]);

        // Render footer
        let help_text = " ↑/↓/j/k: Move | h/l: Smart nav | Space: Toggle | J/K: Siblings | g/G: First/Last | ?: Help | q: Quit ";
        let status_bar = Paragraph::new(help_text);
        frame.render_widget(status_bar, main_chunks[2]);

        // Render help popup if shown
        if self.show_help {
            self.render_help_popup(frame);
        }

        // Render print popup if content is set
        if self.print_content.is_some() {
            self.render_print_popup(frame);
        }
    }

    fn get_node_path(&self) -> String {
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
        // If print popup is shown, any key closes it
        if self.print_content.is_some() {
            self.print_content = None;
            return Ok(());
        }

        // If help is shown, handle help-specific keys
        if self.show_help {
            match key.code {
                KeyCode::Char('?') | KeyCode::Esc => {
                    self.show_help = false;
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle 'y' prefix commands (yank/copy)
        if self.last_key_was_y {
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
        }

        // Handle 'p' prefix commands (print)
        if self.last_key_was_p {
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
        }

        // Normal key handling
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Esc => {
                self.should_quit = true;
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
            KeyCode::PageUp => {
                for _ in 0..10 {
                    self.tree_view.navigate_up();
                }
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
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
            KeyCode::Char('[') => {
                for _ in 0..10 {
                    self.tree_view.navigate_up();
                }
            }
            KeyCode::Char(']') => {
                for _ in 0..10 {
                    self.tree_view.navigate_down(&self.tree);
                }
            }
            _ => {}
        }

        // Reset all prefix flags if we didn't handle them
        self.last_key_was_y = false;
        self.last_key_was_p = false;

        Ok(())
    }

    // Get the node value as pretty-printed JSON
    fn get_node_value_pretty(&self) -> Option<String> {
        let node_id = self.tree_view.get_selected_node_id()?;
        let node = self.tree.get_node(node_id)?;

        // Convert node to JSON value and pretty print
        let json_value = self.node_to_json(&node)?;
        serde_json::to_string_pretty(&json_value).ok()
    }

    // Get the node value as compact one-line JSON
    fn get_node_value_compact(&self) -> Option<String> {
        let node_id = self.tree_view.get_selected_node_id()?;
        let node = self.tree.get_node(node_id)?;

        let json_value = self.node_to_json(&node)?;
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

    // Convert a tree node to a JSON value
    fn node_to_json(&self, node: &crate::tree::TreeNode) -> Option<serde_json::Value> {
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

        // For container nodes, build object or array
        if node.node_type == "object" {
            let map = Map::new();
            // Get children and build object
            // This is a simplified version - in reality we'd need to traverse children
            Some(Value::Object(map))
        } else if node.node_type == "array" {
            Some(Value::Array(vec![]))
        } else {
            Some(Value::String(node.label.clone()))
        }
    }

    // Copy text to clipboard
    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        use arboard::Clipboard;
        let mut clipboard = Clipboard::new().map_err(|e| XtvError::Tui(format!("Clipboard error: {}", e)))?;
        clipboard
            .set_text(text.to_string())
            .map_err(|e| XtvError::Tui(format!("Failed to copy to clipboard: {}", e)))?;
        Ok(())
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
        let popup_width = 80.min(area.width - 4);
        let popup_height = 25.min(area.height - 4);
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
                "Copy/Print",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  yy        Copy value (pretty)  pp        Print value (pretty)"),
            Line::from("  yv        Copy value (compact) pv        Print value (compact)"),
            Line::from("  ys        Copy string value    ps        Print string value"),
            Line::from("  yk        Copy key/label       pk        Print key/label"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Other",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  ?         Toggle this help"),
            Line::from("  q/Esc     Quit"),
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

    fn render_print_popup(&self, frame: &mut ratatui::Frame) {
        use ratatui::{
            layout::Alignment,
            style::{Color, Style},
            widgets::{Block, Borders, Clear, Paragraph, Wrap},
        };

        if let Some(content) = &self.print_content {
            // Create centered popup area
            let area = frame.size();
            let popup_width = (area.width * 4 / 5).min(100);
            let popup_height = (area.height * 3 / 4).min(30);
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
