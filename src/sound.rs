use std::time::Duration;
use std::sync::mpsc::{Sender, channel};
use rodio::{
    queue::queue,
    source::{Source, SineWave},
    OutputStream,
    Sink,
    buffer::SamplesBuffer,
};
use bitvec::{vec::BitVec, order::Msb0};

pub enum Command {
    PlayBeep,
    PlayBuffer([u8; 16]),
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
            let beep_duration = Duration::from_secs_f32(1.5/60.0);
            if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    sink.set_volume(AudioPlayer::VOLUME);
                    sink.append(output_queue);
        
                    loop {
                        if let Ok(cmd) = rx.recv() {
                            match cmd {
                                Command::PlayBeep => queue.append(SineWave::new(Self::BEEP_FREQ).take_duration(beep_duration)),
                                Command::PlayBuffer(buf) => {
                                    let reps = sample_rate / Self::BUF_FREQ;
                                    let samples: Vec<f32> = BitVec::<Msb0, _>::from_vec(buf.to_vec())
                                        .into_iter()
                                        .map(|v| vec![if v { Self::VOLUME } else { 0.0 }; reps as usize])
                                        .flatten()
                                        .collect();
                                    let sample_buffer = SamplesBuffer::new(1, sample_rate, samples);
                                    queue.append(sample_buffer.take_duration(Duration::from_secs_f32(1.0 / 60.0)));
                                },
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            tx_play: tx,
        })
    }

    pub fn beep(&self) {
        // Ignore if something went wrong
        let _ = self.tx_play.send(Command::PlayBeep);
    }

    pub fn play_buffer(&self, buf: [u8; 16]) {
        let _ = self.tx_play.send(Command::PlayBuffer(buf));
    }
}
