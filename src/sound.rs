use rodio::{
    buffer::SamplesBuffer,
    queue::queue,
    source::{SineWave, Source},
    OutputStream, Sink,
};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

pub enum Command {
    PlayBeep,
    PlayBuffer([u8; 16]),
    SetVolume(f32),
}

pub struct AudioPlayer {
    tx_play: Sender<Command>,
}

impl AudioPlayer {
    const BEEP_FREQ: u32 = 440;
    const BUF_FREQ: u32 = 4000;
    const VOLUME: f32 = 0.05;

    pub fn new() -> Result<Self, String> {
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let (queue, output_queue) = queue(true);
            let sample_rate = output_queue.sample_rate();
            let beep_duration = Duration::from_secs_f32(1.5 / 60.0);
            if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    sink.append(output_queue);

                    loop {
                        if let Ok(cmd) = rx.recv() {
                            match cmd {
                                Command::PlayBeep => queue.append(
                                    SineWave::new(Self::BEEP_FREQ).take_duration(beep_duration),
                                ),
                                Command::PlayBuffer(buf) => {
                                    let reps = sample_rate / Self::BUF_FREQ;
                                    let mut samples =
                                        Vec::with_capacity(buf.len() * 8 * reps as usize);
                                    for byte in &buf {
                                        for idx_bit in 0..8 {
                                            let bit = byte >> (7 - idx_bit) & 0b1 == 0b1;
                                            let val = if bit { Self::VOLUME } else { 0.0 };
                                            for _ in 0..reps {
                                                samples.push(val);
                                            }
                                        }
                                    }
                                    let sample_buffer = SamplesBuffer::new(1, sample_rate, samples);
                                    queue.append(
                                        sample_buffer
                                            .take_duration(Duration::from_secs_f32(1.0 / 60.0)),
                                    );
                                }
                                Command::SetVolume(vol) => sink.set_volume(vol),
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { tx_play: tx })
    }

    pub fn beep(&self) {
        // Ignore if something went wrong
        let _ = self.tx_play.send(Command::PlayBeep);
    }

    pub fn play_buffer(&self, buf: [u8; 16]) {
        let _ = self.tx_play.send(Command::PlayBuffer(buf));
    }

    pub fn set_volume(&self, volume: f32) {
        // The default volume range is extremely loud, I found 0 - 10 to be a good range
        let _ = self.tx_play.send(Command::SetVolume(volume / 10.0));
    }
}
