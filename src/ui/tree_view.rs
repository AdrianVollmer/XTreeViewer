use crate::tree::Tree;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::collections::HashSet;

pub struct TreeView {
    expanded: HashSet<usize>,
    visible_nodes: Vec<(usize, usize)>, // (node_id, depth)
    list_state: ListState,
}

impl TreeView {
    pub fn new(root_id: usize) -> Self {
        let mut expanded = HashSet::new();
        expanded.insert(root_id); // Root is expanded by default

        let mut view = Self {
            expanded,
            visible_nodes: Vec::new(),
            list_state: ListState::default(),
        };

        view.list_state.select(Some(0));
        view
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, tree: &Tree) {
        // Rebuild visible nodes list
        self.rebuild_visible_nodes(tree);

        // Create list items - collect the visible nodes data first to avoid borrow issues
        let visible_nodes_copy = self.visible_nodes.clone();
        let items: Vec<ListItem> = visible_nodes_copy
            .iter()
            .map(|(node_id, depth)| {
                let node = tree.get_node(*node_id).unwrap();
                self.create_list_item(node, *depth, *node_id)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tree View"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn create_list_item<'a>(
        &self,
        node: &'a crate::tree::TreeNode,
        depth: usize,
        node_id: usize,
    ) -> ListItem<'a> {
        let indent = "  ".repeat(depth);
        let icon = if node.has_children() {
            if self.expanded.contains(&node_id) {
                "▼"
            } else {
                "▶"
            }
        } else {
            " "
        };

        // Create display text
        let mut spans = vec![
            Span::raw(indent),
            Span::styled(icon, Style::default().fg(Color::Yellow)),
            Span::raw(" "),
            Span::styled(&node.label, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                format!("[{}]", node.node_type),
                Style::default().fg(Color::DarkGray),
            ),
        ];

        // Add first attribute value if available
        if let Some(attr) = node.attributes.first() {
            let value = if attr.value.len() > 40 {
                format!(" = {}...", &attr.value[..40])
            } else {
                format!(" = {}", attr.value)
            };
            spans.push(Span::styled(value, Style::default().fg(Color::Green)));
        }

        ListItem::new(Line::from(spans))
    }

    fn rebuild_visible_nodes(&mut self, tree: &Tree) {
        self.visible_nodes.clear();
        self.collect_visible_nodes(tree, tree.root_id(), 0);
    }

    fn collect_visible_nodes(&mut self, tree: &Tree, node_id: usize, depth: usize) {
        self.visible_nodes.push((node_id, depth));

        // If expanded, add children
        if self.expanded.contains(&node_id) {
            let children = tree.get_children(node_id);
            for child_id in children {
                self.collect_visible_nodes(tree, child_id, depth + 1);
            }
        }
    }

    pub fn navigate_up(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn navigate_down(&mut self, tree: &Tree) {
        self.rebuild_visible_nodes(tree);
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.visible_nodes.len() - 1 {
                    self.visible_nodes.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_expand(&mut self, tree: &Tree) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                let node = tree.get_node(*node_id).unwrap();
                if node.has_children() {
                    if self.expanded.contains(node_id) {
                        self.expanded.remove(node_id);
                    } else {
                        self.expanded.insert(*node_id);
                    }
                }
            }
        }
    }

    pub fn collapse(&mut self, _tree: &Tree) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if self.expanded.contains(node_id) {
                    // Collapse current node
                    self.expanded.remove(node_id);
                }
            }
        }
    }

    pub fn get_selected_node_id(&self) -> Option<usize> {
        self.list_state.selected()
            .and_then(|index| self.visible_nodes.get(index))
            .map(|(node_id, _)| *node_id)
    }
}
