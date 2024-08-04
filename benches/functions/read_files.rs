use std::path::Path;

use criterion::Criterion;

use codesum::Read;
use codesum::SimpleReader;

pub fn criterion_benchmark(c: &mut Criterion) {
    let root = Path::new(".");
    let absolute_path = root.canonicalize().unwrap();
    println!("absolute_path: {:?}", absolute_path);

    c.bench_function("read_files", |b| {
        b.iter(|| {
            let sr = SimpleReader::new();
            let result = sr.read_files(".");
            let len = result.len();
            assert!(len > 0, "Expected result to have a length greater than 0");
        });
    });
}
