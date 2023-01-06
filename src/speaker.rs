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
    sample_bufs: [Vec<f32>; 5],
}

impl Speaker {
    pub fn new(sdl_context: &Sdl, clockrate: u32) -> Result<Self, Box<dyn Error>> {
        let step = ((clockrate / SAMPLERATE) as f64) as usize;

        Ok(Self {
            muted: false,
            output: None,
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
            sample_bufs: [
                Vec::with_capacity(step),
                Vec::with_capacity(step),
                Vec::with_capacity(step),
                Vec::with_capacity(step),
                Vec::with_capacity(step),
            ],
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

    fn mix(audio_samples: [f32; 5]) -> f32 {
        let [pulse1, pulse2, triangle, noise, dmc] = audio_samples;

        let pulse_out = 95.88 / ((8128.0 / (pulse1 + pulse2)) + 100.0);
        let tnd_out =
            159.79 / ((1.0 / ((triangle / 8227.0) + (noise / 12241.0) + (dmc / 22638.0))) + 100.0);

        let mixed = pulse_out + tnd_out;

        mixed * 2.0 - 1.0
    }

    pub fn push_sample(&mut self, audio_samples: [f32; 5]) -> Result<(), Box<dyn Error>> {
        for (channel, &sample) in audio_samples.iter().enumerate() {
            self.sample_bufs[channel].push(sample);
        }

        let sample = self.counter + SAMPLERATE > self.clockrate;

        if sample {
            let output = if self.muted {
                0.0
            } else {
                let pulse1 =
                    self.sample_bufs[0].iter().sum::<f32>() / self.sample_bufs[0].len() as f32;
                let pulse2 =
                    self.sample_bufs[1].iter().sum::<f32>() / self.sample_bufs[1].len() as f32;
                let triangle =
                    self.sample_bufs[2].iter().sum::<f32>() / self.sample_bufs[2].len() as f32;
                let noise =
                    self.sample_bufs[3].iter().sum::<f32>() / self.sample_bufs[3].len() as f32;
                let dmc =
                    self.sample_bufs[4].iter().sum::<f32>() / self.sample_bufs[4].len() as f32;

                let output = Self::mix([pulse1, pulse2, triangle, noise, dmc]);

                self.output.replace([
                    (pulse1 / 15.0) * 2.0 - 1.0,
                    (pulse2 / 15.0) * 2.0 - 1.0,
                    (triangle / 15.0) * 2.0 - 1.0,
                    (noise / 15.0) * 2.0 - 1.0,
                    (dmc / 128.0) * 2.0 - 1.0,
                    output,
                ]);

                output * (self.volume / 1.0)
            };

            self.audio_buf.push(output);

            for buf in &mut self.sample_bufs {
                buf.clear();
            }
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
