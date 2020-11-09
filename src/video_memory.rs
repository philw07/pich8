use std::ops::Index;
use bitvec::{bitarr, array::BitArray, order::Msb0};
use getset::{Getters, Setters};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum VideoMode {
    Default,
    HiRes,
    Extended,
}

#[derive(Getters, Setters, Serialize, Deserialize)]
pub struct VideoMemory {
    vmem: [BitArray<Msb0, [u64; 32]>; 4],
    #[getset(get = "pub", set = "pub")]
    video_mode: VideoMode,
}

impl VideoMemory {
    const WIDTH_DEFAULT: usize = 64;
    const HEIGHT_DEFAULT: usize = 32;
    const WIDTH_HIRES: usize = 64;
    const HEIGHT_HIRES: usize = 64;
    const WIDTH_EXTENDED: usize = 128;
    const HEIGHT_EXTENDED: usize = 64;
    const BUF_LEN: usize = Self::WIDTH_DEFAULT * Self::HEIGHT_DEFAULT;

    pub fn new() -> Self {
        Self {
            vmem: [
                bitarr![Msb0, u64; 0; 64*32],
                bitarr![Msb0, u64; 0; 64*32],
                bitarr![Msb0, u64; 0; 64*32],
                bitarr![Msb0, u64; 0; 64*32]
            ],
            video_mode: VideoMode::Default,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.set_index(self.get_index(x, y), value);
    }

    fn set_index(&mut self, index: usize, value: bool) {
        if index >= self.width() * self.height() {
            panic!("Index out of bounds");
        }
        self.vmem[index / Self::BUF_LEN].set(index % Self::BUF_LEN, value);
    }

    pub fn clear(&mut self) {
        self.set_all(false);
    }

    pub fn set_all(&mut self, value: bool) {
        for i in 0..self.vmem.len() {
            self.vmem[i].set_all(value);
        }
    }

    pub fn width(&self) -> usize {
        match self.video_mode {
            VideoMode::Default => Self::WIDTH_DEFAULT,
            VideoMode::HiRes => Self::WIDTH_HIRES,
            VideoMode::Extended => Self::WIDTH_EXTENDED,
        }
    }

    pub fn height(&self) -> usize {
        match self.video_mode {
            VideoMode::Default => Self::HEIGHT_DEFAULT,
            VideoMode::HiRes => Self::HEIGHT_HIRES,
            VideoMode::Extended => Self::HEIGHT_EXTENDED,
        }
    }

    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width() + x
    }

    pub fn get(&self, x: usize, y:usize) -> bool {
        self[self.get_index(x, y)]
    }

    pub fn scroll_down(&mut self, lines: usize) {
        for y in (0..self.height()).rev() {
            for x in 0..self.width() {
                let val = if y < lines { false } else { self.get(x, y - lines) };
                self.set(x, y, val);
            }
        }
    }

    pub fn scroll_left(&mut self) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                let val = if x >= self.width() - 4 { false } else { self.get(x + 4, y) };
                self.set(x, y, val);
            }
        }
    }

    pub fn scroll_right(&mut self) {
        for x in (0..self.width()).rev() {
            for y in 0..self.height() {
                let val = if x < 4 { false } else { self.get(x - 4, y) };
                self.set(x, y, val);
            }
        }
    }
}

impl Index<usize> for VideoMemory {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.width() * self.height() {
            panic!("Index out of bounds");
        }

        &self.vmem[index / Self::BUF_LEN][index % Self::BUF_LEN]
    }
}

#[cfg(test)]
mod video_memory_test {
    use super::*;

    #[test]
    fn test_initial_state() {
        let vmem = VideoMemory::new();
        assert_eq!(vmem.video_mode, VideoMode::Default);
        for i in 0..64*32 {
            assert_eq!(vmem[i], false);
        }
    }

    #[test]
    fn test_width_height() {
        let mut vmem = VideoMemory::new();
        assert_eq!(vmem.video_mode, VideoMode::Default);
        assert_eq!(vmem.width(), 64);
        assert_eq!(vmem.height(), 32);
        vmem.set_video_mode(VideoMode::HiRes);
        assert_eq!(vmem.video_mode, VideoMode::HiRes);
        assert_eq!(vmem.width(), 64);
        assert_eq!(vmem.height(), 64);
        vmem.set_video_mode(VideoMode::Extended);
        assert_eq!(vmem.video_mode, VideoMode::Extended);
        assert_eq!(vmem.width(), 128);
        assert_eq!(vmem.height(), 64);
    }

    #[test]
    #[should_panic]
    fn test_get_index_out_of_bounds_default() {
        let vmem = VideoMemory::new();
        let _ = vmem[64*32];
    }

    #[test]
    #[should_panic]
    fn test_get_index_out_of_bounds_hires() {
        let mut vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::HiRes);
        let _ = vmem[64*64];
    }

    #[test]
    #[should_panic]
    fn test_get_index_out_of_bounds_extended() {
        let mut vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::Extended);
        let _ = vmem[128*64];
    }

    #[test]
    #[should_panic]
    fn test_set_index_out_of_bounds() {
        let mut vmem = VideoMemory::new();
        vmem.set(0, 50, true);
    }

    #[test]
    fn test_operations() {
        // Set - Default
        let mut vmem = VideoMemory::new();
        vmem.set(32, 20, true);
        assert_eq!(vmem.get(32, 20), true);
        // Set - HiRes
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::HiRes);
        vmem.set(32, 50, true);
        assert_eq!(vmem.get(32, 50), true);
        // Set - Extended
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::Extended);
        vmem.set(100, 50, true);
        assert_eq!(vmem.get(100, 50), true);

        // Set all - Default
        vmem = VideoMemory::new();
        vmem.set_all(true);
        for i in 0..64*32 {
            assert_eq!(vmem[i], true);
        }
        // Set all - HiRes
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::HiRes);
        vmem.set_all(true);
        for i in 0..64*64 {
            assert_eq!(vmem[i], true);
        }
        // Set all - Extended
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::Extended);
        vmem.set_all(true);
        for i in 0..128*64 {
            assert_eq!(vmem[i], true);
        }

        // Clear - Default
        vmem = VideoMemory::new();
        vmem.set_all(true);
        vmem.clear();
        for i in 0..64*32 {
            assert_eq!(vmem[i], false);
        }
        // Clear - Default
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::HiRes);
        vmem.set_all(true);
        vmem.clear();
        for i in 0..64*64 {
            assert_eq!(vmem[i], false);
        }
        // Clear - Default
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::Extended);
        vmem.set_all(true);
        vmem.clear();
        for i in 0..128*64 {
            assert_eq!(vmem[i], false);
        }

        // Scroll down
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::Extended);
        vmem.set_all(true);
        for x in 0..128 {
            vmem.set(x, 35, false);
        }
        vmem.scroll_down(3);
        for x in 0..128 {
            for y in 0..4 {
                assert_eq!(vmem.get(x, y), y==3);
            }
            assert_eq!(vmem.get(x, 38), false);
        }

        // Scroll left
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::Extended);
        vmem.set_all(true);
        for y in 0..64 {
            vmem.set(108, y, false);
        }
        vmem.scroll_left();
        for y in 0..64 {
            for x in 123..128 {
                assert_eq!(vmem.get(x, y), x==123);
            }
            assert_eq!(vmem.get(104, y), false);
        }

        // Scroll right
        vmem = VideoMemory::new();
        vmem.set_video_mode(VideoMode::Extended);
        vmem.set_all(true);
        for y in 0..64 {
            vmem.set(99, y, false);
        }
        vmem.scroll_right();
        for y in 0..64 {
            for x in 0..5 {
                assert_eq!(vmem.get(x, y), x==4);
            }
            assert_eq!(vmem.get(103, y), false);
        }
    }
}