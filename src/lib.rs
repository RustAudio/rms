
extern crate dsp;

use dsp::{Sample, Settings};
use std::collections::VecDeque;

/// The floating point **Wave** representing the continuous RMS.
pub type Wave = dsp::Wave;

/// A type for calculating RMS of a buffer of audio samples and storing it.
#[derive(Clone, Debug)]
pub struct Rms {
    /// The number of samples used to calculate the RMS per sample.
    n_window_samples: usize,
    /// The RMS at each sample within the interleaved buffer.
    ///
    /// After an **Rms::update** this will represent the RMS at each sample for the previous
    /// `n_window_samples` number of samples.
    interleaved_rms: Vec<Wave>,
    /// A **Channel** for each channel given by the Settings.
    window_per_channel: Vec<Window>,
}

/// A wrapper around the ringbuffer of samples used to calculate the RMS per sample.
#[derive(Clone, Debug)]
pub struct Window {
    /// The sample squares (i.e. `sample.powf(2.0)`) used to calculate the RMS per sample.
    ///
    /// When a new sample is received, the **Window** pops the front sample_square and adds the new
    /// sample_square to the back.
    sample_squares: VecDeque<Wave>,
    /// The sum total of all sample_squares currently within the **Window**'s ring buffer.
    sum: Wave,
}


impl Window {

    /// Construct a new empty RMS **Window**.
    pub fn new(n_window_samples: usize) -> Self {
        Window {
            sample_squares: (0..n_window_samples).map(|_| 0.0).collect(),
            sum: 0.0,
        }
    }

    /// Zeroes the sum and the buffer of sample_squares.
    pub fn reset(&mut self) {
        for sample_square in &mut self.sample_squares {
            *sample_square = 0.0;
        }
        self.sum = 0.0;
    }

    /// The next RMS given the new sample in the sequence.
    ///
    /// The **Window** pops the front sample and adds the new sample to the back.
    ///
    /// The yielded RMS is the RMS of all sample_squares in the window after the new sample is added.
    pub fn next_rms(&mut self, new_sample: Wave) -> Wave {
        let removed_sample_square = self.sample_squares.pop_front().unwrap();
        self.sum -= removed_sample_square;
        let new_sample_square = new_sample.powf(2.0);
        self.sample_squares.push_back(new_sample_square);
        self.sum += new_sample_square;
        let rms = self.sum / self.sample_squares.len() as Wave;
        rms
    }

}


impl Rms {

    /// Construct a new **Rms** with the given window size as a number of samples.
    pub fn new(n_window_samples: usize) -> Self {
        Rms {
            n_window_samples: n_window_samples,
            interleaved_rms: Vec::new(),
            window_per_channel: Vec::new(),
        }
    }

    /// The same as **Rms::new** but also prepares the **Rms** for the given number of channels and
    /// frames.
    pub fn with_capacity(n_window_samples: usize, n_channels: usize, n_frames: usize) -> Self {
        let window_per_channel = (0..n_channels).map(|_| Window::new(n_window_samples)).collect();
        let n_samples = n_frames * n_channels;
        let interleaved_rms = Vec::with_capacity(n_samples);
        Rms {
            n_window_samples: n_window_samples,
            window_per_channel: window_per_channel,
            interleaved_rms: interleaved_rms,
        }
    }

    /// Resets the RMS **Window**s for each **Channel**.
    pub fn reset_windows(&mut self) {
        for window in &mut self.window_per_channel {
            window.reset();
        }
    }

    /// Update the stored RMS with the given interleaved buffer of samples.
    pub fn update<S>(&mut self, samples: &[S], n_channels: usize, n_frames: usize)
        where S: Sample,
    {

        // Resizes a **Vec** using the given function.
        fn resize_vec<T, F>(vec: &mut Vec<T>, new_len: usize, mut new_elem: F)
            where F: FnMut() -> T,
        {
            let len = vec.len();
            if len > new_len {
                vec.truncate(new_len);
            } else if len < new_len {
                let extension = (len..new_len).map(|_| new_elem());
                vec.extend(extension);
            }
        }

        // Ensure our `channels` match the `n_channels`.
        if self.window_per_channel.len() != n_channels {
            let n_window_samples = self.n_window_samples;
            resize_vec(&mut self.window_per_channel, n_channels, || Window::new(n_window_samples));
        }

        // Ensure each channel's `rms_per_sample` buffer matches `n_frames`.
        let n_samples = n_frames * n_channels;
        if self.interleaved_rms.len() != n_samples {
            resize_vec(&mut self.interleaved_rms, n_samples, || 0.0);
        }

        let mut idx = 0;
        for _ in 0..n_frames {
            for j in 0..n_channels {
                let sample = samples[idx].to_wave();
                let rms = self.window_per_channel[j].next_rms(sample);
                self.interleaved_rms[idx] = rms;
                idx += 1;
            }
        }
    }

    /// Return the average RMS across all channels at the given frame.
    pub fn avg(&self, frame_idx: usize) -> Wave {
        let frame_slice = self.per_channel(frame_idx);
        let total_rms = frame_slice.iter().fold(0.0, |total, &rms| total + rms);
        let avg_rms = total_rms / self.window_per_channel.len() as Wave;
        avg_rms
    }

    /// Return the RMS for each channel.
    pub fn per_channel(&self, frame_idx: usize) -> &[Wave] {
        let n_channels = self.window_per_channel.len();
        let slice_idx = frame_idx * n_channels;
        let end_idx = slice_idx + n_channels;
        let frame_slice = &self.interleaved_rms[slice_idx..end_idx];
        frame_slice
    }

    /// Borrow the internal RMS interleaved buffer.
    pub fn interleaved_rms(&self) -> &[Wave] {
        &self.interleaved_rms
    }

}

impl<S> dsp::Node<S> for Rms where S: Sample {
    fn audio_requested(&mut self, samples: &mut [S], settings: Settings) {
        self.update(samples, settings.channels as usize, settings.frames as usize);
    }
}

