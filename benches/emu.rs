use criterion::{Criterion, criterion_group, criterion_main};
use rust_emu::{emu::Emu, instructions::INSTR_TABLE};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Emu step", |b| {
        b.iter(|| {
            let mut emu = Emu::new(&[], None);
            let mut bus = emu.bus;
            bus.in_bios = 1;
            for _instr in &INSTR_TABLE {
                // emu.cpu.opcode = instr;
                emu.cpu.step(&mut bus);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
