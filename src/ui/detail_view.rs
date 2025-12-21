use crate::tree::TreeNode;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct DetailView;

impl Default for DetailView {
    fn default() -> Self {
        Self::new()
    }
}

impl DetailView {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, node: Option<&TreeNode>) {
        if let Some(node) = node {
            self.render_node_details(frame, area, node);
        } else {
            self.render_empty(frame, area);
        }
    }

    fn render_node_details(&self, frame: &mut Frame, area: Rect, node: &TreeNode) {
        let mut items = Vec::new();

        // Node label
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                "Label: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&node.label, Style::default().fg(Color::Cyan)),
        ])));

        // Node type
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                "Type: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&node.node_type, Style::default().fg(Color::Magenta)),
        ])));

        // Children count
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                "Children: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", node.children.len()),
                Style::default().fg(Color::Green),
            ),
        ])));

        // Separator
        if !node.attributes.is_empty() {
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from(Span::styled(
                "Attributes:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ))));

            // All attributes
            for attr in node.attributes.iter() {
                // Attribute key line: 4 spaces indent
                items.push(ListItem::new(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        format!("{}:", attr.key),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])));

                // Attribute value lines: 8 spaces indent
                let value_lines =
                    self.wrap_text(&attr.value, (area.width as usize).saturating_sub(8));
                for line in value_lines.iter() {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("        "),
                        Span::styled(line.clone(), Style::default().fg(Color::Green)),
                    ])));
                }
            }
        } else {
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from(Span::styled(
                "(No attributes)",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))));
        }

        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title("Node Details"));

        frame.render_widget(list, area);
    }

    fn render_empty(&self, frame: &mut Frame, area: Rect) {
        let items = vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "No node selected",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))),
        ];

        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title("Node Details"));

        frame.render_widget(list, area);
    }

    fn wrap_text(&self, text: &str, max_width: usize) -> Vec<String> {
        if max_width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(text.to_string());
        }

        lines
    }
}
