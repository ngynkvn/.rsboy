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
- CPU - WIP
- MEM - WIP
- SOUND
- GFX - WIP
- OTHER STUFF

## References
- _Writing a Game Boy emulator, Cinoop_, CTurt: https://cturt.github.io/cinoop.html
- _GameBoy Emulation in JavaScript: GPU Timings_, Imran Nazar: http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings
- _GameBoy Opcode Summary_, Jeff Frohwein: http://www.devrs.com/gb/files/opcodes.html
- _GameBoy CPU Manual_, Pan of Anthrox et al.: https://realboyemulator.files.wordpress.com/2013/01/gbcpuman.pdf
