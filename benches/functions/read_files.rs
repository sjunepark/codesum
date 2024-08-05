use std::fmt::Debug;
use std::path::Path;

use criterion::Criterion;

use codesum::SimpleReader;
use codesum::{ReadResult, SyncRead};

pub fn criterion_benchmark(c: &mut Criterion) {
    let root = Path::new("..");
    let absolute_path = root.canonicalize().unwrap();
    println!("absolute_path: {:?}", absolute_path);

    let result = execute_read_files(root);
    let len = result.content.len();
    let file_count = result.file_count;
    assert!(len > 0, "Expected result to have a length greater than 0");
    assert!(file_count > 0, "Expected file_count to be greater than 0");

    println!("Length of result: {}", len);
    println!("File count: {}", file_count);

    c.bench_function("read_files", |b| {
        b.iter(|| {
            let _ = execute_read_files(root);
        });
    });
}

fn execute_read_files<P>(root: P) -> ReadResult
where
    P: AsRef<Path> + Debug,
{
    let sr = SimpleReader::new();
    sr.read_files(root)
}
