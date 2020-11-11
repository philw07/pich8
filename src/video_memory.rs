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
        // In default video mode, we're translating the 64x32 screen to 128x64,
        // only this way the scroll commands work correctly in S-CHIP low res mode.
        if self.video_mode == VideoMode::Default {
            let (x, y) = (x * 2, y * 2);
            self.set_index(self.get_index(x, y), value);
            self.set_index(self.get_index(x+1, y), value);
            self.set_index(self.get_index(x, y+1), value);
            self.set_index(self.get_index(x+1, y+1), value);
        } else {
            self.set_index(self.get_index(x, y), value);
        }
    }

    fn set_index(&mut self, index: usize, value: bool) {
        if index >= self.render_width() * self.render_height() {
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

    /// Returns the render width.
    /// This differs from the actual screen width as in Default mode we render in Extended size to support S-CHIP low res commands (e.g. scrolling down "half-pixels").
    pub fn render_width(&self) -> usize {
        match self.video_mode {
            VideoMode::Default | VideoMode::Extended => Self::WIDTH_EXTENDED,
            VideoMode::HiRes => Self::WIDTH_HIRES,
        }
    }

    /// Returns the render height.
    /// This differs from the actual screen height as in Default mode we render in Extended size to support S-CHIP low res commands (e.g. scrolling down "half-pixels").
    pub fn render_height(&self) -> usize {
        match self.video_mode {
            VideoMode::Default | VideoMode::Extended => Self::HEIGHT_EXTENDED,
            VideoMode::HiRes => Self::HEIGHT_HIRES,
        }
    }

    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.render_width() + x
    }

    pub fn get(&self, x: usize, y:usize) -> bool {
        if self.video_mode == VideoMode::Default {
            let (x, y) = (x * 2, y * 2);
            self[self.get_index(x, y)]
        } else {
            self[self.get_index(x, y)]
        }
    }

    pub fn scroll_down(&mut self, lines: usize) {
        for y in (0..self.render_height()).rev() {
            for x in 0..self.render_width() {
                let val = if y < lines { false } else { self.get(x, y - lines) };
                // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                self.set_index(self.get_index(x, y), val);
            }
        }
    }

    pub fn scroll_left(&mut self) {
        for x in 0..self.render_width() {
            for y in 0..self.render_height() {
                let val = if x >= self.width() - 4 { false } else { self.get(x + 4, y) };
                // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                self.set_index(self.get_index(x, y), val);
            }
        }
    }

    pub fn scroll_right(&mut self) {
        for x in (0..self.render_width()).rev() {
            for y in 0..self.render_height() {
                let val = if x < 4 { false } else { self.get(x - 4, y) };
                // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                self.set_index(self.get_index(x, y), val);
            }
        }
    }
}

impl Index<usize> for VideoMemory {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.render_width() * self.render_height() {
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
        assert_eq!(vmem.render_width(), 128);
        assert_eq!(vmem.render_height(), 64);
        vmem.set_video_mode(VideoMode::HiRes);
        assert_eq!(vmem.video_mode, VideoMode::HiRes);
        assert_eq!(vmem.width(), 64);
        assert_eq!(vmem.height(), 64);
        assert_eq!(vmem.render_width(), 64);
        assert_eq!(vmem.render_height(), 64);
        vmem.set_video_mode(VideoMode::Extended);
        assert_eq!(vmem.video_mode, VideoMode::Extended);
        assert_eq!(vmem.width(), 128);
        assert_eq!(vmem.height(), 64);
        assert_eq!(vmem.render_width(), 128);
        assert_eq!(vmem.render_height(), 64);
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
    fn test_get_index_out_of_bounds_default_and_extended() {
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