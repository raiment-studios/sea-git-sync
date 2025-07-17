#[macro_export]
macro_rules! debugln {
    ($($arg:tt)*) => {{
        // ANSI escape code: ESC[38;2;<r>;<g>;<b>m
        const COLOR: &str = "\x1b[38;2;208;75;255m";
        const RESET: &str = "\x1b[0m";
        println!("{}{}{}", COLOR, format!($($arg)*), RESET);
    }};
}

#[macro_export]
macro_rules! cprintln {
    ($color:expr, $($arg:tt)*) => {{
        $crate::prelude::cprintln_imp($color, format!($($arg)*).as_str());
    }};
}
