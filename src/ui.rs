use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create lines of spans (each line is a row in the UI)
        let lines = vec![
            Line::from(Span::raw(format!("Feedback: {}", self.feedback))),
            Line::from(Span::raw(format!("Items Sold: {}", self.items_sold))),
            Line::from(Span::raw(format!("Follower Count: {}", self.followers))),
        ];

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center).block(
            Block::default()
                .title("Store Stats")
                .borders(ratatui::widgets::Borders::ALL)
                .border_type(BorderType::Plain),
        );

        paragraph.render(area, buf);
    }
}
