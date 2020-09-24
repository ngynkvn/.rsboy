use crate::emu::Emu;
use crate::instructions::INSTR_TABLE;
use crossterm::cursor::MoveTo;
use crossterm::{
    cursor::*,
    event, execute,
    style::{Color::*, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::ClearType::All,
    terminal::*,
    ExecutableCommand,
};
use std::io::stdout;

type EmuHook = dyn Fn(&Emu) -> Option<String>;

pub struct Tui {
    hooks: Vec<Box<EmuHook>>,
}

const CLOCK: &str = "🕒";
impl Tui {
    pub fn new() -> Self {
        Tui { hooks: vec![] }
    }

    pub fn add_hook<F: 'static + Fn(&Emu) -> Option<String>>(&mut self, f: F) {
        self.hooks.push(Box::new(f));
    }

    pub fn init(&mut self) -> crossterm::Result<()> {
        // self.add_hook(|_emu| Some("A problem was encountered.".to_string()));
        stdout()
            .execute(Clear(All))?
            .execute(Hide)?
            .execute(MoveTo(0, 0))?;
        Ok(())
    }

    pub fn print_state(&self, emu: &Emu) -> crossterm::Result<()> {
        for hook in &self.hooks {
            if let Some(err) = hook(emu) {
                panic!(
                    "\n==HOOK ERROR==\nA problem with a hook occurred:\n{}\n",
                    err
                );
            }
        }
        stdout()
            .execute(MoveTo(0, 0))?
            .execute(Print("RegisterState:\n"))?
            .execute(MoveDown(1))?
            .execute(Print(format!("{}", emu.cpu.registers)))?
            .execute(Print(format!("{} {}", CLOCK, emu.bus.clock)))?
            .execute(MoveTo(20, 0))
            .and_then(|std| {
                let view = emu.view();
                for il in view {
                    std.execute(SavePosition)
                        .and_then(|std| {
                            if il.addr == emu.cpu.op_addr {
                                std.execute(crossterm::style::SetBackgroundColor(Green))?;
                            } else {
                                std.execute(crossterm::style::SetBackgroundColor(Black))?;
                            }
                            std.execute(Print(format!(
                                "{:04x}: {:?} {:04x}                     ",
                                il.addr,
                                il.instr,
                                il.data.unwrap_or(0),
                            )))
                        })?
                        .execute(RestorePosition)?
                        .execute(MoveDown(1))?;
                }
                Ok(())
            })
    }
}
