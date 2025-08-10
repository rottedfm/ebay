use crate::app::{App, AppState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Stylize, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget, List, ListItem},
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
                Constraint::Length(1), // For the paragraph
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
            .bg(Color::DarkGray)
            .centered();

        let num_dots = 20;
        let filled_dots = (self.progress * num_dots as f64) as usize;
        let empty_dots = num_dots - filled_dots;

        let mut spans = Vec::new();
        for _ in 0..filled_dots {
            spans.push(Span::styled(".", Style::default().fg(Color::White)));
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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Stats section
                Constraint::Min(0),    // Listings section
            ])
            .split(area);

        // Render seller stats in the top section
        let stats_block = Block::bordered()
            .title("Seller Stats")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        let stats_text = format!(
            "Feedback: {} | Items Sold: {} | Followers: {}",
            self.feedback_score.as_deref().unwrap_or("N/A"),
            self.items_sold.unwrap_or(0),
            self.follower_count.unwrap_or(0)
        );
        let stats_paragraph = Paragraph::new(stats_text)
            .block(stats_block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();
        stats_paragraph.render(chunks[0], buf);

        // Render listings in the bottom section
        let listings_block = Block::bordered()
            .title(format!("eBay Listings ({})", self.listings.len()))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        if self.listings.is_empty() {
            let no_listings = Paragraph::new("No listings found")
                .block(listings_block)
                .fg(Color::Yellow)
                .centered();
            no_listings.render(chunks[1], buf);
        } else {
            // Create list items from listings
            let list_items: Vec<ListItem> = self.listings
                .iter()
                .take(20) // Limit to first 20 listings for display
                .enumerate()
                .map(|(i, listing)| {
                    let content = format!(
                        "{:2}. {} - {} {}",
                        i + 1,
                        listing.title,
                        listing.price,
                        listing.shipping.as_deref().unwrap_or("")
                    );
                    ListItem::new(content)
                        .style(if i % 2 == 0 { 
                            Style::default().fg(Color::White) 
                        } else { 
                            Style::default().fg(Color::LightBlue) 
                        })
                })
                .collect();

            let list = List::new(list_items)
                .block(listings_block)
                .highlight_style(Style::default().bg(Color::DarkGray));
            
            list.render(chunks[1], buf);
            
            // Show total count if more than 20 listings
            if self.listings.len() > 20 {
                let info_area = Rect {
                    x: chunks[1].x + 2,
                    y: chunks[1].y + chunks[1].height - 2,
                    width: chunks[1].width - 4,
                    height: 1,
                };
                let info_text = format!("Showing 20 of {} listings (saved to CSV)", self.listings.len());
                let info_paragraph = Paragraph::new(info_text)
                    .fg(Color::Yellow)
                    .alignment(Alignment::Center);
                info_paragraph.render(info_area, buf);
            }
        }
    }
}
