use crate::tree::TreeVariant;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
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

    pub fn render(&mut self, frame: &mut Frame, area: Rect, tree: &TreeVariant) {
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
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn create_list_item(
        &self,
        node: crate::tree::TreeNode,
        depth: usize,
        node_id: usize,
    ) -> ListItem<'static> {
        let indent = "  ".repeat(depth);
        let icon = if node.is_virtual_attributes() {
            // Virtual attribute nodes get hollow/solid diamond
            if self.expanded.contains(&node_id) {
                "▽" // Hollow triangle down when expanded
            } else {
                "▷" // Hollow triangle right when collapsed
            }
        } else if node.has_children() {
            if self.expanded.contains(&node_id) {
                "▼"
            } else {
                "▶"
            }
        } else {
            " "
        };

        // Create display text
        let mut spans = vec![Span::raw(indent)];

        // Icon with special color for virtual nodes
        let icon_color = if node.is_virtual_attributes() {
            Color::Magenta
        } else {
            Color::Yellow
        };
        spans.push(Span::styled(icon, Style::default().fg(icon_color)));
        spans.push(Span::raw(" "));

        // Label with special color for virtual nodes
        let label_color = if node.is_virtual_attributes() {
            Color::Magenta
        } else {
            Color::Cyan
        };
        spans.push(Span::styled(
            node.label.clone(),
            Style::default().fg(label_color),
        ));

        // For attribute nodes, show key: value (no type bracket)
        // For text/comment nodes, show label: content
        // For regular nodes, show type
        if node.is_attribute() {
            if let Some(attr) = node.attributes.first() {
                let value = if attr.value.len() > 40 {
                    format!(": {}...", &attr.value[..40])
                } else {
                    format!(": {}", attr.value)
                };
                spans.push(Span::styled(value, Style::default().fg(Color::Green)));
            }
        } else if node.node_type == "text" || node.node_type == "comment" {
            // Show content inline for text and comment nodes
            if let Some(content_attr) = node.attributes.iter().find(|a| a.key == "content") {
                let value = if content_attr.value.len() > 40 {
                    format!(": {}...", &content_attr.value[..40])
                } else {
                    format!(": {}", content_attr.value)
                };
                spans.push(Span::styled(value, Style::default().fg(Color::Green)));
            }
        } else {
            // Only show node type for regular nodes
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[{}]", node.node_type),
                Style::default().fg(Color::DarkGray),
            ));
        }

        ListItem::new(Line::from(spans))
    }

    fn rebuild_visible_nodes(&mut self, tree: &TreeVariant) {
        self.visible_nodes.clear();
        self.collect_visible_nodes(tree, tree.root_id(), 0);
    }

    fn collect_visible_nodes(&mut self, tree: &TreeVariant, node_id: usize, depth: usize) {
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

    pub fn navigate_down(&mut self, tree: &TreeVariant) {
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

    pub fn toggle_expand(&mut self, tree: &TreeVariant) {
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

    pub fn collapse(&mut self, _tree: &TreeVariant) {
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
        self.list_state
            .selected()
            .and_then(|index| self.visible_nodes.get(index))
            .map(|(node_id, _)| *node_id)
    }

    pub fn collapse_parent(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                // Find the parent of the current node
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    // Collapse the parent node
                    self.expanded.remove(&parent_id);

                    // Navigate to the parent node
                    // Rebuild visible nodes to reflect the collapsed state
                    self.rebuild_visible_nodes(tree);

                    // Find the index of the parent in the visible nodes
                    if let Some(parent_index) = self
                        .visible_nodes
                        .iter()
                        .position(|(id, _)| *id == parent_id)
                    {
                        self.list_state.select(Some(parent_index));
                    }
                }
            }
        }
    }

    pub fn expand(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                let node = tree.get_node(*node_id).unwrap();
                if node.has_children() {
                    self.expanded.insert(*node_id);
                }
            }
        }
    }

    // Smart left: collapse if expanded, otherwise move to parent
    pub fn smart_left(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                let node = tree.get_node(*node_id).unwrap();
                if node.has_children() && self.expanded.contains(node_id) {
                    // Collapse if expanded
                    self.expanded.remove(node_id);
                } else {
                    // Move to parent
                    self.navigate_to_parent(tree);
                }
            }
        }
    }

    // Smart right: expand if collapsed, move to first child if expanded
    pub fn smart_right(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                let node = tree.get_node(*node_id).unwrap();
                if node.has_children() {
                    if !self.expanded.contains(node_id) {
                        // Expand if collapsed
                        self.expanded.insert(*node_id);
                    } else {
                        // Move to first child if expanded
                        self.rebuild_visible_nodes(tree);
                        if index + 1 < self.visible_nodes.len() {
                            self.list_state.select(Some(index + 1));
                        }
                    }
                }
            }
        }
    }

    // Navigate to parent without collapsing
    pub fn navigate_to_parent(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    // Find the index of the parent in the visible nodes
                    if let Some(parent_index) = self
                        .visible_nodes
                        .iter()
                        .position(|(id, _)| *id == parent_id)
                    {
                        self.list_state.select(Some(parent_index));
                    }
                }
            }
        }
    }

    // Navigate to next sibling
    pub fn navigate_to_next_sibling(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    let siblings = tree.get_children(parent_id);
                    if let Some(current_pos) = siblings.iter().position(|&id| id == *node_id) {
                        if current_pos + 1 < siblings.len() {
                            let next_sibling = siblings[current_pos + 1];
                            // Find this sibling in visible nodes
                            if let Some(sibling_index) = self
                                .visible_nodes
                                .iter()
                                .position(|(id, _)| *id == next_sibling)
                            {
                                self.list_state.select(Some(sibling_index));
                            }
                        }
                    }
                }
            }
        }
    }

    // Navigate to previous sibling
    pub fn navigate_to_previous_sibling(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    let siblings = tree.get_children(parent_id);
                    if let Some(current_pos) = siblings.iter().position(|&id| id == *node_id) {
                        if current_pos > 0 {
                            let prev_sibling = siblings[current_pos - 1];
                            // Find this sibling in visible nodes
                            if let Some(sibling_index) = self
                                .visible_nodes
                                .iter()
                                .position(|(id, _)| *id == prev_sibling)
                            {
                                self.list_state.select(Some(sibling_index));
                            }
                        }
                    }
                }
            }
        }
    }

    // Navigate to first sibling
    pub fn navigate_to_first_sibling(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    let siblings = tree.get_children(parent_id);
                    if let Some(first_sibling) = siblings.first() {
                        // Find this sibling in visible nodes
                        if let Some(sibling_index) = self
                            .visible_nodes
                            .iter()
                            .position(|(id, _)| *id == *first_sibling)
                        {
                            self.list_state.select(Some(sibling_index));
                        }
                    }
                }
            }
        }
    }

    // Navigate to last sibling
    pub fn navigate_to_last_sibling(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    let siblings = tree.get_children(parent_id);
                    if let Some(last_sibling) = siblings.last() {
                        // Find this sibling in visible nodes
                        if let Some(sibling_index) = self
                            .visible_nodes
                            .iter()
                            .position(|(id, _)| *id == *last_sibling)
                        {
                            self.list_state.select(Some(sibling_index));
                        }
                    }
                }
            }
        }
    }

    // Navigate to first line
    pub fn navigate_to_first_line(&mut self) {
        self.list_state.select(Some(0));
    }

    // Navigate to last line
    pub fn navigate_to_last_line(&mut self, tree: &TreeVariant) {
        self.rebuild_visible_nodes(tree);
        if !self.visible_nodes.is_empty() {
            self.list_state.select(Some(self.visible_nodes.len() - 1));
        }
    }

    // Shallow expand focused node and all its siblings
    pub fn expand_all_siblings(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    let siblings = tree.get_children(parent_id);
                    for sibling_id in siblings {
                        let sibling = tree.get_node(sibling_id).unwrap();
                        if sibling.has_children() {
                            self.expanded.insert(sibling_id);
                        }
                    }
                }
            }
        }
    }

    // Deep expand focused node and all its siblings (recursively expand all descendants)
    pub fn expand_all_siblings_deep(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    let siblings = tree.get_children(parent_id);
                    for sibling_id in siblings {
                        self.expand_recursive(tree, sibling_id);
                    }
                }
            }
        }
    }

    // Shallow collapse focused node and all its siblings
    pub fn collapse_all_siblings(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    let siblings = tree.get_children(parent_id);
                    for sibling_id in siblings {
                        self.expanded.remove(&sibling_id);
                    }
                }
            }
        }
    }

    // Deep collapse focused node and all its siblings (recursively collapse all descendants)
    pub fn collapse_all_siblings_deep(&mut self, tree: &TreeVariant) {
        if let Some(index) = self.list_state.selected() {
            if let Some((node_id, _)) = self.visible_nodes.get(index) {
                if let Some(parent_id) = tree.get_parent(*node_id) {
                    let siblings = tree.get_children(parent_id);
                    for sibling_id in siblings {
                        self.collapse_recursive(tree, sibling_id);
                    }
                }
            }
        }
    }

    // Helper: recursively expand a node and all its descendants
    fn expand_recursive(&mut self, tree: &TreeVariant, node_id: usize) {
        let node = tree.get_node(node_id).unwrap();
        if node.has_children() {
            self.expanded.insert(node_id);
            let children = tree.get_children(node_id);
            for child_id in children {
                self.expand_recursive(tree, child_id);
            }
        }
    }

    // Helper: recursively collapse a node and all its descendants
    fn collapse_recursive(&mut self, tree: &TreeVariant, node_id: usize) {
        let node = tree.get_node(node_id).unwrap();
        if node.has_children() {
            self.expanded.remove(&node_id);
            let children = tree.get_children(node_id);
            for child_id in children {
                self.collapse_recursive(tree, child_id);
            }
        }
    }

}
