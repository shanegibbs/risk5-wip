#[macro_use]
extern crate criterion;

use criterion::Criterion;
use risk5::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("1mil", move |b| {
        b.iter_with_large_setup(
            || (Processor::new(build_memory()), build_matchers()),
            |mut data| {
                let mut cpu = data.0;
                let matchers = &mut data.1;
                let mut counter = 0;
                loop {
                    if counter == 100_000 {
                        break;
                    }
                    cpu.step(matchers);
                    counter += 1;
                }
            },
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
