
extern crate dsp;
extern crate num;

use dsp::{Dsp, Sample, Settings};

pub type RmsUnit = dsp::Wave;

/// A type for calculating RMS of a buffer of audio samples and storing it.
#[derive(Clone, Debug)]
pub struct Rms {
    channels: Vec<RmsUnit>,
}

impl Rms {

    /// Construct an Rms reader for eading from buffers with a given number of interleaved
    /// channels.
    pub fn new(channels: usize) -> Rms {
        Rms { channels: vec![0.0; channels] }
    }

    /// Update the stored RMS with the RMS of the given buffer of samples.
    pub fn update_rms<S>(&mut self, samples: &[S], settings: Settings) where S: Sample {
        let channels = settings.channels as usize;
        if self.channels.len() != channels {
            // We need to reallocate our Vec with the correct number of channels.
            self.channels = vec![0.0; channels];
        }
        // Determine the RMS for each channel, avoiding any allocations.
        for i in 0..channels {
            use num::Float;
            let sum = samples.chunks(channels)
                .map(|frame| frame[i])
                .fold(0.0, |total, sample| total + sample.to_wave().powf(2.0));
            let rms = (sum / settings.frames as RmsUnit).sqrt();
            self.channels[i] = rms;
        }
    }

    /// Return the average RMS across all channels.
    pub fn avg(&self) -> RmsUnit {
        self.channels.iter().fold(0.0, |total, &rms| total + rms) / self.channels.len() as f32
    }

    /// Return the RMS for each channel.
    pub fn per_channel(&self) -> Vec<RmsUnit> {
        self.channels.clone()
    }

}

impl<S> Dsp<S> for Rms where S: Sample {
    fn audio_requested(&mut self, samples: &mut [S], settings: Settings) {
        self.update_rms(samples, settings);
    }
}

