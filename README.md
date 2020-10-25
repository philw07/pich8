# pich8
A CHIP-8 interpreter written in Rust using rust-sdl2

[...]

## Build

By default, SDL2 will be linked dynamically and you'll need the SDL2 runtime library (e.g. SDL2.dll on Windows) which you can download at https://www.libsdl.org/.  
You can compile a statically linked binary instead using the command
> cargo build --features static-link

In this case there are no runtime dependencies, but compile time dependencies are required depending on your OS.  
The [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) repo provides details on static linking.  
