use bitvec::prelude::*;

pub trait DisplayOutput {
    fn draw(&mut self, buffer: &BitArray<Msb0, [u64; 32]>);
}

pub trait SoundOutput {
    fn start_sound(&self);
    fn stop_sound(&self);
}