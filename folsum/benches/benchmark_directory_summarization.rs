use criterion::{criterion_group, criterion_main, Criterion};

use folsum::perform_fake_inventory;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("benchmark_directory_summarization",
                     |b| b.iter(|| perform_fake_inventory()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
