use clap::Parser;
use std::path::PathBuf;

/// RDU: A Rust-based Disk Usage analyzer for Windows
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory to scan (default: current)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Do not cross filesystem boundaries (drives on Windows)
    #[arg(short = 'x', long)]
    pub one_file_system: bool,

    /// Follow symbolic links and Junction points (Caution: can cause loops)
    #[arg(short = 'L', long)]
    pub follow_links: bool,
}