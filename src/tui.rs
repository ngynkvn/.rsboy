use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
    terminal::*,
    terminal::ClearType::All,
    cursor::*,
    event,
};
use crossterm::cursor::MoveTo;
use std::io::stdout;
use crate::emu::Emu;

type EmuHook = dyn Fn(&Emu) -> Option<String>;

pub struct Tui {
    hooks: Vec<Box<EmuHook>>
}

const CLOCK: &str = "🕒";
impl Tui {

pub fn new() -> Self {
    Tui {
        hooks: vec![]
    }
}

pub fn add_hook<F: 'static + Fn(&Emu) -> Option<String>>(&mut self, f: F) {
    self.hooks.push(Box::new(f));
}

pub fn init(&mut self) {
    self.add_hook(|_emu| {
        Some("A problem was encountered.".to_string())
    })
}

pub fn clear(&self) {
    stdout()
        .execute(Clear(All)).unwrap()
        .execute(Hide).unwrap()
        .execute(MoveTo(0, 0)).unwrap();
}
pub fn print_state(&self, emu: &Emu) {
    for hook in &self.hooks {
        if let Some(err) = hook(emu) {
            panic!("\n==HOOK ERROR==\nA problem with a hook occurred:\n{}\n", err);
        }
    }
    (|| -> crossterm::Result<_> {
        stdout()
            .execute(MoveTo(0, 0))?
            .execute(Print("RegisterState:\n"))?
            .execute(MoveDown(1))?
            .execute(Print(format!("{}", emu.cpu.registers)))?
            .execute(MoveTo(0, 10))?
            .execute(Print(format!("{} {}", CLOCK, emu.bus.clock)))?;
        Ok(())
    })().unwrap()
}
}

