use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_emu::emu::Emu;
use rust_emu::instructions::INSTR_TABLE;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Emu step", |b| {
        b.iter(|| {
            let mut emu = Emu::new(vec![]);
            for instr in INSTR_TABLE.iter() {
                emu.cpu.opcode = instr;
            }
            emu.emulate_step();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
