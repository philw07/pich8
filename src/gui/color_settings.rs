use std::slice::{Iter, IterMut};

pub enum Color {
    Background,
    Plane1,
    Plane2,
    PlaneBoth,
}

pub struct ColorSettings {
    colors: [[f32; 3]; 4],
    pub changed: bool,
}

impl ColorSettings {
    pub fn new() -> Self {
        Self {
            colors: [[0.0; 3]; 4],
            changed: false,
        }
    }

    pub fn get(&self, color: Color) -> [f32; 3] {
        self.colors[color as usize]
    }

    pub fn get_mut(&mut self, color: Color) -> &mut [f32; 3] {
        &mut self.colors[color as usize]
    }

    pub fn iter(&self) -> Iter<[f32; 3]> {
        self.colors.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<[f32; 3]> {
        self.colors.iter_mut()
    }
}
