#[macro_use]
extern crate criterion;

use chipsandlib::cpu::R8;
use chipsandlib::mmu::MMU;
use criterion::black_box;
use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    let mmu = MMU::new(Vec::new());
    let mut cpu = R8::new(mmu);
    c.bench_function("regtest", move |b| b.iter(|| black_box(cpu.get_reg_af())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
