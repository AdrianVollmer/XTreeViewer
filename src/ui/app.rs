use crate::error::{Result, XtvError};
use crate::tree::Tree;
use crate::ui::detail_view::DetailView;
use crate::ui::tree_view::TreeView;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
    Terminal,
};
use std::io;

pub struct App {
    tree: Tree,
    tree_view: TreeView,
    detail_view: DetailView,
    should_quit: bool,
    show_help: bool,
}

impl App {
    pub fn new(tree: Tree) -> Self {
        let tree_view = TreeView::new(tree.root_id());
        let detail_view = DetailView::new();

        Self {
            tree,
            tree_view,
            detail_view,
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
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(frame.size());

        // Split the main area horizontally for tree view and detail view
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(main_chunks[0]);

        // Render tree view
        self.tree_view.render(frame, content_chunks[0], &self.tree);

        // Get the selected node and render detail view
        let selected_node = self
            .tree_view
            .get_selected_node_id()
            .and_then(|id| self.tree.get_node(id));
        self.detail_view
            .render(frame, content_chunks[1], selected_node);

        // Render status bar
        let help_text = " ↑/↓: Navigate | Enter/→: Expand | ←: Collapse | c: Collapse Parent | q: Quit | ?: Help ";
        let status_bar = Paragraph::new(help_text);
        frame.render_widget(status_bar, main_chunks[1]);

        // Render help popup if shown
        if self.show_help {
            self.render_help_popup(frame);
        }
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
            Line::from(vec![
                Span::styled("Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  ↑/k       Move up"),
            Line::from("  ↓/j       Move down"),
            Line::from("  PgUp      Move up 10 items"),
            Line::from("  PgDn      Move down 10 items"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Tree Manipulation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  Enter/→/l Expand node"),
            Line::from("  ←/h       Collapse node"),
            Line::from("  c         Collapse parent node"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Other", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  ?         Toggle this help"),
            Line::from("  q/Esc     Quit"),
        ];

        let help_paragraph = Paragraph::new(help_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Keyboard Shortcuts ")
                    .title_alignment(Alignment::Center)
                    .style(Style::default().bg(Color::Black))
            )
            .alignment(Alignment::Left);

        frame.render_widget(help_paragraph, popup_area);
    }
}
