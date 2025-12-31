use ratatui::style::Color;

// Color Theme Constants
pub const COLOR_HEADER_BG: Color = Color::Rgb(222, 222, 222);     // Light gray background
pub const COLOR_HEADER_FG: Color = Color::Rgb(0, 0, 0);           // Dark text
pub const COLOR_DIR_INFO: Color = Color::Rgb(0, 255, 255);        // Bright cyan
pub const COLOR_SIZE: Color = Color::Rgb(78, 154, 6);             // Green for sizes
pub const COLOR_PERCENT: Color = Color::Rgb(255, 255, 255);       // White for percentages
pub const COLOR_DIRECTORY: Color = Color::Rgb(0, 220, 255);       // Bright cyan for dirs
pub const COLOR_FILE: Color = Color::Rgb(220, 220, 220);          // Light gray for files
pub const COLOR_HELP_TITLE: Color = Color::Rgb(0, 255, 255);      // Bright cyan
pub const COLOR_HELP_HEADER: Color = Color::Rgb(255, 220, 0);     // Vibrant yellow
pub const COLOR_HELP_HINT: Color = Color::Rgb(128, 128, 128);     // Gray
pub const COLOR_HIGHLIGHT_BG: Color = Color::Rgb(255, 255, 255);  // White background when selected
pub const COLOR_HIGHLIGHT_FG: Color = Color::Rgb(40, 40, 40);     // Dark gray text when selected (matches terminal bg)