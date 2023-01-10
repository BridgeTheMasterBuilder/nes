use std::error::Error;

use crate::SAMPLERATE;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::Sdl;

pub(super) struct Speaker {
    pub muted: bool,
    pub output: Option<[f32; 6]>,
    pub volume: f32,
    audio_buf: Vec<f32>,
    audio_queue: AudioQueue<f32>,
    clockrate: u32,
    counter: u32,
    sample_bufs: [f32; 6],
}

impl Speaker {
    pub fn new(sdl_context: &Sdl, clockrate: u32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            muted: false,
            output: None,
            volume: 1.0,
            audio_queue: {
                let audio_subsystem = sdl_context.audio()?;

                let desired_spec = AudioSpecDesired {
                    freq: Some(SAMPLERATE as i32),
                    channels: Some(1),
                    samples: None,
                };

                audio_subsystem.open_queue(None, &desired_spec)?
            },
            audio_buf: Vec::with_capacity(1024),
            clockrate,
            counter: 0,
            sample_bufs: [0.0; 6],
        })
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        self.audio_queue.queue_audio(&self.audio_buf[..])?;
        self.audio_buf.clear();

        if self.audio_queue.size() >= 4096 {
            self.audio_queue.resume();
        }

        Ok(())
    }

    pub fn push_sample(&mut self, audio_samples: &[f32; 6]) -> Result<(), Box<dyn Error>> {
        let d = 0.90;
        let b = 1.0 - d;

        for (output, input) in self.sample_bufs.iter_mut().zip(audio_samples.iter()) {
            *output += b * (*input - *output);
        }

        let sample = self.counter + SAMPLERATE > self.clockrate;

        if sample {
            let output = if self.muted {
                0.0
            } else {
                let [pulse1, pulse2, triangle, noise, dmc, output] = self.sample_bufs;

                self.output.replace([
                    (pulse1 / 15.0) * 2.0 - 1.0,
                    (pulse2 / 15.0) * 2.0 - 1.0,
                    (triangle / 15.0) * 2.0 - 1.0,
                    (noise / 15.0) * 2.0 - 1.0,
                    (dmc / 128.0) * 2.0 - 1.0,
                    output * 2.0 - 1.0,
                ]);

                output * (self.volume / 1.0)
            };

            self.audio_buf.push(output);
        }

        self.counter = (self.counter + SAMPLERATE) % self.clockrate;

        if self.audio_buf.len() == 1024 {
            self.audio_queue.queue_audio(&self.audio_buf[..])?;
            self.audio_buf.clear();
        }

        Ok(())
    }

    pub fn _set_clockrate(&mut self, clockrate: u32) {
        self.clockrate = clockrate;
    }
}
