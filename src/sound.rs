use std::time::Duration;
use std::sync::mpsc::{Sender, channel};
use rodio::{
    queue::queue,
    source::{Source, SineWave},
    OutputStream,
    Sink,
};

pub struct BeepSound {
    chan_tx: Sender<()>
}

impl BeepSound {
    const FREQUENCY: u32 = 440;
    const VOLUME: f32 = 0.05;

    pub fn new() -> Result<Self, String> {
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let (queue, output_queue) = queue(true);
            if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    sink.set_volume(BeepSound::VOLUME);
                    sink.append(output_queue);
        
                    loop {
                        if rx.recv().is_ok() {
                            queue.append(SineWave::new(BeepSound::FREQUENCY).take_duration(Duration::from_secs_f32(0.025)));
                        }
                    }
                }
            }
        });

        Ok(Self {
            chan_tx: tx,
        })
    }

    pub fn beep(&self) {
        // Ignore if something went wrong
        let _ = self.chan_tx.send(());
    }
}
