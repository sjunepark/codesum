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

    pub fn validate_result(&self, result: &str) {
        assert!(result.contains("mod"));
        assert!(result.contains("use clap::Parser;"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("main.rs"));
        assert!(result.contains("lib.rs"));
    }
}
