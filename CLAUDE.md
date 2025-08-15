# esd - Ebay seller dashboard

## Description
We are creating a TUI interface that uses web scraping to manage eBay inventory and listings. The current version focuses on displaying active listings and seller stats using public eBay pages. The TUI features multiple widgets in a scrollable interface: seller stats overview, active listing table viewer with filtering capabilities, status modeline, and keybinds reference.

## Tech stack
- Rust
- Tui crate: ratatui
- Webdriver crate: fantoccini
- Webdriver: geckodriver
- csv crate: csv
- toml crate: toml
- error handling crate: anyhow
- throbber crate: throbber-widgets-tui 
- scrollview crate: tui-scrollview

## Project structure
/src - Main source code
- /main.rs - init for tui / entry point
- /app.rs - holds the state and application logic
- /event.rs - handles the terminal events (key press, mouse click, resize, etc.)
- /ui.rs - renders the widgets / ui 
- /scraper.rs - holds all scraping functions for modularity
- /csv.rs - holds all csv saving logic for modularity
- /toml.rs - holds all toml data for modularity
/data - holds important data
- inventory.csv - gets updated with active listings
- sold.csv - gets updated with sold listings
- user_data.toml - gets updated with follower, sold items, and review score
/logs - holds log
- app_8_12_2025.log (timestamped for day)

## Authentication
- Uses environment variables for credentials (future implementation)
- Currently operates on public eBay pages without authentication
- Manual captcha solving when encountered

## Data Schema
### inventory.csv
Fields: title, watchers, carts, price, shipping_price, condition, description, item_id

### user_data.toml  
Fields: followers, sold_items, review_score

## Widget Functionality
### Active Listing Table
- Filter by: name, watchers
- Press 'i' to open item URL in Firefox
- Scrollable table view with all inventory data

### Keybinds
- j/down arrow: scroll down
- k/up arrow: scroll up  
- enter: select/interact with widget
- i: open item URL in Firefox (when in table)
- q: quit application

## Logging & Error Handling
- Debug level logging to timestamped files in /logs
- Network failures and eBay page changes logged to app.log
- Non-blocking functions for snappy TUI performance
- Background scraping when data files present

## Configuration
### User Settings
- Scraping interval (default: 5 minutes)
- Table display preferences (columns shown, sort order)
- Filter defaults and saved filter sets
- Firefox profile selection for item opening
- Log retention period (default: 30 days)

### App Configuration
- Max concurrent web requests (default: 3)
- Request timeout duration (default: 30 seconds)
- Retry attempts for failed requests (default: 3)
- UI refresh rate (default: 100ms)
- Table pagination size (default: 50 items)

## Error Recovery
### Automatic Recovery
- Restart scraping on network timeout
- Reload data from CSV on parsing errors
- Recreate data directory if missing
- Reset UI state on widget crashes

### Manual Recovery Actions
- Clear corrupted CSV files and rescrape
- Reset user_data.toml to defaults
- Restart browser driver on connection loss
- Force refresh all data with 'F5' key

### Graceful Degradation
- Show cached data when scraping fails
- Disable filtering if data is incomplete
- Display error messages in modeline
- Continue UI operation without network

## Status Indicators
### Modeline Display
- Connection status: ●●● (connected) / ○○○ (disconnected)
- Scraping progress: Loading... with spinner animation
- Last successful update timestamp
- Error messages and warnings
- Active filter indicators

### Visual Feedback
- Throbber animation during data loading
- Color coding for listing status (active/ended)
- Row highlighting for filtered results
- Progress bars for bulk operations
- Status icons: ✓ (success) ✗ (error) ⚠ (warning)

## Development Setup
### Prerequisites
- Rust (latest stable)
- geckodriver installed and in PATH
- Firefox browser

### Environment Variables
- Set up for future authentication needs

## Code Style Guidelines
### Rust Conventions
- Use `snake_case` for functions, variables, and modules
- Use `PascalCase` for structs, enums, and traits
- Use `SCREAMING_SNAKE_CASE` for constants
- Use `anyhow::Result<T>` with custom error types for error handling
- Use `async/await` for non-blocking operations

### Error Handling
- Define custom error types using `#[derive(Error)]`
- Use `anyhow::Context` to add context to errors
- Return `anyhow::Result<T>` from fallible functions
- Log errors before propagating with `?` operator
- Use `thiserror` crate for custom error definitions

### Naming Patterns
- Modules: `scraper`, `csv_handler`, `ui_widgets`
- Structs: `AppState`, `ListingData`, `ScrapingConfig`
- Functions: `update_inventory()`, `handle_key_event()`, `scrape_listings()`
- Constants: `DEFAULT_TIMEOUT`, `MAX_RETRIES`, `LOG_FORMAT`
- Error types: `ScrapingError`, `CsvError`, `ConfigError`

### Documentation Standards
- Document all public functions with `///` comments
- Include examples for complex functions
- Document error conditions and return values
- Add module-level documentation for each file
- Use `TODO:` and `FIXME:` comments for known issues

### Code Organization
- Keep functions under 50 lines when possible
- Group related functionality in modules
- Use type aliases for complex types
- Implement `Display` and `Debug` for custom types
- Prefer composition over inheritance

### Testing Approach
- Manual testing by running the app
- Verify CSV data accuracy after scraping
- Check logs for proper error handling
- Test UI responsiveness and key bindings

## Future Plans
- eBay seller account login integration
- Google Drive authentication and image management
- LLM integration for product analysis and listing generation  
- Management widget for creating listings from photos
- sold.csv tracking for historical data
- Automated listing creation and draft saving 
