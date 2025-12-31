use number_prefix::NumberPrefix;
use std::path::Path;

#[cfg(not(windows))]
use std::fs;

pub fn format_size(size: u64) -> String {
    match NumberPrefix::binary(size as f64) {
        NumberPrefix::Standalone(bytes) => format!("{} B", bytes),
        NumberPrefix::Prefixed(prefix, n) => format!("{:.1} {}B", n, prefix),
    }
}

/// Render a progress bar using Unicode block characters (1/8 to 8/8 precision)
pub fn render_bar(percent: f64, width: usize) -> String {
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

/// Get the drive letter for a path (Windows-specific)
#[cfg(windows)]
pub fn get_drive_letter(path: &Path) -> Option<char> {
    use std::path::Component;
    if let Some(Component::Prefix(prefix)) = path.components().next() {
        prefix.as_os_str().to_str()?.chars().next()
    } else {
        None
    }
}

#[cfg(not(windows))]
pub fn get_volume_id(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(path).ok().map(|m| m.dev())
}

/// Get the number of CPUs for parallelism
pub fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
}