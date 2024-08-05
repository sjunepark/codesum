use std::path::Path;

use tracing::debug;
use tracing_subscriber::EnvFilter;

pub struct TestContext {}

impl TestContext {
    pub fn new() -> Self {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();

        debug!("Initialized test context");
        TestContext {}
    }
}

pub struct ReaderTester;

impl ReaderTester {
    pub fn new() -> Self {
        ReaderTester
    }

    pub fn test_for_current_crate(&self, content: &str) {
        let root_path = Path::new(".");
        let cargo_toml = root_path.join("Cargo.toml");
        if !(cargo_toml.is_file() && cargo_toml.exists()) {
            panic!("Cargo.toml not found in current directory");
        }

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
