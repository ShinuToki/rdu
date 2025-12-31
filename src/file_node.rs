use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::SystemTime,
};

/// Represents a file or directory
#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub children: Vec<Rc<RefCell<FileNode>>>,
    pub error_count: usize,
    pub modified_time: Option<SystemTime>,
}

impl FileNode {
    pub fn new(path: PathBuf, name: String, size: u64, is_dir: bool, mtime: Option<SystemTime>) -> Self {
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
    
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}