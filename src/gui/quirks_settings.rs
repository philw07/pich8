use std::slice::{Iter, IterMut};

pub enum Quirk {
    LoadStore = 0,
    Shift = 1,
    Draw = 2,
    Jump = 3,
    VfOrder = 4,
}

pub struct QuirksSettings {
    quirks: [bool; 5],
}

impl QuirksSettings {
    pub fn new() -> Self {
        Self { quirks: [false; 5] }
    }

    pub fn get(&self, quirk: Quirk) -> bool {
        self.quirks[quirk as usize]
    }

    pub fn get_mut(&mut self, quirk: Quirk) -> &mut bool {
        &mut self.quirks[quirk as usize]
    }

    pub fn iter(&self) -> Iter<bool> {
        self.quirks.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<bool> {
        self.quirks.iter_mut()
    }
}
