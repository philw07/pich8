use crate::contracts::SoundOutput;
use std::f64::consts::PI;
use sdl2::{
    Sdl,
    audio::{
        AudioSpecDesired,
        AudioQueue,
    }
};

pub struct NoSound{}
impl SoundOutput for NoSound { fn beep(&self) {} }

pub struct BeepSound {
    queue: AudioQueue<f32>,
}

impl SoundOutput for BeepSound {
    fn beep(&self) {
        let len = BeepSound::SAMPLING_RATE / 10;
        if self.queue.size() < len as u32 * 2 {
            // Create sine wave
            let mut buf = Vec::new();
            for i in 0..len {
                buf.push(BeepSound::VOLUME * ((2.0 * PI as f32 * BeepSound::FREQUENCY * i as f32) / BeepSound::SAMPLING_RATE as f32).sin());
            }
            
            // Queue and play it
            self.queue.queue(&buf);
            self.queue.resume();
        }
    }
}

impl BeepSound {
    const SAMPLING_RATE: i32 = 48_000;
    const VOLUME: f32 = 0.05;
    const FREQUENCY: f32 = 440.0;

    pub fn new(sdl_context: &Sdl) -> Result<Self, String> {
        let spec = AudioSpecDesired{
            freq: Some(BeepSound::SAMPLING_RATE),
            channels: Some(1),
            samples: Some(2048),
        };

        Ok(Self{
            queue: sdl_context.audio()?.open_queue(None, &spec)?,
        })
    }
}
