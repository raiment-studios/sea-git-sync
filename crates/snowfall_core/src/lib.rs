mod console;
mod macros;
pub mod strings;

pub mod prelude {
    pub use super::console::*;
    pub use super::cprintln;
    pub use super::debugln;

    pub mod core {
        pub use super::super::strings::*;
    }
}

pub mod internal {
    pub use super::prelude::*;
}
