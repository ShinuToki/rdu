use crate::{args::Args, file_node::FileNode, utils::num_cpus};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    time::SystemTime,
};

#[cfg(windows)]
use crate::utils::get_drive_letter;

#[cfg(not(windows))]
use crate::utils::get_volume_id;

/// Parallel directory scanner using jwalk
pub fn scan_dir(path: &Path, args: &Args) -> Rc<RefCell<FileNode>> {
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
        if *is_dir
            && let Some(node) = nodes.get(entry_path) {
                let dir_size = node.borrow().size;
                if let Some(parent_path) = entry_path.parent()
                    && let Some(parent_node) = nodes.get(parent_path) {
                        parent_node.borrow_mut().size += dir_size;
                    }
            }
    }
    
    root_node
}