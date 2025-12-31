#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    Size,
    ModifiedTime,
    ItemCount,
}

impl SortMode {
    #[allow(dead_code)] // May be used for cycling through modes
    pub fn next(&self) -> Self {
        match self {
            SortMode::Size => SortMode::ModifiedTime,
            SortMode::ModifiedTime => SortMode::ItemCount,
            SortMode::ItemCount => SortMode::Size,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            SortMode::Size => "size",
            SortMode::ModifiedTime => "mtime",
            SortMode::ItemCount => "count",
        }
    }
}