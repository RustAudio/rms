//! Read the RMS from the input stream buffer (and pass the input buffer straight to the output).

extern crate dsp;
extern crate rms;

use dsp::{CallbackFlags, CallbackResult, Settings, SoundStream, StreamParams};
use rms::Rms;

fn main() {

    // The number of channels we want in our stream.
    const CHANNELS: u16 = 2;
    // The size of the **Rms**' moving **Window**.
    const WINDOW_SIZE: usize = 1024;

    // Construct our Rms reader.
    let mut rms = Rms::new(WINDOW_SIZE);

    // Callback used to construct the duplex sound stream.
    let callback = Box::new(move |input: &[f32], in_settings: Settings,
                                  output: &mut[f32], _out_settings: Settings,
                                  _dt: f64,
                                  _: CallbackFlags| {

        // Update our rms state.
        let n_channels = in_settings.channels as usize;
        let n_frames = in_settings.frames as usize;
        rms.update(input, n_channels, n_frames);

        println!("Input RMS avg: {:?}, RMS per channel: {:?}",
                 rms.avg(n_frames - 1),
                 rms.per_channel(n_frames - 1));

        // Write the input to the output for fun.
        for (out_sample, in_sample) in output.iter_mut().zip(input.iter()) {
            *out_sample = *in_sample;
        }

        CallbackResult::Continue
    });

    // Construct parameters for a duplex stream and the stream itself.
    let params = StreamParams::new().channels(CHANNELS as i32);
    let stream = SoundStream::new()
        .sample_hz(44_100.0)
        .frames_per_buffer(128)
        .duplex(params, params)
        .run_callback(callback)
        .unwrap();

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep_ms(16);
    }

}

