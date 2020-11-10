# pich8
A cross-platform CHIP-8 and SUPER-CHIP interpreter written in Rust

## Features

- Support for CHIP-8 and SUPER-CHIP 1.1 (S-CHIP)
- Supports screen resolutions 64x32, 64x64 (CHIP-8 HiRes) and 128x64 (S-CHIP)
- Rendering and sound using native Rust crates [glium](https://github.com/glium/glium) and [rodio](https://github.com/RustAudio/rodio)
- GUI using crate [imgui-rs](https://github.com/Gekkio/imgui-rs) (Rust bindings for [Dear ImGui](https://github.com/ocornut/imgui))
- Load ROMs from local file system or download them directly from a URL
- Save and load current CPU state
- Fullscreen mode and possibility to change background and foreground colors
- Change CPU speed dynamically
- Disable CHIP-8 load/store and shift quirks (enabled by default) and vertical wrapping (necessary at least for the ROM BLITZ)

## Sources for CHIP-8 ROM files

- https://johnearnest.github.io/chip8Archive/
    - Sources: https://github.com/JohnEarnest/chip8Archive
- https://www.zophar.net/pdroms/chip8.html
- https://github.com/loktar00/chip8/tree/master/roms
- https://github.com/dmatlack/chip8/tree/master/roms

## Resources

These are the resources I mostly used during development.

- http://www.pong-story.com/chip8/ (especially CHIP8.DOC)
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
- http://devernay.free.fr/hacks/chip8/schip.txt
- http://devernay.free.fr/hacks/chip8/
- https://en.wikipedia.org/wiki/CHIP-8
- http://www.multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
- https://github.com/JohnEarnest/Octo
- https://github.com/tomdaley92/kiwi-8
