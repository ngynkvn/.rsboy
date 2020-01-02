pub struct Emu {
    memory: Memory,
    cpu: CPU,
    gpu: GPU,
}

impl Emu {
    fn new(rom: Vec<u8>) -> Self {
        let memory = Memory::new(rom);
        Self {
            memory,
            cpu: CPU::new(),
            gpu: GPU::new(),
        }
    }

    fn step(&self) {
        self.cpu.cycle(&mut self.memory);
        self.gpu.cycle(&self.memory);
    }
}