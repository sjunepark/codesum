use aho_corasick::AhoCorasick;
use criterion::{BenchmarkId, Criterion, Throughput};

fn single_pass(haystack: &str, needles: &[&str]) {
    assert!(!haystack
        .lines()
        .any(|line| needles.iter().all(|substr| line.contains(substr))));
}

fn multiple_asserts(haystack: &str, needles: &[&str]) {
    needles.iter().for_each(|&substr| {
        assert!(!haystack.contains(substr));
    });
}

fn aho_corasick_search(ac: &AhoCorasick, haystack: &str) {
    assert!(ac.find(haystack).is_none());
}

fn generate_large_string_without_expected(size: usize) -> String {
    let base = "Some random string which doesn't contain the expected substrings.";
    let mut result = String::with_capacity(size);
    while result.len() < size {
        result.push_str(base);
    }
    result.truncate(size);
    result
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let needles = vec!["mod", "use clap::Parser;", "fn main()", "main.rs", "lib.rs"];
    let ac = AhoCorasick::new(&needles).expect("Failed to create AhoCorasick");

    let small_string = generate_large_string_without_expected(1_000);
    let medium_string = generate_large_string_without_expected(100_000);
    let large_string = generate_large_string_without_expected(1_000_000);

    let mut group = c.benchmark_group("validate_result");

    for (name, haystack) in [
        ("small", &small_string),
        ("medium", &medium_string),
        ("large", &large_string),
    ] {
        group.throughput(Throughput::Elements(haystack.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("multiple_asserts", name),
            haystack,
            |b, haystack| b.iter(|| multiple_asserts(haystack, &needles)),
        );

        group.bench_with_input(
            BenchmarkId::new("single_pass", name),
            haystack,
            |b, haystack| b.iter(|| single_pass(haystack, &needles)),
        );

        group.bench_with_input(
            BenchmarkId::new("aho_corasick_search", name),
            haystack,
            |b, haystack| b.iter(|| aho_corasick_search(&ac, haystack)),
        );
    }

    group.finish();
}
