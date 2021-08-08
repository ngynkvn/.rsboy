use criterion::{criterion_group, criterion_main, Criterion};
use rust_emu::emu::Emu;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Emu step", |b| {
        b.iter(|| {
            let mut emu = Emu::from_bytes(
                include_bytes!("../Dr. Mario (JU) (V1.1).gb"),
                include_bytes!("../dmg_boot.bin"),
            )
            .unwrap();
            let mut bus = emu.bus;
            bus.in_bios = 1;
            for _ in 0..999999 {
                // emu.cpu.opcode = instr;
                emu.cpu.step(&mut bus);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
