use criterion::{BenchmarkId, Criterion};
use tokio::runtime::Runtime;

use codesum::{AsyncReader, SimpleReader};
use codesum::{Read, SyncRead};

pub fn criterion_benchmark(c: &mut Criterion) {
    let paths = [".", ".."];
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("aggregate");
    for path in paths {
        group.bench_with_input(BenchmarkId::new("simple", path), path, |b, path| {
            b.iter(|| {
                let sr = SimpleReader::new();
                let _ = sr.aggregate(path);
            });
        });
        group.bench_with_input(BenchmarkId::new("async", path), path, |b, path| {
            b.to_async(&runtime).iter(|| async {
                let path = path.to_string();
                let ar = AsyncReader::new();
                let _ = ar.aggregate(path).await;
            });
        });
    }
}
