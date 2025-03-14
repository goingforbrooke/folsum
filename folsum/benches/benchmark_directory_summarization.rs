use criterion::{criterion_group, criterion_main, Criterion};

use folsum::run_fake_summarization;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("benchmark_directory_summarization",
                     |b| b.iter(|| run_fake_summarization()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
