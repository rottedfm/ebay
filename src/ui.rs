use crate::app::{App, AppState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Stylize, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

impl Widget for &App {
    /// Renders the user interface widgets.
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.state {
            AppState::Loading => self.render_loading(area, buf),
            AppState::Running => self.render_running(area, buf),
        }
    }
}

impl App {
    fn render_loading(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("ebay")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3), // For the paragraph
                Constraint::Length(1), // For the gauge
                Constraint::Min(0),
            ])
            .split(inner_area);

        let paragraph = Paragraph::new(if self.progress_message.is_empty() {
            "Loading..."
        } else {
            self.progress_message.as_str()
        })
            .fg(Color::White)
            .bg(Color::Black)
            .centered();

        let num_dots = 20;
        let filled_dots = (self.progress * num_dots as f64) as usize;
        let empty_dots = num_dots - filled_dots;

        let mut spans = Vec::new();
        for _ in 0..filled_dots {
            spans.push(Span::styled(".", Style::default().fg(Color::Green)));
        }
        for _ in 0..empty_dots {
            spans.push(Span::styled(".", Style::default().fg(Color::DarkGray)));
        }

        let gauge_line = Line::from(spans);
        let gauge_paragraph = Paragraph::new(gauge_line).centered();

        paragraph.render(chunks[1], buf);
        gauge_paragraph.render(chunks[2], buf);
    }

    fn render_running(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("ebay")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        let text = format!(
            "{}, {}, {}",
            self.feedback_score.as_deref().unwrap_or("N/A"),
            self.items_sold.unwrap_or(0),
            self.follower_count.unwrap_or(0)
        );
        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();
        paragraph.render(area, buf);
    }
}