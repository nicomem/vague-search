use criterion::*;

fn bench_test(n: u64) -> u64 {
    // let mut r = 0;
    // for i in 0..n {
    //     r += i
    // }
    // r
    n
}

fn cr_bench_test(c: &mut Criterion) {
    c.bench_function("bench_test", |b| b.iter(|| bench_test(black_box(10))));
}

criterion_group!(benches, cr_bench_test);
criterion_main!(benches);
