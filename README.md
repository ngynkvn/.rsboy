# RUST GB EMULATOR
Cause that hasnt been done before.

- The code is extremely rough. View at your own discretion.

## Some Goals
- Utilize macros wherever possible to DRY code out and increase legibility.
   - and make it easier to test performance between a mutable implementation and an immutable copy-based approach. (since it's DRYer I'll theoretically have an easier time rewriting part of the codebase to make this a compile-time flag)
- Add time-rewinding.
- Cycle-accurate emulation.
- Eventually try to copy Google Stadia and play with "negative" latency methods
- Work smarter not harder, use the tools and knowledge I have to speed up development by avoiding gruntwork tasks like linking every single opcode to its corresponding function. Some examples include using vim's line based capabilities and scraping data to make it more managable for me to work with.

## TODO
- CREATE CONTEXT - DONE
- CPU
- MEM
- SOUND
- GFX
- OTHER STUFF

## Spec
2.3. Game Boy Specs
ï CPU: 8-bit (Similar to the Z80 processor.)
ï Main RAM: 8K Byte
ï Video RAM: 8K Byte
ï Screen Size 2.6"
ï Resolution: 160x144 (20x18 tiles)
ï Max # of sprites: 40
ï Max # sprites/line: 10
ï Max sprite size: 8x16
ï Min sprite size: 8x8
ï Clock Speed: 4.194304 MHz
(4.295454 SGB, 4.194/8.388MHz GBC)
ï Horiz Sync: 9198 KHz (9420 KHz for SGB)
ï Vert Sync: 59.73 Hz (61.17 Hz for SGB)
ï Sound: 4 channels with stereo sound
ï Power: DC6V 0.7W (DC3V 0.7W for GB Pocket)
 Nintendo documents describe the CPU & instructions
speed in machine cycles while this document describes
them in clock cycles. Here is the translation:
 1 machine cycle = 4 clock cycles
 GB CPU Speed NOP Instruction
Machine Cycles 1.05MHz 1 cycle
Clock Cycles 4.19MHz 4 cycles

## References
- Writing a Game Boy emulator, Cinoop: https://cturt.github.io/cinoop.html
- GameBoy Opcode Summary: http://www.devrs.com/gb/files/opcodes.html
- GameBoy CPU Manual: https://realboyemulator.files.wordpress.com/2013/01/gbcpuman.pdf
