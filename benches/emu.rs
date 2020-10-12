use criterion::{criterion_group, criterion_main, Criterion};
use rust_emu::emu::Emu;
use rust_emu::instructions::INSTR_TABLE;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Emu step", |b| {
        b.iter(|| {
            let mut emu = Emu::new(vec![]);
            let mut bus = emu.bus;
            bus.in_bios = 1;
            for _instr in INSTR_TABLE.iter() {
                // emu.cpu.opcode = instr;
                emu.cpu.step(&mut bus);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
