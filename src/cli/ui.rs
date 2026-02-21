use crate::cli::app::{App, Suggestion};
use crate::cli::theme::{BG, BORDER_ACTIVE, FG, SELECTION_BG, SELECTION_FG};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};
use crate::config::CONFIG; // Import CONFIG

pub fn ui(f: &mut Frame, app: &App) {
    let area = f.area(); // Corrected: f.size() is deprecated, use f.area()

    // Define a centered area for the launcher
    let launcher_width = CONFIG.ui.launcher_width;
    let suggestions_to_display = (app.suggestions.len() as u16).min(CONFIG.ui.max_suggestions_to_display);
    let launcher_height = 3 + suggestions_to_display; // 1 for border, 1 for input, 1 for border, + suggestions

    let area = centered_rect(launcher_width, launcher_height, area);

    // Clear the area before drawing to handle dynamic height
    f.render_widget(Clear, area);

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_ACTIVE))
        .style(Style::default().bg(BG));

    let inner_area = main_block.inner(area);
    f.render_widget(main_block, area);
    // Rest of the code ...


    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(inner_area);

    // --- Input Line ---
    let input_line = Line::from(vec![
        Span::styled("❯ ", Style::default().fg(BORDER_ACTIVE)),
        Span::styled(app.input.as_str(), Style::default().fg(FG)),
    ]);
    let input_paragraph = Paragraph::new(input_line);
    f.render_widget(input_paragraph, chunks[0]);

    // --- Suggestions List ---
    let suggestions: Vec<ListItem> = app
        .suggestions
        .iter()
        .enumerate()
        .map(|(i, suggestion)| {
            // Corrected: Ensure both match arms return the same type (&str, &str)
            let (icon, text): (&str, &str) = match suggestion {
                Suggestion::App(app) => (app.icon.as_str(), app.name.as_str()),
                Suggestion::Calc(res) => ("", res.as_str()),
            };

            let content = Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(SELECTION_FG)),
                // Corrected: `text` is now a `&str`, no clone needed.
                Span::raw(text),
            ]);

            let mut list_item = ListItem::new(content).style(Style::default().fg(FG));

            if i == app.selected_index {
                list_item = list_item.style(
                    Style::default()
                        .bg(SELECTION_BG)
                        .fg(FG)
                        .add_modifier(Modifier::BOLD),
                );
            }
            list_item
        })
        .collect();

    let suggestions_list = List::new(suggestions);
    f.render_widget(suggestions_list, chunks[1]);
}

/// Helper function to create a centered rect with a max width and dynamic height
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15), // 15% from top
            Constraint::Min(height),
            Constraint::Max(
                r.height
                    .saturating_sub(height)
                    .saturating_sub(r.height / 10),
            ), // flexible bottom margin
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
