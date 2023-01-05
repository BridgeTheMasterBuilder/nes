use std::error::Error;

use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::Sdl;

const SAMPLERATE: u32 = 44100;

pub(super) struct Speaker {
    pub volume: f32,
    audio_buf: Vec<f32>,
    audio_queue: AudioQueue<f32>,
    clockrate: u32,
    counter: u32,
    sample_buf: Vec<f32>,
}

impl Speaker {
    pub fn new(sdl_context: &Sdl, clockrate: u32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            volume: 0.5,
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
            sample_buf: {
                let step = ((clockrate / SAMPLERATE) as f64) as u32;

                Vec::with_capacity(step as usize)
            },
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

    pub fn push_sample(&mut self, audio_sample: f32) -> Result<(), Box<dyn Error>> {
        self.sample_buf.push(audio_sample);

        let sample = self.counter + SAMPLERATE > self.clockrate;

        if sample {
            let len = self.sample_buf.len();

            let avg = self.sample_buf.iter().sum::<f32>() / len as f32;

            self.audio_buf.push(avg * (self.volume / 1.0));
            self.sample_buf.clear();
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
