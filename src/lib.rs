mod messages;
mod reader;

pub mod error;
pub mod parser;

// Re-export them to be in `csgo_demo_parser`
#[doc(inline)]
pub use parser::DemoParser;

#[cfg(test)]
mod tests {
    use crate::parser::DemoParser;
    use std::fs::File;

    pub const DATA_TESTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data");
}
