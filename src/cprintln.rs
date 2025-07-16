//! cprintln! uses a Markdown-esque format to quickly format text for
//! printing.  It is not Markdown, just Markdown-like in structure.
//!
//! The [text](url) link syntax is used not for links but to format sections
//! of text, for example with a hex color.
//!

use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

//===========================================================================//
// cprintln! macro
//===========================================================================//

#[macro_export]
macro_rules! cprintln {
    ($color:expr, $($arg:tt)*) => {
        $crate::cprintln::cprintln_imp($color, &format!($($arg)*))
    };
}
pub use cprintln;

#[macro_export]
macro_rules! cprint {
    ($color:expr, $($arg:tt)*) => {
        $crate::util::cprintln::cprint_imp($color, &format!($($arg)*))
    };
}
pub use cprint;

//===========================================================================//
// cprintln implementation
//===========================================================================//

/// Parse Markdown and apply color formatting to links while treating everything else as plain text
pub fn cprintln_imp(color: &str, msg: &str) {
    cprint_imp(color, msg);
    println!();
}

pub fn cprint_imp(color: &str, msg: &str) {
    let base_color = parse_color(color);
    let (msg, trailing_ws) = {
        let trimmed = msg.trim_end_matches(|c: char| c == ' ' || c == '\t');
        let ws = &msg[trimmed.len()..];
        (trimmed, ws)
    };
    let (msg, leading_ws) = if !msg.contains('\n') {
        let trimmed = msg.trim_start_matches(|c: char| c == ' ' || c == '\t');
        let ws = &msg[..msg.len() - trimmed.len()];
        (trimmed, ws)
    } else {
        (msg, "")
    };

    if !msg.is_empty() {
        let processed_msg = process_markdown(msg, base_color);

        print!(
            "{}{}{}{}{}",
            leading_ws,
            ansi_rgb(base_color.0, base_color.1, base_color.2),
            processed_msg,
            trailing_ws,
            ANSI_RESET,
        );
    }
}

//===========================================================================//
// Custom color storage
//===========================================================================//

/// Static storage for custom colors
static CUSTOM_COLORS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

pub fn ensure_custom_colors() -> &'static Mutex<HashMap<String, String>> {
    CUSTOM_COLORS.get_or_init(|| {
        let table = vec![
            ("h1", "#fff"),
            ("txt,text", "#bbb"),
            ("error", "#f00"),
            ("warn", "#ffea00"),
            ("key", "#4CF"),
            ("opt,option", "#78aeff"),
            ("filename", "#e0c16c"),
            ("command", "#dbd488"),
            ("success", "#32CD32"),
            ("success_dim", "#80ad80"),
        ];
        let mut colors = HashMap::new();
        for (key, val) in table {
            for part in key.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    colors.insert(trimmed.to_string(), val.to_string());
                }
                if trimmed == "text" {
                    colors.insert("".to_string(), val.to_string());
                }
            }
        }
        Mutex::new(colors)
    })
}

/// Add a custom color to the global color table
pub fn cprintln_add_color(name: &str, value: &str) {
    let colors = ensure_custom_colors();
    if let Ok(mut colors_guard) = colors.lock() {
        colors_guard.insert(name.to_string(), value.to_string());
    }
}

//===========================================================================//
// Internal helpers
//===========================================================================//

fn process_markdown(text: &str, base_color: (u8, u8, u8)) -> String {
    // Regex to match [text](color) patterns
    let re = Regex::new(r"\\?\[([^\]]+)\]\(([^\)]+)\)").unwrap();
    let mut result = String::new();
    let mut last = 0;
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(0) {
            // Add text before the match
            result.push_str(&text[last..m.start()]);
            // If the match is escaped (starts with \[), print literally
            if m.as_str().starts_with(r"\[") {
                result.push_str(&m.as_str()[1..]);
            } else {
                let content = &cap[1];
                let color = &cap[2];
                let rgb = parse_color(color);
                let reset = ansi_rgb(base_color.0, base_color.1, base_color.2);
                result.push_str(&format!(
                    "{}{}{}",
                    ansi_rgb(rgb.0, rgb.1, rgb.2),
                    content,
                    reset
                ));
            }
            last = m.end();
        }
    }
    // Add any remaining text
    result.push_str(&text[last..]);
    result
}

fn parse_color(color: &str) -> (u8, u8, u8) {
    // Check custom colors first
    let colors = ensure_custom_colors();
    let resolved_color = if let Ok(colors_guard) = colors.lock() {
        colors_guard
            .get(color)
            .cloned()
            .unwrap_or_else(|| color.to_string())
    } else {
        color.to_string()
    };
    let color = resolved_color.as_str();

    // Try to match all standard named HTML colors
    let color = match color {
        "black" => "#000000",
        "silver" => "#c0c0c0",
        "gray" => "#808080",
        "white" => "#ffffff",
        "maroon" => "#800000",
        "red" => "#ff0000",
        "purple" => "#800080",
        "fuchsia" => "#ff00ff",
        "green" => "#008000",
        "lime" => "#00ff00",
        "olive" => "#808000",
        "yellow" => "#ffff00",
        "navy" => "#000080",
        "blue" => "#0000ff",
        "teal" => "#008080",
        "aqua" => "#00ffff",
        "orange" => "#ffa500",
        "aliceblue" => "#f0f8ff",
        "antiquewhite" => "#faebd7",
        "aquamarine" => "#7fffd4",
        "azure" => "#f0ffff",
        "beige" => "#f5f5dc",
        "bisque" => "#ffe4c4",
        "blanchedalmond" => "#ffebcd",
        "blueviolet" => "#8a2be2",
        "brown" => "#a52a2a",
        "burlywood" => "#deb887",
        "cadetblue" => "#5f9ea0",
        "chartreuse" => "#7fff00",
        "chocolate" => "#d2691e",
        "coral" => "#ff7f50",
        "cornflowerblue" => "#6495ed",
        "cornsilk" => "#fff8dc",
        "crimson" => "#dc143c",
        "cyan" => "#00ffff",
        "darkblue" => "#00008b",
        "darkcyan" => "#008b8b",
        "darkgoldenrod" => "#b8860b",
        "darkgray" => "#a9a9a9",
        "darkgreen" => "#006400",
        "darkgrey" => "#a9a9a9",
        "darkkhaki" => "#bdb76b",
        "darkmagenta" => "#8b008b",
        "darkolivegreen" => "#556b2f",
        "darkorange" => "#ff8c00",
        "darkorchid" => "#9932cc",
        "darkred" => "#8b0000",
        "darksalmon" => "#e9967a",
        "darkseagreen" => "#8fbc8f",
        "darkslateblue" => "#483d8b",
        "darkslategray" => "#2f4f4f",
        "darkslategrey" => "#2f4f4f",
        "darkturquoise" => "#00ced1",
        "darkviolet" => "#9400d3",
        "deeppink" => "#ff1493",
        "deepskyblue" => "#00bfff",
        "dimgray" => "#696969",
        "dimgrey" => "#696969",
        "dodgerblue" => "#1e90ff",
        "firebrick" => "#b22222",
        "floralwhite" => "#fffaf0",
        "forestgreen" => "#228b22",
        "gainsboro" => "#dcdcdc",
        "ghostwhite" => "#f8f8ff",
        "gold" => "#ffd700",
        "goldenrod" => "#daa520",
        "greenyellow" => "#adff2f",
        "grey" => "#808080",
        "honeydew" => "#f0fff0",
        "hotpink" => "#ff69b4",
        "indianred" => "#cd5c5c",
        "indigo" => "#4b0082",
        "ivory" => "#fffff0",
        "khaki" => "#f0e68c",
        "lavender" => "#e6e6fa",
        "lavenderblush" => "#fff0f5",
        "lawngreen" => "#7cfc00",
        "lemonchiffon" => "#fffacd",
        "lightblue" => "#add8e6",
        "lightcoral" => "#f08080",
        "lightcyan" => "#e0ffff",
        "lightgoldenrodyellow" => "#fafad2",
        "lightgray" => "#d3d3d3",
        "lightgreen" => "#90ee90",
        "lightgrey" => "#d3d3d3",
        "lightpink" => "#ffb6c1",
        "lightsalmon" => "#ffa07a",
        "lightseagreen" => "#20b2aa",
        "lightskyblue" => "#87cefa",
        "lightslategray" => "#778899",
        "lightslategrey" => "#778899",
        "lightsteelblue" => "#b0c4de",
        "lightyellow" => "#ffffe0",
        "limegreen" => "#32cd32",
        "linen" => "#faf0e6",
        "magenta" => "#ff00ff",
        "mediumaquamarine" => "#66cdaa",
        "mediumblue" => "#0000cd",
        "mediumorchid" => "#ba55d3",
        "mediumpurple" => "#9370db",
        "mediumseagreen" => "#3cb371",
        "mediumslateblue" => "#7b68ee",
        "mediumspringgreen" => "#00fa9a",
        "mediumturquoise" => "#48d1cc",
        "mediumvioletred" => "#c71585",
        "midnightblue" => "#191970",
        "mintcream" => "#f5fffa",
        "mistyrose" => "#ffe4e1",
        "moccasin" => "#ffe4b5",
        "navajowhite" => "#ffdead",
        "oldlace" => "#fdf5e6",
        "olivedrab" => "#6b8e23",
        "orangered" => "#ff4500",
        "orchid" => "#da70d6",
        "palegoldenrod" => "#eee8aa",
        "palegreen" => "#98fb98",
        "paleturquoise" => "#afeeee",
        "palevioletred" => "#db7093",
        "papayawhip" => "#ffefd5",
        "peachpuff" => "#ffdab9",
        "peru" => "#cd853f",
        "pink" => "#ffc0cb",
        "plum" => "#dda0dd",
        "powderblue" => "#b0e0e6",
        "rosybrown" => "#bc8f8f",
        "royalblue" => "#4169e1",
        "saddlebrown" => "#8b4513",
        "salmon" => "#fa8072",
        "sandybrown" => "#f4a460",
        "seagreen" => "#2e8b57",
        "seashell" => "#fff5ee",
        "sienna" => "#a0522d",
        "skyblue" => "#87ceeb",
        "slateblue" => "#6a5acd",
        "slategray" => "#708090",
        "slategrey" => "#708090",
        "snow" => "#fffafa",
        "springgreen" => "#00ff7f",
        "steelblue" => "#4682b4",
        "tan" => "#d2b48c",
        "thistle" => "#d8bfd8",
        "tomato" => "#ff6347",
        "turquoise" => "#40e0d0",
        "violet" => "#ee82ee",
        "wheat" => "#f5deb3",
        "whitesmoke" => "#f5f5f5",
        "yellowgreen" => "#9acd32",
        "rebeccapurple" => "#663399",
        _ => color,
    };

    let color = color.to_ascii_lowercase();
    let hex = color.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(200);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(100);
            (r, g, b)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(200);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(100);
            (r, g, b)
        }
        _ => (200, 200, 200), // Default
    }
}

/// Helper function to generate ANSI RGB color escape sequences
fn ansi_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

/// ANSI reset sequence to default foreground color
const ANSI_RESET: &str = "\x1b[39m";
