> [!CAUTION]
> Firstly, this doesn't compile and from the brief 30 minutes I put into it, is not trivial to fix
> This is very bad rust code. It is not performant at all. I [rebuilt a Go version](https://github.com/ngynkvn/gogb) in a couple weeks that beats this in about every metric.
>
> I only leave it up here for those that are nonetheless curious.
>
> For me, at the time I remember being very obsessed with understanding algebraic data types and definitely approached this project with the mentality that [a single hammer was all I needed.](https://en.wikipedia.org/wiki/Law_of_the_instrument#:~:text=The%20law%20of%20the%20instrument,original%20to%20either%20of%20them.)
> This was a learning moment for me that simplicity cannot be understated, and to be considerate about the techniques you choose to deploy in your applications.
>
> Regardless, I'm still proud that I put a lot of effort into trying to make it work !

![Rust](https://github.com/ngynkvn/.rsboy/workflows/Rust/badge.svg)

## A gameboy emulator in Rust

Cause that hasnt been done before.

- The code is extremely rough. View at your own discretion.

# Features
- Software Renderer
- Parse and decode instructions from gameboy binaries

---

<img src="docs/image.png" style="display:block;margin:0 auto" width=300px/>
Render image of Tetris main screen

## TODO
- CPU - Passing blargg's cpu_instr test suite, sans interrupts
  - Pass "02-interrupts.gb"
  - Create Github Action to test these gb files by reading from I/O port
- MEM - Some memory access issues are still in place.
  - Research, fix memory R/W issues
- SOUND
  - This will be a long one. Low priority
- GFX
  - Still some inaccuracies. I will not be implementing the full PPU operations
- WebAssembly Port

## References
- _Writing a Game Boy emulator, Cinoop_, CTurt: https://cturt.github.io/cinoop.html
- _GameBoy Emulation in JavaScript: GPU Timings_, Imran Nazar: http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings
- _GameBoy Opcode Summary_, Jeff Frohwein: http://www.devrs.com/gb/files/opcodes.html
- _GameBoy CPU Manual_, Pan of Anthrox et al.: https://realboyemulator.files.wordpress.com/2013/01/gbcpuman.pdf
- _Pan Docs_, Pan of ATX et al.: https://gbdev.io/pandocs/
- _mooneye-gb_, Game Boy research project and emulator, Joonas Javanainen: https://github.com/Gekkio/mooneye-gb

---
<img src="docs/cuddlyferris.svg" style="display:block;margin:0 auto" width=200px/>
