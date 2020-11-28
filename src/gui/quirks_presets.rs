use super::QuirksSettings;

#[derive(Copy, Clone)]
pub enum QuirksPreset {
    Default,
    Octo,
}

pub struct QuirksPresetHandler<'a> {
    settings: &'a mut QuirksSettings,
}

impl<'a> QuirksPresetHandler<'a> {
    const QUIRKS_PRESET_DEFAULT: [bool; 5] = [true; 5];
    const QUIRKS_PRESET_OCTO: [bool; 5] = [false, false, true, false, true];

    pub fn new(settings: &'a mut QuirksSettings) -> Self {
        Self {
            settings,
        }
    }

    pub fn is_active(&self, preset: QuirksPreset) -> bool {
        for (v1, v2) in self.settings.iter().zip(self.get_preset(preset).iter()) {
            if v1 != v2 {
                return false;
            }
        }

        true
    }

    pub fn set_preset(&mut self, preset: QuirksPreset) {
        let preset = self.get_preset(preset);
        for (v1, v2) in self.settings.iter_mut().zip(preset.iter()) {
            *v1 = *v2;
        }
    }

    fn get_preset(&self, preset: QuirksPreset) -> [bool; 5] {
        match preset {
            QuirksPreset::Default => Self::QUIRKS_PRESET_DEFAULT,
            QuirksPreset::Octo => Self::QUIRKS_PRESET_OCTO,
        }
    }
}
