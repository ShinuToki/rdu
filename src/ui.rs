use crate::{
    app::App,
    colors::*,
    utils::{format_size, render_bar},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Length(1), // Directory info
            Constraint::Min(0),    // List
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    let [title_area, dir_info_area, list_area, footer_area] = *chunks else {
        return;
    };

    render_title_bar(f, title_area);
    render_directory_info(f, app, dir_info_area);
    render_file_list(f, app, list_area);
    render_footer(f, app, footer_area);

    if app.show_help {
        render_help_overlay(f);
    }
}

fn render_title_bar(f: &mut Frame, area: ratatui::layout::Rect) {
    let version = env!("CARGO_PKG_VERSION");
    let terminal_width = f.area().width as usize;
    
    // Calculate padding: " rdu vX.X.X (press ? for help)" = 1 + 3 + 2 + version.len + 8 + 1 + 10 + 1
    let title_len = 1 + 3 + 2 + version.len() + 8 + 1 + 10 + 1; // " rdu vX.X.X (press ? for help)"
    let padding = terminal_width.saturating_sub(title_len);
    
    let title_bar = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("rdu", Style::default().fg(COLOR_HEADER_FG).add_modifier(Modifier::BOLD)),
        Span::raw(format!(" v{}    (press ", version)),
        Span::styled("?", Style::default().fg(COLOR_HEADER_FG).add_modifier(Modifier::BOLD)),
        Span::raw(" for help)"),
        Span::raw(" ".repeat(padding)),
    ]))
    .style(Style::default().fg(COLOR_HEADER_FG).bg(COLOR_HEADER_BG));
    f.render_widget(title_bar, area);
}

fn render_directory_info(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let children = app.current_children();
    let item_count = children.len();
    let current_size = format_size(app.current_total_size());
    let current_path = app.current_path();
    
    let dir_info = format!(" {} ({} visible, {})", 
        current_path.display(), 
        item_count, 
        current_size
    );
    let dir_line = Paragraph::new(Line::from(vec![
        Span::styled(dir_info, Style::default().fg(COLOR_DIR_INFO)),
    ]))
    .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    f.render_widget(dir_line, area);
}

fn render_file_list(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let children = app.current_children();
    let parent_size = app.current_total_size();
    
    let items: Vec<ListItem> = children
        .iter()
        .map(|node_rc| {
            let node = node_rc.borrow();
            let size_str = format_size(node.size);
            let name = &node.name;
            let percent = if parent_size > 0 {
                (node.size as f64 / parent_size as f64) * 100.0
            } else {
                0.0
            };

            // Create bar graph using fractional block characters
            let bar = render_bar(percent, 10);
            
            // Prefix: / for directories, space for files
            let prefix = if node.is_dir { "/" } else { " " };
            let name_color = if node.is_dir { COLOR_DIRECTORY } else { COLOR_FILE };

            // Multi-colored line: olive size | white percent | bar | colored name
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>10}", size_str), Style::default().fg(COLOR_SIZE)),
                Span::raw(" | "),
                Span::styled(format!("{:>5.1}%", percent), Style::default().fg(COLOR_PERCENT)),
                Span::raw(" | "),
                Span::styled(format!("{:10}", bar), Style::default().fg(COLOR_PERCENT)),
                Span::raw(" | "),
                Span::styled(format!("{}{}", prefix, name), Style::default().fg(name_color)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM))
        .highlight_style(Style::default().bg(COLOR_HIGHLIGHT_BG).fg(COLOR_HIGHLIGHT_FG));

    f.render_stateful_widget(list, area, &mut app.state);
}

fn render_footer(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let terminal_width = f.area().width as usize;
    let status_msg = app.status_message.as_deref().unwrap_or("");
    let sort_order = if app.sort_ascending { "ascending" } else { "descending" };
    let current_size = format_size(app.current_total_size());
    let footer_left = format!("Sort mode: {} {}  Total disk usage: {}", 
        app.sort_mode.name(), sort_order, current_size);
    let footer_right = if !status_msg.is_empty() {
        format!("  {}", status_msg)
    } else {
        String::new()
    };
    let footer_padding = terminal_width.saturating_sub(footer_left.len() + footer_right.len());
    let footer_text = format!("{}{:padding$}{}", footer_left, "", footer_right, padding = footer_padding);
    
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(COLOR_HEADER_FG).bg(COLOR_HEADER_BG));
    f.render_widget(footer, area);
}

fn render_help_overlay(f: &mut Frame) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("  rdu - Rust Disk Usage Analyzer", Style::default().fg(COLOR_HELP_TITLE).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("  Navigation:", Style::default().fg(COLOR_HELP_HEADER).add_modifier(Modifier::BOLD))),
        Line::from("    j / ↓           Move down 1 item"),
        Line::from("    k / ↑           Move up 1 item"),
        Line::from("    Ctrl+d / PgDn   Move down 10 items"),
        Line::from("    Ctrl+u / PgUp   Move up 10 items"),
        Line::from("    H / Home        Go to first item"),
        Line::from("    G / End         Go to last item"),
        Line::from(""),
        Line::from(Span::styled("  Actions:", Style::default().fg(COLOR_HELP_HEADER).add_modifier(Modifier::BOLD))),
        Line::from("    o / l / Enter   Enter directory"),
        Line::from("    u / h / Bksp    Go up one level"),
        Line::from("    r               Refresh current view"),
        Line::from(""),
        Line::from(Span::styled("  Display:", Style::default().fg(COLOR_HELP_HEADER).add_modifier(Modifier::BOLD))),
        Line::from("    s               Toggle sort by size"),
        Line::from("    m               Toggle sort by mtime"),
        Line::from("    c               Toggle sort by count"),
        Line::from(""),
        Line::from(Span::styled("  Other:", Style::default().fg(COLOR_HELP_HEADER).add_modifier(Modifier::BOLD))),
        Line::from("    ?               Toggle this help"),
        Line::from("    q / Esc         Quit"),
        Line::from(""),
        Line::from(Span::styled("  Press any key to close", Style::default().fg(COLOR_HELP_HINT))),
        Line::from(""),
    ];

    let help_height = help_text.len() as u16 + 2;
    let help_width = 42;
    let area = f.area();
    let help_area = ratatui::layout::Rect {
        x: area.width.saturating_sub(help_width) / 2,
        y: area.height.saturating_sub(help_height) / 2,
        width: help_width.min(area.width),
        height: help_height.min(area.height),
    };

    f.render_widget(Clear, help_area);
    let help_block = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(help_block, help_area);
}