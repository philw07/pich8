use super::ColorSettings;

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
    settings: &'a mut ColorSettings,
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

    pub fn new(settings: &'a mut ColorSettings) -> Self {
        Self {
            settings,
        }
    }

    pub fn is_active(&self, preset: ColorPreset) -> bool {
        for (v1, v2) in self.settings.iter().zip(self.get_preset(preset).iter()) {
            if v1 != v2 {
                return false;
            }
        }

        true
    }

    pub fn set_preset(&mut self, preset: ColorPreset) {
        let preset = self.get_preset(preset);
        for (v1, v2) in self.settings.iter_mut().zip(preset.iter()) {
            *v1 = *v2;
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
