use criterion::{criterion_group, criterion_main};

mod functions;

criterion_group!(
    benches,
    functions::read_files::criterion_benchmark,
    functions::validate_result::criterion_benchmark
);

criterion_main!(benches);
