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
}

impl App {
    pub fn new(tree: TreeVariant) -> Self {
        let tree_view = TreeView::new(tree.root_id());

        Self {
            tree,
            tree_view,
            should_quit: false,
            show_help: false,
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
        let help_text = " ↑/↓: Navigate | Enter/→: Expand | ←: Collapse | c: Collapse Parent | q: Quit | ?: Help ";
        let status_bar = Paragraph::new(help_text);
        frame.render_widget(status_bar, main_chunks[2]);

        // Render help popup if shown
        if self.show_help {
            self.render_help_popup(frame);
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
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                self.tree_view.toggle_expand(&self.tree);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.tree_view.collapse(&self.tree);
            }
            KeyCode::Char('c') => {
                self.tree_view.collapse_parent(&self.tree);
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
            _ => {}
        }
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
        let popup_width = 60.min(area.width - 4);
        let popup_height = 18.min(area.height - 4);
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
            Line::from("  ↑/k       Move up"),
            Line::from("  ↓/j       Move down"),
            Line::from("  PgUp      Move up 10 items"),
            Line::from("  PgDn      Move down 10 items"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Tree Manipulation",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  Enter/→/l Expand node"),
            Line::from("  ←/h       Collapse node"),
            Line::from("  c         Collapse parent node"),
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
}
