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

pub fn clear() {
    stdout()
        .execute(Clear(All)).unwrap()
        .execute(Hide).unwrap()
        .execute(MoveTo(0, 0)).unwrap();
}

const CLOCK: &str = "🕒";
pub fn print_state(emu: &Emu) {
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
