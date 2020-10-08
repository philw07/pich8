use rodio::{
    OutputStream,
    Sink,
    source::SineWave,
    Source
};
use std::time::Duration;
use crate::contracts::SoundOutput;

pub struct NoSound{}
impl SoundOutput for NoSound { fn start_sound(&self) {} fn stop_sound(&self) {} }

pub struct BeepSound {
    frequency: u32,
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
}

impl SoundOutput for BeepSound {
    fn start_sound(&self) {
        if self.sink.is_some() {
            let beep = SineWave::new(self.frequency).take_duration(Duration::from_secs(5)).repeat_infinite();
            let sink = self.sink.as_ref().unwrap();
            sink.append(beep);
        }
    }

    fn stop_sound(&self) { //TODO: sound is not audible if it's too short, e.g. BRIX rom uses sound timer vaues of 1, 4 and 32.
        if self.sink.is_some() {
            self.sink.as_ref().unwrap().stop();
        }
    }
}

impl BeepSound {
    const VOLUME: f32 = 0.2;

    pub fn new(frequency: u32) -> Self {
        let mut stream: Option<OutputStream> = None;
        let mut sink: Option<Sink> = None;

        let default_stream = OutputStream::try_default();
        if default_stream.is_ok() {
            let (o_stream, handle) = default_stream.unwrap();
            stream = Some(o_stream);
            let new_sink = Sink::try_new(&handle);
            if new_sink.is_ok() {
                sink = Some(new_sink.unwrap());
                sink.as_ref().unwrap().set_volume(BeepSound::VOLUME);
            }
        }

        Self {
            frequency: frequency,
            _stream: stream,
            sink: sink,
        }
    }
}
