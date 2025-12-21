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
        let help_text = " ↑/↓: Navigate | Enter/→: Expand | ←: Collapse | c: Collapse Parent | q: Quit ";
        let status_bar = Paragraph::new(help_text);
        frame.render_widget(status_bar, main_chunks[1]);
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
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
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
}
