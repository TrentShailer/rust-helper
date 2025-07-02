//! Feature toggle for output style

#[cfg(feature = "styled")]
pub use styled::*;
#[cfg(not(feature = "styled"))]
pub use unstyled::*;

/// Make the first letter lowercase and remove any trailing punctuation.
pub fn normalize_error(message: &str) -> String {
    const ILLEGAL_LAST_CHARS: [char; 3] = ['.', '?', '!'];

    let mut chars = message.trim().chars();

    // Make first letter lowercase.
    let first_char = chars
        .next()
        .map_or_else(String::new, |v| v.to_lowercase().to_string());

    // Remove trailing punctuation.
    let last_char = if let Some(real_last_char) = chars.next_back()
        && !ILLEGAL_LAST_CHARS.contains(&real_last_char)
    {
        real_last_char.to_string()
    } else {
        String::new()
    };

    let middle = chars.as_str();

    format!("{first_char}{middle}{last_char}")
}

#[cfg(feature = "styled")]
mod styled {
    /// Reset styling
    pub const RESET: &str = "\x1b[0m";

    /// Following text will be bold
    pub const BOLD: &str = "\x1b[1m";
    /// Following text will NOT be bold
    pub const NO_BOLD: &str = "\x1b[21m";

    /// Following text will be dim
    pub const DIM: &str = "\x1b[2m";
    /// Following text will NOT be dim
    pub const NO_DIM: &str = "\x1b[22m";

    /// Following text will be italic
    pub const ITALIC: &str = "\x1b[3m";
    /// Following text will NOT be italic
    pub const NO_ITALIC: &str = "\x1b[23m";

    /// Following text will be underlined
    pub const UNDERLINE: &str = "\x1b[4m";
    /// Following text will NOT be underlined
    pub const NO_UNDERLINE: &str = "\x1b[24m";

    /// Following text will be blinking
    pub const BLINK: &str = "\x1b[5m";
    /// Following text will NOT be blinking
    pub const NO_BLINK: &str = "\x1b[25m";

    /// Foreground and background for the following text will be reversed
    pub const REVERSE: &str = "\x1b[7m";
    /// Foreground and background for the following text will NOT be reversed
    pub const NO_REVERSE: &str = "\x1b[27m";

    /// Following text will be invisible
    pub const HIDE: &str = "\x1b[8m";
    /// Following text will be visible
    pub const NO_HIDE: &str = "\x1b[28m";

    /// Following text will be crossed out
    pub const STRIKETHROUGH: &str = "\x1b[9m";
    /// Following text will NOT be crossed out
    pub const NO_STRIKETHROUGH: &str = "\x1b[29m";

    /// Set color of text to black
    pub const BLACK: &str = "\x1b[90m";
    /// Set background of text to black
    pub const BG_BLACK: &str = "\x1b[100m";
    /// Set color of text to dim black
    pub const DIM_BLACK: &str = "\x1b[30m";
    /// Set background of text to dim black
    pub const BG_DIM_BLACK: &str = "\x1b[40m";

    /// Set color of text to red
    pub const RED: &str = "\x1b[91m";
    /// Set background of text to red
    pub const BG_RED: &str = "\x1b[101m";
    /// Set color of text to dim red
    pub const DIM_RED: &str = "\x1b[31m";
    /// Set background of text to dim red
    pub const BG_DIM_RED: &str = "\x1b[41m";

    /// Set color of text to green
    pub const GREEN: &str = "\x1b[92m";
    /// Set background of text to green
    pub const BG_GREEN: &str = "\x1b[102m";
    /// Set color of text to dim green
    pub const DIM_GREEN: &str = "\x1b[32m";
    /// Set background of text to dim green
    pub const BG_DIM_GREEN: &str = "\x1b[42m";

    /// Set color of text to yellow
    pub const YELLOW: &str = "\x1b[93m";
    /// Set background of text to yellow
    pub const BG_YELLOW: &str = "\x1b[103m";
    /// Set color of text to dim yellow
    pub const DIM_YELLOW: &str = "\x1b[33m";
    /// Set background of text to dim yellow
    pub const BG_DIM_YELLOW: &str = "\x1b[43m";

    /// Set color of text to blue
    pub const BLUE: &str = "\x1b[94m";
    /// Set background of text to blue
    pub const BG_BLUE: &str = "\x1b[104m";
    /// Set color of text to dim blue
    pub const DIM_BLUE: &str = "\x1b[34m";
    /// Set background of text to dim blue
    pub const BG_DIM_BLUE: &str = "\x1b[44m";

    /// Set color of text to magenta
    pub const MAGENTA: &str = "\x1b[95m";
    /// Set background of text to magenta
    pub const BG_MAGENTA: &str = "\x1b[105m";
    /// Set color of text to dim magenta
    pub const DIM_MAGENTA: &str = "\x1b[35m";
    /// Set background of text to dim magenta
    pub const BG_DIM_MAGENTA: &str = "\x1b[45m";

    /// Set color of text to cyan
    pub const CYAN: &str = "\x1b[96m";
    /// Set background of text to cyan
    pub const BG_CYAN: &str = "\x1b[106m";
    /// Set color of text to dim cyan
    pub const DIM_CYAN: &str = "\x1b[36m";
    /// Set background of text to dim cyan
    pub const BG_DIM_CYAN: &str = "\x1b[46m";

    /// Set color of text to white
    pub const WHITE: &str = "\x1b[97m";
    /// Set background of text to white
    pub const BG_WHITE: &str = "\x1b[107m";
    /// Set color of text to dim white
    pub const DIM_WHITE: &str = "\x1b[37m";
    /// Set background of text to dim white
    pub const BG_DIM_WHITE: &str = "\x1b[47m";

    /// Set color of text to default
    pub const DEFAULT: &str = "\x1b[99m";
    /// Set background of text to default
    pub const BG_DEFAULT: &str = "\x1b[109m";
    /// Set color of text to default
    pub const DIM_DEFAULT: &str = "\x1b[39m";
    /// Set background of text to default
    pub const BG_DIM_DEFAULT: &str = "\x1b[49m";
}

#[cfg(not(feature = "styled"))]
mod unstyled {
    /// Reset styling
    pub const RESET: &str = "";

    /// Following text will be bold
    pub const BOLD: &str = "";
    /// Following text will NOT be bold
    pub const NO_BOLD: &str = "";

    /// Following text will be dim
    pub const DIM: &str = "";
    /// Following text will NOT be dim
    pub const NO_DIM: &str = "";

    /// Following text will be italic
    pub const ITALIC: &str = "";
    /// Following text will NOT be italic
    pub const NO_ITALIC: &str = "";

    /// Following text will be underlined
    pub const UNDERLINE: &str = "";
    /// Following text will NOT be underlined
    pub const NO_UNDERLINE: &str = "";

    /// Following text will be blinking
    pub const BLINK: &str = "";
    /// Following text will NOT be blinking
    pub const NO_BLINK: &str = "";

    /// Foreground and background for the following text will be reversed
    pub const REVERSE: &str = "";
    /// Foreground and background for the following text will NOT be reversed
    pub const NO_REVERSE: &str = "";

    /// Following text will be invisible
    pub const HIDE: &str = "";
    /// Following text will be visible
    pub const NO_HIDE: &str = "";

    /// Following text will be crossed out
    pub const STRIKETHROUGH: &str = "";
    /// Following text will NOT be crossed out
    pub const NO_STRIKETHROUGH: &str = "";

    /// Set color of text to black
    pub const BLACK: &str = "";
    /// Set background of text to black
    pub const BG_BLACK: &str = "";
    /// Set color of text to dim black
    pub const DIM_BLACK: &str = "";
    /// Set background of text to dim black
    pub const BG_DIM_BLACK: &str = "";

    /// Set color of text to red
    pub const RED: &str = "";
    /// Set background of text to red
    pub const BG_RED: &str = "";
    /// Set color of text to dim red
    pub const DIM_RED: &str = "";
    /// Set background of text to dim red
    pub const BG_DIM_RED: &str = "";

    /// Set color of text to green
    pub const GREEN: &str = "";
    /// Set background of text to green
    pub const BG_GREEN: &str = "";
    /// Set color of text to dim green
    pub const DIM_GREEN: &str = "";
    /// Set background of text to dim green
    pub const BG_DIM_GREEN: &str = "";

    /// Set color of text to yellow
    pub const YELLOW: &str = "";
    /// Set background of text to yellow
    pub const BG_YELLOW: &str = "";
    /// Set color of text to dim yellow
    pub const DIM_YELLOW: &str = "";
    /// Set background of text to dim yellow
    pub const BG_DIM_YELLOW: &str = "";

    /// Set color of text to blue
    pub const BLUE: &str = "";
    /// Set background of text to blue
    pub const BG_BLUE: &str = "";
    /// Set color of text to dim blue
    pub const DIM_BLUE: &str = "";
    /// Set background of text to dim blue
    pub const BG_DIM_BLUE: &str = "";

    /// Set color of text to magenta
    pub const MAGENTA: &str = "";
    /// Set background of text to magenta
    pub const BG_MAGENTA: &str = "";
    /// Set color of text to dim magenta
    pub const DIM_MAGENTA: &str = "";
    /// Set background of text to dim magenta
    pub const BG_DIM_MAGENTA: &str = "";

    /// Set color of text to cyan
    pub const CYAN: &str = "";
    /// Set background of text to cyan
    pub const BG_CYAN: &str = "";
    /// Set color of text to dim cyan
    pub const DIM_CYAN: &str = "";
    /// Set background of text to dim cyan
    pub const BG_DIM_CYAN: &str = "";

    /// Set color of text to white
    pub const WHITE: &str = "";
    /// Set background of text to white
    pub const BG_WHITE: &str = "";
    /// Set color of text to dim white
    pub const DIM_WHITE: &str = "";
    /// Set background of text to dim white
    pub const BG_DIM_WHITE: &str = "";

    /// Set color of text to default
    pub const DEFAULT: &str = "";
    /// Set background of text to default
    pub const BG_DEFAULT: &str = "";
    /// Set color of text to default
    pub const DIM_DEFAULT: &str = "";
    /// Set background of text to default
    pub const BG_DIM_DEFAULT: &str = "";
}
