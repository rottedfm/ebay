use crate::app::{App, AppState, ScrollViewMode};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Stylize, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Table, Row, Cell},
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
            .fg(Color::Magenta)
            .bg(Color::Black)
            .centered();

        let num_dots = 20;
        let filled_dots = (self.progress * num_dots as f64) as usize;
        let empty_dots = num_dots - filled_dots;

        let mut spans = Vec::new();
        for _ in 0..filled_dots {
            spans.push(Span::styled(".", Style::default().fg(Color::Magenta)));
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
        self.render_combined_scrollview(area, buf);
    }

    fn render_paragraph_view(&self, area: Rect, buf: &mut Buffer) {
        let main_block = Block::bordered()
            .title("eBay Seller Dashboard - Paragraph View")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let inner_area = main_block.inner(area);
        main_block.render(area, buf);

        // Create comprehensive paragraph content
        let mut paragraph_lines = Vec::new();
        
        // Title section
        paragraph_lines.push(Line::from(vec![
            Span::styled("📊 EBAY SELLER DASHBOARD", Style::default().fg(Color::Magenta).bold())
        ]));
        paragraph_lines.push(Line::from(""));
        
        // Stats section
        paragraph_lines.push(Line::from(vec![
            Span::styled("🏪 SELLER STATISTICS", Style::default().fg(Color::Cyan).bold())
        ]));
        paragraph_lines.push(Line::from(""));
        
        paragraph_lines.push(Line::from(vec![
            Span::styled("Feedback Score: ", Style::default().fg(Color::White)),
            Span::styled(
                self.feedback_score.as_deref().unwrap_or("N/A"), 
                Style::default().fg(Color::Green).bold()
            )
        ]));
        
        paragraph_lines.push(Line::from(vec![
            Span::styled("Items Sold: ", Style::default().fg(Color::White)),
            Span::styled(
                self.items_sold.unwrap_or(0).to_string(), 
                Style::default().fg(Color::Yellow).bold()
            )
        ]));
        
        paragraph_lines.push(Line::from(vec![
            Span::styled("Followers: ", Style::default().fg(Color::White)),
            Span::styled(
                self.follower_count.unwrap_or(0).to_string(), 
                Style::default().fg(Color::Blue).bold()
            )
        ]));
        
        paragraph_lines.push(Line::from(""));
        paragraph_lines.push(Line::from(""));
        
        // Listings overview
        paragraph_lines.push(Line::from(vec![
            Span::styled("📋 LISTINGS OVERVIEW", Style::default().fg(Color::Cyan).bold())
        ]));
        paragraph_lines.push(Line::from(""));
        
        paragraph_lines.push(Line::from(vec![
            Span::styled("Total Active Listings: ", Style::default().fg(Color::White)),
            Span::styled(
                self.listings.len().to_string(), 
                Style::default().fg(Color::Green).bold()
            )
        ]));
        
        paragraph_lines.push(Line::from(""));
        
        // Sample listings (first few)
        if !self.listings.is_empty() {
            paragraph_lines.push(Line::from(vec![
                Span::styled("🔍 RECENT LISTINGS PREVIEW", Style::default().fg(Color::Cyan).bold())
            ]));
            paragraph_lines.push(Line::from(""));
            
            for (i, listing) in self.listings.iter().take(10).enumerate() {
                paragraph_lines.push(Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        listing.title.chars().take(60).collect::<String>(), 
                        Style::default().fg(Color::White)
                    )
                ]));
                paragraph_lines.push(Line::from(vec![
                    Span::styled("   Price: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&listing.price, Style::default().fg(Color::Green)),
                    Span::styled(" | Condition: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        listing.condition.as_deref().unwrap_or("N/A"), 
                        Style::default().fg(Color::Yellow)
                    )
                ]));
                paragraph_lines.push(Line::from(""));
            }
            
            if self.listings.len() > 10 {
                paragraph_lines.push(Line::from(vec![
                    Span::styled(
                        format!("... and {} more listings", self.listings.len() - 10), 
                        Style::default().fg(Color::DarkGray)
                    )
                ]));
                paragraph_lines.push(Line::from(""));
            }
        }
        
        // Instructions
        paragraph_lines.push(Line::from(""));
        paragraph_lines.push(Line::from(vec![
            Span::styled("🎮 NAVIGATION", Style::default().fg(Color::Cyan).bold())
        ]));
        paragraph_lines.push(Line::from(""));
        paragraph_lines.push(Line::from(vec![
            Span::styled("↑/↓ j/k: ", Style::default().fg(Color::Yellow)),
            Span::styled("Scroll up/down", Style::default().fg(Color::White))
        ]));
        paragraph_lines.push(Line::from(vec![
            Span::styled("PgUp/PgDn: ", Style::default().fg(Color::Yellow)),
            Span::styled("Fast scroll", Style::default().fg(Color::White))
        ]));
        paragraph_lines.push(Line::from(vec![
            Span::styled("Home/End: ", Style::default().fg(Color::Yellow)),
            Span::styled("Go to top/bottom", Style::default().fg(Color::White))
        ]));
        paragraph_lines.push(Line::from(vec![
            Span::styled("Enter: ", Style::default().fg(Color::Green).bold()),
            Span::styled("Switch to Table View", Style::default().fg(Color::White).bold())
        ]));
        paragraph_lines.push(Line::from(vec![
            Span::styled("q/Esc: ", Style::default().fg(Color::Red)),
            Span::styled("Quit application", Style::default().fg(Color::White))
        ]));

        // Create a paragraph with all the lines
        let text = Text::from(paragraph_lines);
        let paragraph = Paragraph::new(text)
            .scroll((self.paragraph_scroll_offset as u16, 0));
            
        paragraph.render(inner_area, buf);

        // Status line at bottom
        let status_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 1,
            width: area.width - 4,
            height: 1,
        };
        let status_text = format!("📄 Paragraph View | Scroll: {} | Press Enter to switch to Table View", self.paragraph_scroll_offset);
        let status_paragraph = Paragraph::new(status_text)
            .fg(Color::Magenta)
            .bg(Color::Black)
            .alignment(Alignment::Center);
        status_paragraph.render(status_area, buf);
    }

    fn render_table_view(&self, area: Rect, buf: &mut Buffer) {
        let main_block = Block::bordered()
            .title(format!("eBay Listings - Table View ({})", self.listings.len()))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let inner_area = main_block.inner(area);
        main_block.render(area, buf);

        if self.listings.is_empty() {
            let no_listings = Paragraph::new("No listings found")
                .fg(Color::Magenta)
                .bg(Color::Black)
                .centered();
            no_listings.render(inner_area, buf);
        } else {
            // Create table rows from listings
            let header = Row::new(vec![
                Cell::from("Title").style(Style::default().fg(Color::Magenta).bg(Color::Black)),
                Cell::from("Price").style(Style::default().fg(Color::Magenta).bg(Color::Black)),
                Cell::from("Shipping").style(Style::default().fg(Color::Magenta).bg(Color::Black)),
                Cell::from("Condition").style(Style::default().fg(Color::Magenta).bg(Color::Black)),
            ]);

            let visible_rows = (inner_area.height.saturating_sub(3)) as usize; // Account for header and borders
            let rows: Vec<Row> = self.listings
                .iter()
                .skip(self.scroll_offset)
                .take(visible_rows)
                .enumerate()
                .map(|(relative_i, listing)| {
                    let absolute_i = relative_i + self.scroll_offset;
                    let style = if absolute_i == self.selected_listing_index {
                        Style::default().fg(Color::Black).bg(Color::Magenta)
                    } else {
                        Style::default().fg(Color::Magenta).bg(Color::Black)
                    };
                    
                    Row::new(vec![
                        Cell::from(listing.title.chars().take(40).collect::<String>()),
                        Cell::from(listing.price.as_str()),
                        Cell::from(listing.shipping.as_deref().unwrap_or("N/A")),
                        Cell::from(listing.condition.as_deref().unwrap_or("N/A")),
                    ]).style(style)
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(50),  // Title
                    Constraint::Percentage(15),  // Price
                    Constraint::Percentage(20),  // Shipping
                    Constraint::Percentage(15),  // Condition
                ]
            )
            .header(header)
            .column_spacing(1);
            
            table.render(inner_area, buf);
        }

        // Status line at bottom
        let status_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 1,
            width: area.width - 4,
            height: 1,
        };
        let status_text = if !self.listings.is_empty() {
            format!("📊 Table View | ↑/↓ j/k: Navigate | i: Open in Firefox | Enter: Switch to Paragraph | Selected: {}/{}", 
                   self.selected_listing_index + 1, self.listings.len())
        } else {
            "📊 Table View | No listings | Enter: Switch to Paragraph View".to_string()
        };
        let status_paragraph = Paragraph::new(status_text)
            .fg(Color::Magenta)
            .bg(Color::Black)
            .alignment(Alignment::Center);
        status_paragraph.render(status_area, buf);
    }

    fn render_combined_scrollview(&self, area: Rect, buf: &mut Buffer) {
        let main_block = Block::bordered()
            .title(format!(
                "eBay Seller Dashboard{}", 
                if self.section_locked { " - LOCKED" } else { "" }
            ))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let inner_area = main_block.inner(area);
        main_block.render(area, buf);

        // Create the combined content
        let mut combined_content = Vec::new();
        
        // Add paragraph section
        combined_content.push(Line::from(vec![
            Span::styled("📊 EBAY SELLER DASHBOARD", Style::default().fg(Color::Magenta).bold())
        ]));
        combined_content.push(Line::from(""));
        
        // Stats section
        combined_content.push(Line::from(vec![
            Span::styled("🏪 SELLER STATISTICS", Style::default().fg(Color::Cyan).bold())
        ]));
        combined_content.push(Line::from(""));
        
        combined_content.push(Line::from(vec![
            Span::styled("Feedback Score: ", Style::default().fg(Color::White)),
            Span::styled(
                self.feedback_score.as_deref().unwrap_or("N/A"), 
                Style::default().fg(Color::Green).bold()
            )
        ]));
        
        combined_content.push(Line::from(vec![
            Span::styled("Items Sold: ", Style::default().fg(Color::White)),
            Span::styled(
                self.items_sold.unwrap_or(0).to_string(), 
                Style::default().fg(Color::Yellow).bold()
            )
        ]));
        
        combined_content.push(Line::from(vec![
            Span::styled("Followers: ", Style::default().fg(Color::White)),
            Span::styled(
                self.follower_count.unwrap_or(0).to_string(), 
                Style::default().fg(Color::Blue).bold()
            )
        ]));
        
        combined_content.push(Line::from(""));
        combined_content.push(Line::from(""));
        
        // Table section header
        combined_content.push(Line::from(vec![
            Span::styled("📋 LISTINGS TABLE", Style::default().fg(Color::Cyan).bold())
        ]));
        combined_content.push(Line::from(""));
        
        if self.listings.is_empty() {
            combined_content.push(Line::from("No listings found"));
        } else {
            // Add table header
            combined_content.push(Line::from(vec![
                Span::styled("Title", Style::default().fg(Color::Magenta).bold()),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("Price", Style::default().fg(Color::Magenta).bold()),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("Shipping", Style::default().fg(Color::Magenta).bold()),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("Condition", Style::default().fg(Color::Magenta).bold()),
            ]));
            combined_content.push(Line::from(
                "─".repeat(80)
            ));
            
            // Add table rows
            for (index, listing) in self.listings.iter().enumerate() {
                let style = if index == self.selected_listing_index && self.section_locked && self.scroll_view_mode == ScrollViewMode::Table {
                    Style::default().fg(Color::Black).bg(Color::Magenta)
                } else {
                    Style::default().fg(Color::White)
                };
                
                let title_truncated = listing.title.chars().take(35).collect::<String>();
                let price = &listing.price;
                let shipping = listing.shipping.as_deref().unwrap_or("N/A");
                let condition = listing.condition.as_deref().unwrap_or("N/A");
                
                combined_content.push(Line::from(vec![
                    Span::styled(format!("{:<35}", title_truncated), style),
                    Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:<12}", price), style),
                    Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:<15}", shipping), style),
                    Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:<10}", condition), style),
                ]));
            }
        }
        
        combined_content.push(Line::from(""));
        combined_content.push(Line::from(""));
        
        // Instructions
        combined_content.push(Line::from(vec![
            Span::styled("🎮 NAVIGATION", Style::default().fg(Color::Cyan).bold())
        ]));
        combined_content.push(Line::from(""));
        
        if self.section_locked {
            combined_content.push(Line::from(vec![
                Span::styled("LOCKED MODE:", Style::default().fg(Color::Red).bold())
            ]));
            combined_content.push(Line::from(vec![
                Span::styled("↑/↓ j/k: ", Style::default().fg(Color::Yellow)),
                Span::styled("Navigate within current section", Style::default().fg(Color::White))
            ]));
            combined_content.push(Line::from(vec![
                Span::styled("Enter: ", Style::default().fg(Color::Green).bold()),
                Span::styled("Unlock and return to scrollview", Style::default().fg(Color::White).bold())
            ]));
            if self.scroll_view_mode == ScrollViewMode::Table {
                combined_content.push(Line::from(vec![
                    Span::styled("i: ", Style::default().fg(Color::Blue)),
                    Span::styled("Open selected item in Firefox", Style::default().fg(Color::White))
                ]));
            }
        } else {
            combined_content.push(Line::from(vec![
                Span::styled("SCROLLVIEW MODE:", Style::default().fg(Color::Green).bold())
            ]));
            combined_content.push(Line::from(vec![
                Span::styled("↑/↓ j/k: ", Style::default().fg(Color::Yellow)),
                Span::styled("Scroll entire view", Style::default().fg(Color::White))
            ]));
            combined_content.push(Line::from(vec![
                Span::styled("Tab: ", Style::default().fg(Color::Cyan)),
                Span::styled("Switch focus between sections", Style::default().fg(Color::White))
            ]));
            combined_content.push(Line::from(vec![
                Span::styled("Enter: ", Style::default().fg(Color::Green).bold()),
                Span::styled("Lock to current section", Style::default().fg(Color::White).bold())
            ]));
            combined_content.push(Line::from(vec![
                Span::styled("PgUp/PgDn, Home/End: ", Style::default().fg(Color::Yellow)),
                Span::styled("Fast navigation", Style::default().fg(Color::White))
            ]));
        }
        
        combined_content.push(Line::from(vec![
            Span::styled("q/Esc: ", Style::default().fg(Color::Red)),
            Span::styled("Quit application", Style::default().fg(Color::White))
        ]));

        // Create the paragraph widget with scroll support
        let text = Text::from(combined_content);
        let paragraph = Paragraph::new(text)
            .scroll((
                if self.section_locked {
                    match self.scroll_view_mode {
                        ScrollViewMode::Paragraph => self.paragraph_scroll_offset as u16,
                        ScrollViewMode::Table => {
                            // Calculate offset to keep selected item visible
                            let visible_lines = inner_area.height.saturating_sub(2) as usize;
                            let table_start = 10; // Approximate line where table starts
                            let selected_line = table_start + self.selected_listing_index + 3;
                            if selected_line >= visible_lines {
                                (selected_line - visible_lines + 1) as u16
                            } else {
                                0
                            }
                        }
                    }
                } else {
                    self.scroll_view_state.vertical_scroll as u16
                },
                0,
            ));
        
        paragraph.render(inner_area, buf);

        // Status line at bottom
        let status_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 1,
            width: area.width - 4,
            height: 1,
        };
        
        let status_text = if self.section_locked {
            match self.scroll_view_mode {
                ScrollViewMode::Paragraph => "📄 LOCKED to Paragraph Section | Press Enter to unlock".to_string(),
                ScrollViewMode::Table => format!(
                    "📊 LOCKED to Table Section | Item {}/{} | Press Enter to unlock | i: Open in Firefox",
                    if self.listings.is_empty() { 0 } else { self.selected_listing_index + 1 },
                    self.listings.len()
                ),
            }
        } else {
            format!(
                "🔄 Scrollview Mode | Focus: {} | Tab: Switch | Enter: Lock", 
                match self.scroll_view_mode {
                    ScrollViewMode::Paragraph => "Paragraph",
                    ScrollViewMode::Table => "Table",
                }
            )
        };
        
        let status_paragraph = Paragraph::new(status_text)
            .fg(Color::Magenta)
            .bg(Color::Black)
            .alignment(Alignment::Center);
        status_paragraph.render(status_area, buf);
    }
}
