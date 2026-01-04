use crate::{
    args::Args,
    file_node::FileNode,
    scanner::scan_dir,
    sort::SortMode,
};
use ratatui::widgets::ListState;
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
};

/// Application State
pub struct App {
    #[allow(dead_code)] // Kept for potential navigation reset feature
    pub root: Rc<RefCell<FileNode>>,
    pub current_node: Rc<RefCell<FileNode>>,
    pub path_history: Vec<Rc<RefCell<FileNode>>>,
    pub state: ListState,
    pub args: Args,
    pub status_message: Option<String>,
    pub show_help: bool,
    pub sort_mode: SortMode,
    pub sort_ascending: bool,
}

impl App {
    pub fn new(root: Rc<RefCell<FileNode>>, args: Args) -> Self {
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

    pub fn sort_current_view(&mut self) {
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

    pub fn toggle_sort_by_size(&mut self) {
        if self.sort_mode == SortMode::Size {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_mode = SortMode::Size;
            self.sort_ascending = false;
        }
        self.sort_current_view();
        self.status_message = Some(format!("Sort: {} {}", self.sort_mode.name(), if self.sort_ascending { "asc" } else { "desc" }));
    }

    pub fn toggle_sort_by_mtime(&mut self) {
        if self.sort_mode == SortMode::ModifiedTime {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_mode = SortMode::ModifiedTime;
            self.sort_ascending = false;
        }
        self.sort_current_view();
        self.status_message = Some(format!("Sort: {} {}", self.sort_mode.name(), if self.sort_ascending { "asc" } else { "desc" }));
    }

    pub fn toggle_sort_by_count(&mut self) {
        if self.sort_mode == SortMode::ItemCount {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_mode = SortMode::ItemCount;
            self.sort_ascending = false;
        }
        self.sort_current_view();
        self.status_message = Some(format!("Sort: {} {}", self.sort_mode.name(), if self.sort_ascending { "asc" } else { "desc" }));
    }

    pub fn current_children(&self) -> Vec<Rc<RefCell<FileNode>>> {
        self.current_node.borrow().children.clone()
    }

    pub fn current_path(&self) -> PathBuf {
        self.current_node.borrow().path.clone()
    }

    pub fn current_total_size(&self) -> u64 {
        self.current_node.borrow().children.iter()
            .map(|c| c.borrow().size)
            .sum()
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn page_down(&mut self) {
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

    pub fn page_up(&mut self) {
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

    pub fn go_to_first(&mut self) {
        let children = self.current_children();
        if !children.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn go_to_last(&mut self) {
        let children = self.current_children();
        if !children.is_empty() {
            self.state.select(Some(children.len() - 1));
        }
    }

    /// Enter the selected directory
    pub fn enter_dir(&mut self) {
        let children = self.current_children();
        if let Some(selected_idx) = self.state.selected()
            && selected_idx < children.len()
                && let Some(child) = children.get(selected_idx) {
                    let selected = Rc::clone(child);
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
    pub fn go_up(&mut self) {
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
    pub fn refresh(&mut self) {
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