use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use number_prefix::NumberPrefix;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{
    cell::RefCell,
    fs,
    io,
    path::{Path, PathBuf},
    rc::Rc,
    time::SystemTime,
};

// =============================================================================
// Color Theme Constants
// =============================================================================
const COLOR_HEADER_BG: Color = Color::Rgb(222, 222, 222);     // Light gray background
const COLOR_HEADER_FG: Color = Color::Rgb(0, 0, 0);           // Dark text
const COLOR_DIR_INFO: Color = Color::Rgb(0, 255, 255);        // Bright cyan
const COLOR_SIZE: Color = Color::Rgb(78, 154, 6);             // Green for sizes
const COLOR_PERCENT: Color = Color::Rgb(255, 255, 255);       // White for percentages
const COLOR_DIRECTORY: Color = Color::Rgb(0, 220, 255);       // Bright cyan for dirs
const COLOR_FILE: Color = Color::Rgb(220, 220, 220);          // Light gray for files
const COLOR_HELP_TITLE: Color = Color::Rgb(0, 255, 255);      // Bright cyan
const COLOR_HELP_HEADER: Color = Color::Rgb(255, 220, 0);     // Vibrant yellow
const COLOR_HELP_HINT: Color = Color::Rgb(128, 128, 128);     // Gray
const COLOR_HIGHLIGHT_BG: Color = Color::Rgb(255, 255, 255);  // White background when selected
const COLOR_HIGHLIGHT_FG: Color = Color::Rgb(40, 40, 40);     // Dark gray text when selected (matches terminal bg)

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortMode {
    Size,
    ModifiedTime,
    ItemCount,
}

impl SortMode {
    #[allow(dead_code)] // May be used for cycling through modes
    fn next(&self) -> Self {
        match self {
            SortMode::Size => SortMode::ModifiedTime,
            SortMode::ModifiedTime => SortMode::ItemCount,
            SortMode::ItemCount => SortMode::Size,
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            SortMode::Size => "size",
            SortMode::ModifiedTime => "mtime",
            SortMode::ItemCount => "count",
        }
    }
}


/// RDU: A Rust-based Disk Usage analyzer for Windows
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan (default: current)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Do not cross filesystem boundaries (drives on Windows)
    #[arg(short = 'x', long)]
    one_file_system: bool,

    /// Follow symbolic links and Junction points (Caution: can cause loops)
    #[arg(short = 'L', long)]
    follow_links: bool,
}

/// Represents a file or directory
#[derive(Debug, Clone)]
struct FileNode {
    name: String,
    path: PathBuf,
    size: u64,
    is_dir: bool,
    children: Vec<Rc<RefCell<FileNode>>>,
    error_count: usize,
    modified_time: Option<SystemTime>,
}

impl FileNode {
    fn new(path: PathBuf, name: String, size: u64, is_dir: bool, mtime: Option<SystemTime>) -> Self {
        Self {
            name,
            path,
            size,
            is_dir,
            children: vec![],
            error_count: 0,
            modified_time: mtime,
        }
    }
    
    fn child_count(&self) -> usize {
        self.children.len()
    }
}

/// Application State
struct App {
    #[allow(dead_code)] // Kept for potential navigation reset feature
    root: Rc<RefCell<FileNode>>,
    current_node: Rc<RefCell<FileNode>>,
    path_history: Vec<Rc<RefCell<FileNode>>>,
    state: ListState,
    args: Args,
    status_message: Option<String>,
    show_help: bool,
    sort_mode: SortMode,
    sort_ascending: bool,
}

impl App {
    fn new(root: Rc<RefCell<FileNode>>, args: Args) -> Self {
        let current_node = Rc::clone(&root);
        let mut app = Self {
            root,
            current_node,
            path_history: Vec::new(),
            state: ListState::default(),
            args,
            status_message: None,
            show_help: false,
            sort_mode: SortMode::Size,
            sort_ascending: false,
        };
        app.sort_current_view();
        let has_children = !app.current_node.borrow().children.is_empty();
        if has_children {
            app.state.select(Some(0));
        }
        app
    }

    fn sort_current_view(&mut self) {
        let sort_mode = self.sort_mode;
        let ascending = self.sort_ascending;
        let mut node = self.current_node.borrow_mut();
        node.children.sort_by(|a, b| {
            let a = a.borrow();
            let b = b.borrow();
            let cmp = match sort_mode {
                SortMode::Size => a.size.cmp(&b.size),
                SortMode::ModifiedTime => a.modified_time.cmp(&b.modified_time),
                SortMode::ItemCount => a.child_count().cmp(&b.child_count()),
            };
            if ascending { cmp } else { cmp.reverse() }
        });
    }

    fn toggle_sort_by_size(&mut self) {
        if self.sort_mode == SortMode::Size {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_mode = SortMode::Size;
            self.sort_ascending = false;
        }
        self.sort_current_view();
        self.status_message = Some(format!("Sort: {} {}", self.sort_mode.name(), if self.sort_ascending { "asc" } else { "desc" }));
    }

    fn toggle_sort_by_mtime(&mut self) {
        if self.sort_mode == SortMode::ModifiedTime {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_mode = SortMode::ModifiedTime;
            self.sort_ascending = false;
        }
        self.sort_current_view();
        self.status_message = Some(format!("Sort: {} {}", self.sort_mode.name(), if self.sort_ascending { "asc" } else { "desc" }));
    }

    fn toggle_sort_by_count(&mut self) {
        if self.sort_mode == SortMode::ItemCount {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_mode = SortMode::ItemCount;
            self.sort_ascending = false;
        }
        self.sort_current_view();
        self.status_message = Some(format!("Sort: {} {}", self.sort_mode.name(), if self.sort_ascending { "asc" } else { "desc" }));
    }

    fn current_children(&self) -> Vec<Rc<RefCell<FileNode>>> {
        self.current_node.borrow().children.clone()
    }

    fn current_path(&self) -> PathBuf {
        self.current_node.borrow().path.clone()
    }

    fn current_total_size(&self) -> u64 {
        self.current_node.borrow().children.iter()
            .map(|c| c.borrow().size)
            .sum()
    }

    fn next(&mut self) {
        let children = self.current_children();
        let i = match self.state.selected() {
            Some(i) => {
                if !children.is_empty() && i >= children.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        if !children.is_empty() {
            self.state.select(Some(i));
        }
    }

    fn previous(&mut self) {
        let children = self.current_children();
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    if !children.is_empty() {
                        children.len() - 1
                    } else {
                        0
                    }
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        if !children.is_empty() {
            self.state.select(Some(i));
        }
    }

    fn page_down(&mut self) {
        let children = self.current_children();
        if children.is_empty() {
            return;
        }
        let page_size = 10;
        let i = match self.state.selected() {
            Some(i) => (i + page_size).min(children.len() - 1),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn page_up(&mut self) {
        let children = self.current_children();
        if children.is_empty() {
            return;
        }
        let page_size = 10;
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(page_size),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn go_to_first(&mut self) {
        let children = self.current_children();
        if !children.is_empty() {
            self.state.select(Some(0));
        }
    }

    fn go_to_last(&mut self) {
        let children = self.current_children();
        if !children.is_empty() {
            self.state.select(Some(children.len() - 1));
        }
    }

    /// Enter the selected directory
    fn enter_dir(&mut self) {
        let children = self.current_children();
        if let Some(selected_idx) = self.state.selected()
            && selected_idx < children.len() {
                let selected = Rc::clone(&children[selected_idx]);
                if selected.borrow().is_dir {
                    self.path_history.push(Rc::clone(&self.current_node));
                    self.current_node = selected;
                    self.sort_current_view();
                    let new_children = self.current_children();
                    if new_children.is_empty() {
                        self.state.select(None);
                    } else {
                        self.state.select(Some(0));
                    }
                }
            }
    }

    /// Go up one level
    fn go_up(&mut self) {
        if let Some(parent) = self.path_history.pop() {
            self.current_node = parent;
            self.sort_current_view();
            let children = self.current_children();
            if children.is_empty() {
                self.state.select(None);
            } else {
                self.state.select(Some(0));
            }
        }
    }

    /// Refresh the current directory by rescanning
    fn refresh(&mut self) {
        self.status_message = Some("Rescanning...".to_string());
        let path = self.current_path();
        let new_node = scan_dir(&path, &self.args);
        
        // Update current node's children
        let mut current = self.current_node.borrow_mut();
        current.children = new_node.borrow().children.clone();
        current.size = new_node.borrow().size;
        current.error_count = new_node.borrow().error_count;
        drop(current);
        
        self.sort_current_view();
        let children = self.current_children();
        if children.is_empty() {
            self.state.select(None);
        } else {
            self.state.select(Some(0));
        }
        self.status_message = Some("Refresh complete!".to_string());
    }

}

/// Get the drive letter for a path (Windows-specific)
#[cfg(windows)]
fn get_drive_letter(path: &Path) -> Option<char> {
    use std::path::Component;
    if let Some(Component::Prefix(prefix)) = path.components().next() {
        prefix.as_os_str().to_str()?.chars().next()
    } else {
        None
    }
}

#[cfg(not(windows))]
fn get_volume_id(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(path).ok().map(|m| m.dev())
}

/// Parallel directory scanner using jwalk
fn scan_dir(path: &Path, args: &Args) -> Rc<RefCell<FileNode>> {
    use std::collections::HashMap;
    use jwalk::WalkDir;
    
    let root_path = path.to_path_buf();
    let mtime = fs::metadata(&root_path).ok().and_then(|m| m.modified().ok());
    let root_name = root_path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    // Configure jwalk walker
    let walker = WalkDir::new(&root_path)
        .follow_links(args.follow_links)
        .skip_hidden(false)
        .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus()));
    
    // Collect all entries in parallel
    let mut entries: Vec<(PathBuf, u64, bool, Option<SystemTime>)> = Vec::new();
    let mut error_count = 0usize;
    
    for entry_result in walker {
        match entry_result {
            Ok(entry) => {
                let entry_path = entry.path();
                
                // Skip the root itself
                if entry_path == root_path {
                    continue;
                }
                
                // One-file-system check
                #[cfg(windows)]
                if args.one_file_system
                    && let (Some(root_drive), Some(entry_drive)) = (
                        get_drive_letter(&root_path),
                        get_drive_letter(&entry_path)
                    )
                        && root_drive != entry_drive {
                            continue;
                        }
                
                #[cfg(not(windows))]
                if args.one_file_system {
                    if let (Some(root_vol), Some(entry_vol)) = (
                        get_volume_id(&root_path),
                        get_volume_id(&entry_path)
                    ) {
                        if root_vol != entry_vol {
                            continue;
                        }
                    }
                }
                
                let meta = if args.follow_links {
                    fs::metadata(&entry_path)
                } else {
                    fs::symlink_metadata(&entry_path)
                };
                
                match meta {
                    Ok(m) => {
                        let size = if m.is_file() { m.len() } else { 0 };
                        let mtime = m.modified().ok();
                        entries.push((entry_path.to_path_buf(), size, m.is_dir(), mtime));
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("Warning: Could not access {:?}: {}", entry_path, e);
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Warning: Walk error: {}", e);
            }
        }
    }
    
    // Build tree structure from flat entries
    let mut nodes: HashMap<PathBuf, Rc<RefCell<FileNode>>> = HashMap::new();
    
    // Create root node
    let root_node = Rc::new(RefCell::new(FileNode::new(
        root_path.clone(),
        root_name,
        0,
        true,
        mtime,
    )));
    root_node.borrow_mut().error_count = error_count;
    nodes.insert(root_path.clone(), Rc::clone(&root_node));
    
    // Sort entries by path depth (parents before children)
    let mut sorted_entries = entries;
    sorted_entries.sort_by(|a, b| a.0.components().count().cmp(&b.0.components().count()));
    
    // Create all nodes and link children to parents
    for (entry_path, size, is_dir, mtime) in &sorted_entries {
        let name = entry_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let node = Rc::new(RefCell::new(FileNode::new(
            entry_path.clone(),
            name,
            *size,
            *is_dir,
            *mtime,
        )));
        nodes.insert(entry_path.clone(), Rc::clone(&node));
        
        // Add to parent (but don't update size yet for directories)
        if let Some(parent_path) = entry_path.parent()
            && let Some(parent_node) = nodes.get(parent_path) {
                parent_node.borrow_mut().children.push(Rc::clone(&node));
                // Only add file sizes directly - directory sizes will be propagated later
                if !*is_dir {
                    parent_node.borrow_mut().size += size;
                }
            }
    }
    
    // Propagate directory sizes from deepest to shallowest
    for (entry_path, _, is_dir, _) in sorted_entries.iter().rev() {
        if *is_dir {
            if let Some(node) = nodes.get(entry_path) {
                let dir_size = node.borrow().size;
                if let Some(parent_path) = entry_path.parent() {
                    if let Some(parent_node) = nodes.get(parent_path) {
                        parent_node.borrow_mut().size += dir_size;
                    }
                }
            }
        }
    }
    
    root_node
}

/// Get the number of CPUs for parallelism
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
}

fn format_size(size: u64) -> String {
    match NumberPrefix::binary(size as f64) {
        NumberPrefix::Standalone(bytes) => format!("{} B", bytes),
        NumberPrefix::Prefixed(prefix, n) => format!("{:.1} {}B", n, prefix),
    }
}

/// Render a progress bar using Unicode block characters (1/8 to 8/8 precision)
fn render_bar(percent: f64, width: usize) -> String {
    const PARTIAL_CHARS: [char; 7] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉'];
    
    let fraction = percent / 100.0 * width as f64;
    let full_blocks = fraction.floor() as usize;
    let partial = ((fraction - full_blocks as f64) * 8.0).round() as usize;
    
    let mut bar = "█".repeat(full_blocks.min(width));
    if full_blocks < width && partial > 0 {
        bar.push(PARTIAL_CHARS[(partial - 1).min(6)]);
    }
    bar
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press {
                // Clear status message on any key press
                app.status_message = None;
                
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Esc, _) if !app.show_help => return Ok(()),
                    (KeyCode::Esc, _) => app.show_help = false,
                    (KeyCode::Char('?'), _) => app.show_help = !app.show_help,
                    _ if app.show_help => app.show_help = false, // Any key closes help
                    // Navigation
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.next(),
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.previous(),
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => app.page_down(),
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => app.page_up(),
                    (KeyCode::Char('H'), _) | (KeyCode::Home, _) => app.go_to_first(),
                    (KeyCode::Char('G'), _) | (KeyCode::End, _) => app.go_to_last(),
                    // Actions
                    (KeyCode::Enter, _) | (KeyCode::Right, _) | (KeyCode::Char('l'), _) | (KeyCode::Char('o'), _) => app.enter_dir(),
                    (KeyCode::Backspace, _) | (KeyCode::Left, _) | (KeyCode::Char('h'), _) | (KeyCode::Char('u'), _) => app.go_up(),
                    (KeyCode::Char('r'), _) => app.refresh(),
                    // Sort options
                    (KeyCode::Char('s'), _) => app.toggle_sort_by_size(),
                    (KeyCode::Char('m'), _) => app.toggle_sort_by_mtime(),
                    (KeyCode::Char('c'), _) => app.toggle_sort_by_count(),
                    _ => {}
                }
            }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Length(1), // Directory info
            Constraint::Min(0),    // List
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    // TITLE BAR
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
    f.render_widget(title_bar, chunks[0]);

    // DIRECTORY INFO LINE (dua style: /path (N visible, SIZE))
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
    f.render_widget(dir_line, chunks[1]);

    // LIST (dua style - no borders)
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

    f.render_stateful_widget(list, chunks[2], &mut app.state);

    // FOOTER (dua style)
    let status_msg = app.status_message.as_deref().unwrap_or("");
    let sort_order = if app.sort_ascending { "ascending" } else { "descending" };
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
    f.render_widget(footer, chunks[3]);

    // HELP OVERLAY
    if app.show_help {
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
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Scanning {}... This may take a moment.", args.path.display());

    let root_node = scan_dir(&args.path, &args);

    // Setup panic hook before entering raw mode
    setup_panic_hook();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(root_node, args);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}