use std::path::Path;

use tracing_subscriber::EnvFilter;

use crate::Read;

pub struct TestContext;

impl TestContext {
    pub fn new() -> Self {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_test_writer()
            .init();

        TestContext {}
    }
}

pub struct ReaderTester<R>(R)
where
    R: Read;

impl<R: Read> ReaderTester<R> {
    pub fn new(reader: R) -> Self {
        ReaderTester(reader)
    }

    pub fn test_reader_for_current_crate(&self) {
        let root_path = Path::new(".");
        let cargo_toml = root_path.join("Cargo.toml");
        if !(cargo_toml.is_file() && cargo_toml.exists()) {
            panic!("Cargo.toml not found in current directory");
        }

        let content = self.0.read_files(".").content;

        assert!(
            !content.is_empty(),
            "Expected content to have a length greater than 0"
        );

        assert!(&content.contains("mod"));
        assert!(&content.contains("fn main()"));
        assert!(&content.contains("Cargo.toml"));
        assert!(&content.contains("src"));
    }
}
