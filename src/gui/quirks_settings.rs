use std::slice::{Iter, IterMut};

pub enum Quirk {
    LoadStore = 0,
    Shift = 1,
    Draw = 2,
    Jump = 3,
    VfOrder = 4,
    PartialWrapH = 5,
    PartialWrapV = 6,
}

pub struct QuirksSettings {
    quirks: [bool; Self::NUM_QUIRKS],
}

impl QuirksSettings {
    pub const NUM_QUIRKS: usize = 7;

    pub fn new() -> Self {
        Self {
            quirks: [false; Self::NUM_QUIRKS],
        }
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
