use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VideoMode {
    Default,
    HiRes,
    Extended,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Plane {
    None,
    First,
    Second,
    Both,
}

#[derive(Serialize, Deserialize)]
pub struct VideoMemory {
    vmem1: Box<[bool]>,
    vmem2: Box<[bool]>,
    pub video_mode: VideoMode,
    plane: Plane,
}

impl VideoMemory {
    const WIDTH_DEFAULT: usize = 64;
    const HEIGHT_DEFAULT: usize = 32;
    const WIDTH_HIRES: usize = 64;
    const HEIGHT_HIRES: usize = 64;
    const WIDTH_EXTENDED: usize = 128;
    const HEIGHT_EXTENDED: usize = 64;

    pub fn new() -> Self {
        Self {
            vmem1: vec![false; 128*64].into_boxed_slice(),
            vmem2: vec![false; 128*64].into_boxed_slice(),
            video_mode: VideoMode::Default,
            plane: Plane::First,
        }
    }

    pub fn select_plane(&mut self, plane: Plane) {
        self.plane = plane;
    }

    pub fn current_plane(&self) -> Plane {
        self.plane
    }

    pub fn set_plane(&mut self, plane: Plane, x: usize, y: usize, value: bool) {
        // In default video mode, we're translating the 64x32 screen to 128x64,
        // only this way the scroll commands work correctly in S-CHIP low res mode.
        if self.video_mode == VideoMode::Default {
            let (x, y) = (x * 2, y * 2);
            self.set_index_plane(plane, self.to_index(x, y), value);
            self.set_index_plane(plane, self.to_index(x+1, y), value);
            self.set_index_plane(plane, self.to_index(x, y+1), value);
            self.set_index_plane(plane, self.to_index(x+1, y+1), value);
        } else {
            self.set_index_plane(plane, self.to_index(x, y), value);
        }
    }

    fn set_index_plane(&mut self, plane: Plane, index: usize, value: bool) {
        if index >= self.render_width() * self.render_height() {
            panic!("Index out of bounds");
        }
        match plane {
            Plane::None => (),
            Plane::First => self.vmem1[index] = value,
            Plane::Second => self.vmem2[index] = value,
            Plane::Both => {
                self.vmem1[index] = value;
                self.vmem2[index] = value;
            },
        }
    }

    pub fn clear(&mut self) {
        self.set_all(false);
    }

    pub fn set_all(&mut self, value: bool) {
        match self.plane {
            Plane::None => (),
            Plane::First => self.vmem1.iter_mut().for_each(|x| *x = value),
            Plane::Second => self.vmem2.iter_mut().for_each(|x| *x = value),
            Plane::Both => { self.vmem1.iter_mut().for_each(|x| *x = value); self.vmem2.iter_mut().for_each(|x| *x = value); },
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

    pub fn to_index(&self, x: usize, y: usize) -> usize {
        y * self.render_width() + x
    }

    pub fn get_index_plane(&self, plane: Plane, index: usize) -> bool {
        if index >= self.render_width() * self.render_height() {
            panic!("Index out of bounds");
        }
        match plane {
            Plane::None     => false,
            Plane::First    => self.vmem1[index],
            Plane::Second   => self.vmem2[index],
            Plane::Both     => panic!("shouldn't call get with both planes selected!"),
        }
    }

    pub fn get_plane(&self, plane: Plane, x: usize, y:usize) -> bool {
        let (x, y) = if self.video_mode == VideoMode::Default { (x * 2, y * 2) } else { (x, y) };
        self.get_index_plane(plane, self.to_index(x, y))
    }

    pub fn scroll_down(&mut self, lines: usize) {
        for y in (0..self.render_height()).rev() {
            for x in 0..self.render_width() {
                if self.plane == Plane::First || self.plane == Plane::Both {
                    let val = if y < lines { false } else { self.get_plane(Plane::First, x, y - lines) };
                    // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                    self.set_index_plane(Plane::First, self.to_index(x, y), val);
                }
                if self.plane == Plane::Second || self.plane == Plane::Both {
                    let val = if y < lines { false } else { self.get_plane(Plane::Second, x, y - lines) };
                    // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                    self.set_index_plane(Plane::Second, self.to_index(x, y), val);
                }
            }
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        for y in 0..self.render_height() {
            for x in 0..self.render_width() {
                if self.plane == Plane::First || self.plane == Plane::Both {
                    let val = if y >= self.render_height() - lines { false } else { self.get_plane(Plane::First, x, y + lines) };
                    // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                    self.set_index_plane(Plane::First, self.to_index(x, y), val);
                }
                if self.plane == Plane::Second || self.plane == Plane::Both {
                    let val = if y >= self.render_height() - lines { false } else { self.get_plane(Plane::Second, x, y + lines) };
                    // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                    self.set_index_plane(Plane::Second, self.to_index(x, y), val);
                }
            }
        }
    }

    pub fn scroll_left(&mut self) {
        for x in 0..self.render_width() {
            for y in 0..self.render_height() {
                if self.plane == Plane::First || self.plane == Plane::Both {
                    let val = if x >= self.width() - 4 { false } else { self.get_plane(Plane::First,x + 4, y) };
                    // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                    self.set_index_plane(Plane::First, self.to_index(x, y), val);
                }
                if self.plane == Plane::Second || self.plane == Plane::Both {
                    let val = if x >= self.width() - 4 { false } else { self.get_plane(Plane::Second,x + 4, y) };
                    // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                    self.set_index_plane(Plane::Second, self.to_index(x, y), val);
                }
            }
        }
    }

    pub fn scroll_right(&mut self) {
        for x in (0..self.render_width()).rev() {
            for y in 0..self.render_height() {
                if self.plane == Plane::First || self.plane == Plane::Both {
                    let val = if x < 4 { false } else { self.get_plane(Plane::First,x - 4, y) };
                    // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                    self.set_index_plane(Plane::First,self.to_index(x, y), val);
                }
                if self.plane == Plane::Second || self.plane == Plane::Both {
                    let val = if x < 4 { false } else { self.get_plane(Plane::Second,x - 4, y) };
                    // Need to use set_index instead of set, because set expects 64x32 coordinates in default video mode
                    self.set_index_plane(Plane::Second,self.to_index(x, y), val);
                }
            }
        }
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
            assert_eq!(vmem.get_index_plane(Plane::First, i), false);
            assert_eq!(vmem.get_index_plane(Plane::Second, i), false);
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
        vmem.video_mode = VideoMode::HiRes;
        assert_eq!(vmem.video_mode, VideoMode::HiRes);
        assert_eq!(vmem.width(), 64);
        assert_eq!(vmem.height(), 64);
        assert_eq!(vmem.render_width(), 64);
        assert_eq!(vmem.render_height(), 64);
        vmem.video_mode = VideoMode::Extended;
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
        vmem.video_mode = VideoMode::HiRes;
        let _ = vmem.get_index_plane(vmem.plane, 64*64);
    }

    #[test]
    #[should_panic]
    fn test_get_index_out_of_bounds_default_and_extended() {
        let mut vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::Extended;
        let _ = vmem.get_index_plane(vmem.plane, 128*64);
    }

    #[test]
    #[should_panic]
    fn test_set_index_out_of_bounds() {
        let mut vmem = VideoMemory::new();
        vmem.set_plane(vmem.plane, 0, 50, true);
    }

    #[test]
    fn test_operations() {
        // Set - Default
        let mut vmem = VideoMemory::new();
        vmem.set_plane(vmem.plane, 32, 20, true);
        assert_eq!(vmem.get_plane(vmem.plane, 32, 20), true);
        // Set - HiRes
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::HiRes;
        vmem.set_plane(vmem.plane, 32, 50, true);
        assert_eq!(vmem.get_plane(vmem.plane, 32, 50), true);
        // Set - Extended
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::Extended;
        vmem.set_plane(vmem.plane, 100, 50, true);
        assert_eq!(vmem.get_plane(vmem.plane, 100, 50), true);

        // Set all - Default
        vmem = VideoMemory::new();
        vmem.set_all(true);
        for i in 0..64*32 {
            assert_eq!(vmem.get_index_plane(vmem.plane, i), true);
        }
        // Set all - HiRes
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::HiRes;
        vmem.set_all(true);
        for i in 0..64*64 {
            assert_eq!(vmem.get_index_plane(vmem.plane, i), true);
        }
        // Set all - Extended
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::Extended;
        vmem.set_all(true);
        for i in 0..128*64 {
            assert_eq!(vmem.get_index_plane(vmem.plane, i), true);
        }

        // Clear - Default
        vmem = VideoMemory::new();
        vmem.set_all(true);
        vmem.clear();
        for i in 0..64*32 {
            assert_eq!(vmem.get_index_plane(vmem.plane, i), false);
        }
        // Clear - Default
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::HiRes;
        vmem.set_all(true);
        vmem.clear();
        for i in 0..64*64 {
            assert_eq!(vmem.get_index_plane(vmem.plane, i), false);
        }
        // Clear - Default
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::Extended;
        vmem.set_all(true);
        vmem.clear();
        for i in 0..128*64 {
            assert_eq!(vmem.get_index_plane(vmem.plane, i), false);
        }

        // Scroll down
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::Extended;
        vmem.set_all(true);
        for x in 0..128 {
            vmem.set_plane(vmem.plane, x, 35, false);
        }
        vmem.scroll_down(3);
        for x in 0..128 {
            for y in 0..4 {
                assert_eq!(vmem.get_plane(vmem.plane, x, y), y==3);
            }
            assert_eq!(vmem.get_plane(vmem.plane, x, 38), false);
        }

        // Scroll up
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::Extended;
        vmem.set_all(true);
        for x in 0..128 {
            vmem.set_plane(vmem.plane, x, 35, false);
        }
        vmem.scroll_up(7);
        for x in 0..128 {
            for y in 56..64 {
                assert_eq!(vmem.get_plane(vmem.plane, x, y), y==56);
            }
            assert_eq!(vmem.get_plane(vmem.plane, x, 28), false);
        }

        // Scroll left
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::Extended;
        vmem.set_all(true);
        for y in 0..64 {
            vmem.set_plane(vmem.plane, 108, y, false);
        }
        vmem.scroll_left();
        for y in 0..64 {
            for x in 123..128 {
                assert_eq!(vmem.get_plane(vmem.plane, x, y), x==123);
            }
            assert_eq!(vmem.get_plane(vmem.plane, 104, y), false);
        }

        // Scroll right
        vmem = VideoMemory::new();
        vmem.video_mode = VideoMode::Extended;
        vmem.set_all(true);
        for y in 0..64 {
            vmem.set_plane(vmem.plane, 99, y, false);
        }
        vmem.scroll_right();
        for y in 0..64 {
            for x in 0..5 {
                assert_eq!(vmem.get_plane(vmem.plane, x, y), x==4);
            }
            assert_eq!(vmem.get_plane(vmem.plane, 103, y), false);
        }
    }

    #[test]
    fn test_planes() {
        for plane in vec![Plane::None, Plane::First, Plane::Second, Plane::Both] {
            let plane1 = plane == Plane::First || plane == Plane::Both;
            let plane2 = plane == Plane::Second || plane == Plane::Both;

            let mut vmem = VideoMemory::new();
            vmem.select_plane(plane.clone());
            assert_eq!(vmem.plane, plane);

            // Set all
            vmem.set_all(true);
            for i in 0..128*64 {
                assert_eq!(vmem.get_index_plane(Plane::First, i), plane1);
                assert_eq!(vmem.get_index_plane(Plane::Second, i), plane2);
            }

            // Clear
            vmem.clear();
            for i in 0..128*64 {
                assert_eq!(vmem.get_index_plane(Plane::First, i), false);
                assert_eq!(vmem.get_index_plane(Plane::Second, i), false);
            }

            // Set x y + Get x y
            vmem.clear();
            vmem.set_plane(vmem.plane, 10, 10, true);
            assert_eq!(vmem.get_plane(Plane::First, 10, 10), plane1);
            assert_eq!(vmem.get_plane(Plane::Second, 10, 10), plane2);

            // Scroll down
            vmem = VideoMemory::new();
            vmem.select_plane(plane);
            vmem.video_mode = VideoMode::Extended;
            vmem.set_all(true);
            for x in 0..128 {
                vmem.set_plane(vmem.plane, x, 35, false);
            }
            vmem.scroll_down(3);
            for x in 0..128 {
                for y in 0..4 {
                    assert_eq!(vmem.get_plane(Plane::First, x, y), plane1 && y==3);
                    assert_eq!(vmem.get_plane(Plane::Second, x, y), plane2 && y==3);
                }
                assert_eq!(vmem.get_plane(Plane::First, x, 38), false);
                assert_eq!(vmem.get_plane(Plane::Second, x, 38), false);
            }

            // Scroll up
            vmem = VideoMemory::new();
            vmem.select_plane(plane);
            vmem.video_mode = VideoMode::Extended;
            vmem.set_all(true);
            for x in 0..128 {
                vmem.set_plane(vmem.plane, x, 35, false);
            }
            vmem.scroll_up(7);
            for x in 0..128 {
                for y in 56..64 {
                    assert_eq!(vmem.get_plane(Plane::First, x, y), plane1 && y==56);
                    assert_eq!(vmem.get_plane(Plane::Second, x, y), plane2 && y==56);
                }
                assert_eq!(vmem.get_plane(Plane::First, x, 28), false);
                assert_eq!(vmem.get_plane(Plane::Second, x, 28), false);
            }

            // Scroll left
            vmem = VideoMemory::new();
            vmem.select_plane(plane);
            vmem.video_mode = VideoMode::Extended;
            vmem.set_all(true);
            for y in 0..64 {
                vmem.set_plane(vmem.plane, 108, y, false);
            }
            vmem.scroll_left();
            for y in 0..64 {
                for x in 123..128 {
                    assert_eq!(vmem.get_plane(Plane::First, x, y), plane1 && x==123);
                    assert_eq!(vmem.get_plane(Plane::Second, x, y), plane2 && x==123);
                }
                assert_eq!(vmem.get_plane(Plane::First, 104, y), false);
                assert_eq!(vmem.get_plane(Plane::Second, 104, y), false);
            }

            // Scroll right
            vmem = VideoMemory::new();
            vmem.select_plane(plane);
            vmem.video_mode = VideoMode::Extended;
            vmem.set_all(true);
            for y in 0..64 {
                vmem.set_plane(vmem.plane, 99, y, false);
            }
            vmem.scroll_right();
            for y in 0..64 {
                for x in 0..5 {
                    assert_eq!(vmem.get_plane(Plane::First, x, y), plane1 && x==4);
                    assert_eq!(vmem.get_plane(Plane::Second, x, y), plane2 && x==4);
                }
                assert_eq!(vmem.get_plane(Plane::First, 103, y), false);
                assert_eq!(vmem.get_plane(Plane::Second, 103, y), false);
            }
        }
    }
}