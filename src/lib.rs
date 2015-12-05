
extern crate dsp;
extern crate time_calc as time;

use dsp::{Sample, Settings};
use std::collections::VecDeque;
use time::Ms;

/// The floating point **Wave** representing the continuous RMS.
pub type Wave = dsp::Wave;

/// A type for calculating RMS of a buffer of audio samples and storing it.
#[derive(Clone, Debug)]
pub struct Rms {
    /// The duration of the window used to calculate the RMS in milliseconds.
    window_ms: Ms,
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

    /// Set the buffer size used to some new size.
    pub fn set_len(&mut self, n_window_samples: usize) {
        let len = self.sample_squares.len();
        if len > n_window_samples {
            let diff = len - n_window_samples;
            for _ in 0..diff {
                self.pop_front();
            }
        } else if len < n_window_samples {
            let diff = n_window_samples - len;
            for _ in 0..diff {
                // Push the new fake samples onto the front so they are the first to be removed.
                // We'll generate the fake samples as the current RMS to avoid affecting the
                // Window's RMS output as much as possible.
                let rms = self.calc_rms();
                self.sample_squares.push_front(rms);
            }
        }
    }

    /// Remove the front sample and subtract it from the `sum`.
    fn pop_front(&mut self) {
        let removed_sample_square = self.sample_squares.pop_front().unwrap();
        self.sum -= removed_sample_square;

        // Don't let floating point rounding errors put us below 0.0.
        if self.sum < 0.0 {
            self.sum = 0.0;
        }
    }

    /// Determines the square of the given sample, pushes it back onto our buffer and adds it to
    /// the `sum`.
    fn push_back(&mut self, new_sample: Wave) {
        // Push back the new sample_square and add it to the `sum`.
        let new_sample_square = new_sample.powf(2.0);
        self.sample_squares.push_back(new_sample_square);
        self.sum += new_sample_square;
    }

    /// Calculate the RMS for the **Window** in its current state.
    fn calc_rms(&self) -> Wave {
        (self.sum / self.sample_squares.len() as Wave).sqrt()
    }

    /// The next RMS given the new sample in the sequence.
    ///
    /// The **Window** pops the front sample and adds the new sample to the back.
    ///
    /// The yielded RMS is the RMS of all sample_squares in the window after the new sample is added.
    ///
    /// Returns `0.0` if the **Window**'s `sample_squares` buffer is empty.
    pub fn next_rms(&mut self, new_sample: Wave) -> Wave {
        // If our **Window** has no length, there's nothing to calculate.
        if self.sample_squares.len() == 0 {
            return 0.0;
        }

        self.pop_front();
        self.push_back(new_sample);

        self.calc_rms()
    }

}


impl Rms {

    /// Construct a new **Rms** with the given window size as a number of samples.
    pub fn new<I: Into<Ms>>(window_ms: I) -> Self {
        Rms {
            window_ms: window_ms.into(),
            interleaved_rms: Vec::new(),
            window_per_channel: Vec::new(),
        }
    }

    /// The same as **Rms::new** but also prepares the **Rms** for the given number of channels and
    /// frames.
    pub fn with_capacity<I: Into<Ms>>(window_ms: I, settings: Settings) -> Self {
        let n_channels = settings.channels as usize;
        let window_ms: Ms = window_ms.into();
        let window_samples = window_ms.samples(settings.sample_hz as f64) as usize;
        let window_per_channel = (0..n_channels).map(|_| Window::new(window_samples)).collect();
        let n_samples = settings.frames as usize * n_channels;
        let interleaved_rms = Vec::with_capacity(n_samples);
        Rms {
            window_ms: window_ms.into(),
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
    pub fn update<S>(&mut self, samples: &[S], settings: Settings)
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

        let n_window_samples = self.window_ms.samples(settings.sample_hz as f64) as usize;

        // Ensure our `channels` match the `n_channels`.
        let n_channels = settings.channels as usize;
        if self.window_per_channel.len() != n_channels {
            resize_vec(&mut self.window_per_channel, n_channels, || Window::new(n_window_samples));
        }

        // Make sure the window buffer sizes match `n_window_samples`.
        for window in &mut self.window_per_channel {
            if window.sample_squares.len() != n_window_samples {
                window.set_len(n_window_samples);
            }
        }

        // Ensure each channel's `rms_per_sample` buffer matches `n_frames`.
        let n_frames = settings.frames as usize;
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
    ///
    /// **Panics** if the given frame index is out of bounds.
    pub fn avg(&self, frame_idx: usize) -> Wave {
        let frame_slice = self.per_channel(frame_idx);
        let total_rms = frame_slice.iter().fold(0.0, |total, &rms| total + rms);
        let n_channels = self.window_per_channel.len();
        let avg_rms = total_rms / n_channels as Wave;
        avg_rms
    }

    /// Return the RMS for each channel at the given frame.
    ///
    /// **Panics** if the given frame index is out of bounds.
    pub fn per_channel(&self, frame_idx: usize) -> &[Wave] {
        let n_channels = self.window_per_channel.len();
        let slice_idx = frame_idx * n_channels;
        let end_idx = slice_idx + n_channels;
        let frame_slice = &self.interleaved_rms[slice_idx..end_idx];
        frame_slice
    }

    /// The index of the last frame if there is one.
    fn last_frame(&self) -> Option<usize> {
        let n_channels = self.window_per_channel.len();
        let n_samples = self.interleaved_rms.len();

        // If we don't have any channels or samples, just return None.
        if n_channels == 0 || n_samples == 0 {
            return None;
        }

        let n_frames = n_samples / n_channels;
        let last_frame = n_frames - 1;
        Some(last_frame)
    }

    /// The average RMS across all channels at the last frame.
    ///
    /// Returns `0.0` if there are no frames.
    pub fn avg_at_last_frame(&self) -> Wave {
        self.last_frame()
            .map(|last_frame| self.avg(last_frame))
            .unwrap_or(0.0)
    }

    /// The RMS for each channel at the last frame.
    ///
    /// Returns an empty slice if there are no frames.
    pub fn per_channel_at_last_frame(&self) -> &[Wave] {
        self.last_frame()
            .map(|last_frame| self.per_channel(last_frame))
            .unwrap_or(&[])
    }

    /// Borrow the internal RMS interleaved buffer.
    pub fn interleaved_rms(&self) -> &[Wave] {
        &self.interleaved_rms
    }

    /// The window size in milliseconds.
    pub fn window_ms(&self) -> time::calc::Ms {
        self.window_ms.ms()
    }

}

impl<S> dsp::Node<S> for Rms where S: Sample {
    fn audio_requested(&mut self, samples: &mut [S], settings: Settings) {
        self.update(samples, settings);
    }
}

