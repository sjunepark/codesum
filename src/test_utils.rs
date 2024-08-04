use tracing_subscriber::EnvFilter;

pub struct TestContext;

impl TestContext {
    pub fn new() -> Self {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_test_writer()
            .init();

        TestContext {}
    }

    pub fn validate_content(&self, content: &str) {
        assert!(content.contains("mod"));
        assert!(content.contains("use clap::Parser;"));
        assert!(content.contains("fn main()"));
        assert!(content.contains("main.rs"));
        assert!(content.contains("lib.rs"));
    }
}
