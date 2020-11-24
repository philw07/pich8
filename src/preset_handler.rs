#[derive(Copy, Clone)]
pub enum QuirksPreset {
    Default,
    Octo,
}

pub struct QuirksPresetHandler<'a> {
    values: [&'a mut bool; 5],
}

impl<'a> QuirksPresetHandler<'a> {
    const QUIRKS_PRESET_DEFAULT: [bool; 5] = [true; 5];
    const QUIRKS_PRESET_OCTO: [bool; 5] = [false, false, true, false, true];

    pub fn new(load_store: &'a mut bool, shift: &'a mut bool, draw: &'a mut bool, jump: &'a mut bool, vf_order: &'a mut bool) -> Self {
        Self {
            values: [load_store, shift, draw, jump, vf_order],
        }
    }

    pub fn is_active(&self, preset: QuirksPreset) -> bool {
        for (v1, v2) in self.values.iter().zip(self.get_preset(preset).iter()) {
            if *v1 != v2 {
                return false;
            }
        }

        true
    }

    pub fn set_preset(&mut self, preset: QuirksPreset) {
        let preset = self.get_preset(preset);
        for (v1, v2) in self.values.iter_mut().zip(preset.iter()) {
            **v1 = *v2;
        }
    }

    fn get_preset(&self, preset: QuirksPreset) -> [bool; 5] {
        match preset {
            QuirksPreset::Default => Self::QUIRKS_PRESET_DEFAULT,
            QuirksPreset::Octo => Self::QUIRKS_PRESET_OCTO,
        }
    }
}


#[derive(Copy, Clone)]
pub enum ColorPreset {
    Default,
    OctoClassic,
    OctoLcd,
    OctoHotdog,
    OctoGray,
    OctoCga0,
    OctoCga1,
}

pub struct ColorPresetHandler<'a> {
    values: [&'a mut [f32; 3]; 4],
}

impl<'a> ColorPresetHandler<'a> {
    const COLOR_PRESET_DEFAULT: [[f32; 3]; 4] = [
        [0.0; 3],
        [1.0; 3],
        [0.333; 3],
        [0.667; 3],
    ];
    const COLOR_PRESET_OCTO_CLASSIC: [[f32; 3]; 4] = [
        [0.6, 0.4, 0.0],
        [1.0, 0.8, 0.0],
        [1.0, 0.4, 0.0],
        [0.4, 0.133, 0.0],
    ];
    const COLOR_PRESET_OCTO_LCD: [[f32; 3]; 4] = [
        [0.976, 1.0, 0.702],
        [0.239, 0.502, 0.149],
        [0.671, 0.8, 0.278],
        [0.0, 0.075, 0.102],
    ];
    const COLOR_PRESET_OCTO_HOTDOG: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
    ];
    const COLOR_PRESET_OCTO_GRAY: [[f32; 3]; 4] = [
        [0.667, 0.667, 0.667],
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.4, 0.4, 0.4],
    ];
    const COLOR_PRESET_OCTO_CGA0: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    const COLOR_PRESET_OCTO_CGA1: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ];

    pub fn new(color_bg: &'a mut [f32; 3], color_plane_1: &'a mut [f32; 3], color_plane_2: &'a mut [f32; 3], color_plane_both: &'a mut [f32; 3]) -> Self {
        Self {
            values: [color_bg, color_plane_1, color_plane_2, color_plane_both],
        }
    }

    pub fn is_active(&self, preset: ColorPreset) -> bool {
        for (v1, v2) in self.values.iter().zip(self.get_preset(preset).iter()) {
            if *v1 != v2 {
                return false;
            }
        }

        true
    }

    pub fn set_preset(&mut self, preset: ColorPreset) {
        let preset = self.get_preset(preset);
        for (v1, v2) in self.values.iter_mut().zip(preset.iter()) {
            **v1 = *v2;
        }
    }

    fn get_preset(&self, preset: ColorPreset) -> [[f32; 3]; 4] {
        match preset {
            ColorPreset::Default => Self::COLOR_PRESET_DEFAULT,
            ColorPreset::OctoClassic => Self::COLOR_PRESET_OCTO_CLASSIC,
            ColorPreset::OctoLcd => Self::COLOR_PRESET_OCTO_LCD,
            ColorPreset::OctoHotdog => Self::COLOR_PRESET_OCTO_HOTDOG,
            ColorPreset::OctoGray => Self::COLOR_PRESET_OCTO_GRAY,
            ColorPreset::OctoCga0 => Self::COLOR_PRESET_OCTO_CGA0,
            ColorPreset::OctoCga1 => Self::COLOR_PRESET_OCTO_CGA1,
        }
    }
}