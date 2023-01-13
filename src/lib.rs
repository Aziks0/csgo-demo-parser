mod messages;
mod parser;
mod reader;

pub mod error;

#[cfg(test)]
mod tests {
    use crate::parser::DemoParser;
    use std::fs::File;

    pub const DATA_TESTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data");
}
