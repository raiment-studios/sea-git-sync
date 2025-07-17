//! A fairly sloppy, good-enough implementation to support color-formatted
//! printing to the console.  Uses a markdown-like syntax of `[text](tag)`
//! to specify how to format the text.
//!
//! The `tag` can be a hex color value, a named HTML color, or a "semnatic"
//! tag like `filename`, `filepath`, or `number`. The tag can both change
//! the color of the text as well as apply formatting to the text.
//!
//! The implementation was partly generated using Gemini 2.5 Pro. Apologies :)
//!

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

//===========================================================================//
// Custom color storage (dynamic, with aliasing and runtime add)
//===========================================================================//

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
pub fn cprint_add_color(name: &str, value: &str) {
    let colors = ensure_custom_colors();
    if let Ok(mut colors_guard) = colors.lock() {
        colors_guard.insert(name.to_string(), value.to_string());
    }
}

const RESET: &str = "\x1b[0m";

//===========================================================================//
// Public symbols
//===========================================================================//

pub fn cprint_imp(color: &str, s: &str) {
    let base_color_rgb = match parse_color(color) {
        Some(rgb) => rgb,
        None => RGB::gray(),
    };

    print!("{}", base_color_rgb.to_ansi());
    for fragment in parse_text(s) {
        if fragment.tag.is_empty() {
            print!("{}", fragment.text);
        } else {
            let text = format_text(fragment.text, &fragment.tag);

            match parse_color(&fragment.tag) {
                Some(rgb) => {
                    print!("{}{}{}", rgb.to_ansi(), text, base_color_rgb.to_ansi());
                }
                None => {
                    print!("[{}]({})", text, fragment.tag);
                }
            }
        }
    }
    print!("{}", RESET);
}

pub fn cprintln_imp(color: &str, s: &str) {
    cprint_imp(color, s);
    println!();
}

//===========================================================================//
// Implementation internals
//===========================================================================//

/// Given a string, parses out anything matching the markdown-like
/// syntax of [some text](tag) and returns a vector of Fragments.
///
fn parse_text(s: &str) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    while current_pos < len {
        if let Some(mut open_bracket_pos) = chars[current_pos..].iter().position(|&c| c == '[') {
            open_bracket_pos += current_pos; // Make absolute

            // Add text before the '['
            if open_bracket_pos > current_pos {
                fragments.push(Fragment {
                    tag: "".to_string(),
                    text: chars[current_pos..open_bracket_pos].iter().collect(),
                });
            }

            // --- Start of logic for finding matching ']' and parsing tag ---
            let mut scan_pos = open_bracket_pos + 1;
            let mut bracket_nesting_level = 1;
            let mut found_matching_close_bracket = false;
            let mut actual_close_bracket_pos = 0;

            while scan_pos < len {
                match chars[scan_pos] {
                    '[' => bracket_nesting_level += 1,
                    ']' => {
                        bracket_nesting_level -= 1;
                        if bracket_nesting_level == 0 {
                            actual_close_bracket_pos = scan_pos;
                            found_matching_close_bracket = true;
                            break;
                        }
                    }
                    _ => {}
                }
                scan_pos += 1;
            }

            if found_matching_close_bracket {
                let close_bracket_pos = actual_close_bracket_pos;
                let text_match: String = chars[(open_bracket_pos + 1)..close_bracket_pos]
                    .iter()
                    .collect();

                // Check for '(' immediately after ']'
                if close_bracket_pos + 1 < len && chars[close_bracket_pos + 1] == '(' {
                    // Search for ')' in the slice starting after '('
                    let tag_content_start_pos = close_bracket_pos + 2;
                    if tag_content_start_pos <= len {
                        // Ensure slice is valid
                        if let Some(relative_close_paren_pos) = chars[tag_content_start_pos..]
                            .iter()
                            .position(|&c| c == ')')
                        {
                            let close_paren_pos = tag_content_start_pos + relative_close_paren_pos;
                            // tag_start is effectively tag_content_start_pos
                            // Ensure tag content is valid (close_paren_pos is not before tag_content_start_pos)
                            // This is implicitly true if position found something.
                            let tag_str: String = chars[tag_content_start_pos..close_paren_pos]
                                .iter()
                                .collect();
                            fragments.push(Fragment {
                                tag: tag_str,
                                text: text_match,
                            });
                            current_pos = close_paren_pos + 1; // Update current_pos for outer loop
                            continue; // Process next token
                        }
                    }
                }

                // Fallback: Not a full [text](tag) pattern.
                // This includes "[text]" (no tag), or "[text](malformed_tag".
                // Treat the '[' at open_bracket_pos as literal.
                fragments.push(Fragment {
                    tag: "".to_string(),
                    text: "[".to_string(),
                });
                current_pos = open_bracket_pos + 1; // Next iteration starts after this '['
            } else {
                // No matching ']' found for the '[' at open_bracket_pos.
                // The text from open_bracket_pos (which is where '[' is) onwards is literal.
                current_pos = open_bracket_pos;
                break; // Exit while loop, remainder will be added by post-loop logic.
            }
        } else {
            // No more opening brackets
            break;
        }
    }

    // Add any remaining text after the last processed position
    if current_pos < len {
        fragments.push(Fragment {
            tag: "".to_string(),
            text: chars[current_pos..].iter().collect(),
        });
    }
    if fragments.is_empty() && !s.is_empty() {
        fragments.push(Fragment {
            tag: "".to_string(),
            text: s.to_string(),
        });
    }

    fragments
}

struct Fragment {
    tag: String,
    text: String,
}

#[derive(Debug, Clone, Copy)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    fn gray() -> Self {
        RGB {
            r: 128,
            g: 128,
            b: 128,
        }
    }

    fn to_ansi(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }
}

/// Semantic text formatting (not just color)
fn format_text(s: String, tag: &str) -> String {
    match tag {
        "number" => {
            if let Ok(num) = s.parse::<i64>() {
                let num_str = num.abs().to_string();
                let chars: Vec<char> = num_str.chars().rev().collect();
                let mut with_commas = String::new();
                for (i, c) in chars.iter().enumerate() {
                    if i > 0 && i % 3 == 0 {
                        with_commas.push(',');
                    }
                    with_commas.push(*c);
                }
                let formatted: String = with_commas.chars().rev().collect();
                if num < 0 {
                    format!("-{}", formatted)
                } else {
                    formatted
                }
            } else {
                s
            }
        }
        "filename" | "filepath" => {
            let prefix_rgb = parse_hex("#ed552b").unwrap().to_ansi();
            let text_rgb = parse_color(tag).unwrap().to_ansi();

            let cwd = match std::env::current_dir() {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(_) => "".to_string(),
            };
            if !cwd.is_empty() && s != cwd && s.starts_with(&cwd) {
                return format!("{}.{}{}", prefix_rgb, text_rgb, &s[cwd.len()..]);
            }
            if let Ok(home) = std::env::var("HOME") {
                if s.starts_with(&home) {
                    return format!("{}~{}{}", prefix_rgb, text_rgb, &s[home.len()..]);
                }
            }
            s
        }
        _ => s,
    }
}

fn parse_color(color: &str) -> Option<RGB> {
    // Check custom colors first (dynamic, with aliasing)
    let resolved_color = {
        let colors = ensure_custom_colors();
        if let Ok(colors_guard) = colors.lock() {
            colors_guard
                .get(color)
                .cloned()
                .unwrap_or_else(|| color.to_string())
        } else {
            color.to_string()
        }
    };
    let hex = if let Some(hex) = html_named_color(&resolved_color) {
        hex
    } else if let Some(hex) = snowfall_color(&resolved_color) {
        hex
    } else {
        resolved_color.as_str()
    };
    parse_hex(hex)
}

fn parse_hex(hex: &str) -> Option<RGB> {
    let hex = if hex.len() == 7 && hex.starts_with('#') {
        &hex[1..]
    } else if hex.len() == 4 && hex.starts_with('#') {
        &hex[1..]
    } else {
        hex
    };

    let rgb = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            RGB { r, g, b }
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            RGB { r, g, b }
        }
        _ => {
            return None;
        }
    };
    Some(rgb)
}

fn snowfall_color(name: &str) -> Option<&'static str> {
    match name {
        "filename" | "filepath" => Some("#f7cd43"),
        "number" | "digits" => Some("#556fed"),
        _ => None,
    }
}

fn html_named_color(name: &str) -> Option<&'static str> {
    match name {
        "aliceblue" => Some("#f0f8ff"),
        "antiquewhite" => Some("#faebd7"),
        "aqua" => Some("#00ffff"),
        "aquamarine" => Some("#7fffd4"),
        "azure" => Some("#f0ffff"),
        "beige" => Some("#f5f5dc"),
        "bisque" => Some("#ffe4c4"),
        "black" => Some("#000000"),
        "blanchedalmond" => Some("#ffebcd"),
        "blue" => Some("#0000ff"),
        "blueviolet" => Some("#8a2be2"),
        "brown" => Some("#a52a2a"),
        "burlywood" => Some("#deb887"),
        "cadetblue" => Some("#5f9ea0"),
        "chartreuse" => Some("#7fff00"),
        "chocolate" => Some("#d2691e"),
        "coral" => Some("#ff7f50"),
        "cornflowerblue" => Some("#6495ed"),
        "cornsilk" => Some("#fff8dc"),
        "crimson" => Some("#dc143c"),
        "cyan" => Some("#00ffff"),
        "darkblue" => Some("#00008b"),
        "darkcyan" => Some("#008b8b"),
        "darkgoldenrod" => Some("#b8860b"),
        "darkgray" => Some("#a9a9a9"),
        "darkgreen" => Some("#006400"),
        "darkgrey" => Some("#a9a9a9"),
        "darkkhaki" => Some("#bdb76b"),
        "darkmagenta" => Some("#8b008b"),
        "darkolivegreen" => Some("#556b2f"),
        "darkorange" => Some("#ff8c00"),
        "darkorchid" => Some("#9932cc"),
        "darkred" => Some("#8b0000"),
        "darksalmon" => Some("#e9967a"),
        "darkseagreen" => Some("#8fbc8f"),
        "darkslateblue" => Some("#483d8b"),
        "darkslategray" => Some("#2f4f4f"),
        "darkslategrey" => Some("#2f4f4f"),
        "darkturquoise" => Some("#00ced1"),
        "darkviolet" => Some("#9400d3"),
        "deeppink" => Some("#ff1493"),
        "deepskyblue" => Some("#00bfff"),
        "dimgray" => Some("#696969"),
        "dimgrey" => Some("#696969"),
        "dodgerblue" => Some("#1e90ff"),
        "firebrick" => Some("#b22222"),
        "floralwhite" => Some("#fffaf0"),
        "forestgreen" => Some("#228b22"),
        "fuchsia" => Some("#ff00ff"),
        "gainsboro" => Some("#dcdcdc"),
        "ghostwhite" => Some("#f8f8ff"),
        "gold" => Some("#ffd700"),
        "goldenrod" => Some("#daa520"),
        "gray" => Some("#808080"),
        "green" => Some("#008000"),
        "greenyellow" => Some("#adff2f"),
        "grey" => Some("#808080"),
        "honeydew" => Some("#f0fff0"),
        "hotpink" => Some("#ff69b4"),
        "indianred" => Some("#cd5c5c"),
        "indigo" => Some("#4b0082"),
        "ivory" => Some("#fffff0"),
        "khaki" => Some("#f0e68c"),
        "lavender" => Some("#e6e6fa"),
        "lavenderblush" => Some("#fff0f5"),
        "lawngreen" => Some("#7cfc00"),
        "lemonchiffon" => Some("#fffacd"),
        "lightblue" => Some("#add8e6"),
        "lightcoral" => Some("#f08080"),
        "lightcyan" => Some("#e0ffff"),
        "lightgoldenrodyellow" => Some("#fafad2"),
        "lightgray" => Some("#d3d3d3"),
        "lightgreen" => Some("#90ee90"),
        "lightgrey" => Some("#d3d3d3"),
        "lightpink" => Some("#ffb6c1"),
        "lightsalmon" => Some("#ffa07a"),
        "lightseagreen" => Some("#20b2aa"),
        "lightskyblue" => Some("#87cefa"),
        "lightslategray" => Some("#778899"),
        "lightslategrey" => Some("#778899"),
        "lightsteelblue" => Some("#b0c4de"),
        "lightyellow" => Some("#ffffe0"),
        "lime" => Some("#00ff00"),
        "limegreen" => Some("#32cd32"),
        "linen" => Some("#faf0e6"),
        "magenta" => Some("#ff00ff"),
        "maroon" => Some("#800000"),
        "mediumaquamarine" => Some("#66cdaa"),
        "mediumblue" => Some("#0000cd"),
        "mediumorchid" => Some("#ba55d3"),
        "mediumpurple" => Some("#9370db"),
        "mediumseagreen" => Some("#3cb371"),
        "mediumslateblue" => Some("#7b68ee"),
        "mediumspringgreen" => Some("#00fa9a"),
        "mediumturquoise" => Some("#48d1cc"),
        "mediumvioletred" => Some("#c71585"),
        "midnightblue" => Some("#191970"),
        "mintcream" => Some("#f5fffa"),
        "mistyrose" => Some("#ffe4e1"),
        "moccasin" => Some("#ffe4b5"),
        "navajowhite" => Some("#ffdead"),
        "navy" => Some("#000080"),
        "oldlace" => Some("#fdf5e6"),
        "olive" => Some("#808000"),
        "olivedrab" => Some("#6b8e23"),
        "orange" => Some("#ffa500"),
        "orangered" => Some("#ff4500"),
        "orchid" => Some("#da70d6"),
        "palegoldenrod" => Some("#eee8aa"),
        "palegreen" => Some("#98fb98"),
        "paleturquoise" => Some("#afeeee"),
        "palevioletred" => Some("#db7093"),
        "papayawhip" => Some("#ffefd5"),
        "peachpuff" => Some("#ffdab9"),
        "peru" => Some("#cd853f"),
        "pink" => Some("#ffc0cb"),
        "plum" => Some("#dda0dd"),
        "powderblue" => Some("#b0e0e6"),
        "purple" => Some("#800080"),
        "rebeccapurple" => Some("#663399"),
        "red" => Some("#ff0000"),
        "rosybrown" => Some("#bc8f8f"),
        "royalblue" => Some("#4169e1"),
        "saddlebrown" => Some("#8b4513"),
        "salmon" => Some("#fa8072"),
        "sandybrown" => Some("#f4a460"),
        "seagreen" => Some("#2e8b57"),
        "seashell" => Some("#fff5ee"),
        "sienna" => Some("#a0522d"),
        "silver" => Some("#c0c0c0"),
        "skyblue" => Some("#87ceeb"),
        "slateblue" => Some("#6a5acd"),
        "slategray" => Some("#708090"),
        "slategrey" => Some("#708090"),
        "snow" => Some("#fffafa"),
        "springgreen" => Some("#00ff7f"),
        "steelblue" => Some("#4682b4"),
        "tan" => Some("#d2b48c"),
        "teal" => Some("#008080"),
        "thistle" => Some("#d8bfd8"),
        "tomato" => Some("#ff6347"),
        "turquoise" => Some("#40e0d0"),
        "violet" => Some("#ee82ee"),
        "wheat" => Some("#f5deb3"),
        "white" => Some("#ffffff"),
        "whitesmoke" => Some("#f5f5f5"),
        "yellow" => Some("#ffff00"),
        "yellowgreen" => Some("#9acd32"),
        _ => None,
    }
}
