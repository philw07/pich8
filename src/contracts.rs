use bitvec::prelude::*;

pub trait DisplayOutput {
    fn draw(&mut self, buffer: &BitArray<Msb0, [u64; 32]>) -> Result<(), String>;
}

pub trait SoundOutput {
    fn beep(&self);
}