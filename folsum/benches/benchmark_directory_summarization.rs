use criterion::{criterion_group, criterion_main, Criterion};

use folsum::{generate_fake_file_paths, perform_fake_inventory};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("benchmark_directory_summarization",
                     |b| b.iter(|| {
                         // Set up the test.
                         let expected_file_paths = generate_fake_file_paths(20, 3);
                         perform_fake_inventory(&expected_file_paths)
                     }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
