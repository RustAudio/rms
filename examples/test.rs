//! Read the RMS from the input stream buffer (and pass the input buffer straight to the output).

#![feature(collections)]

extern crate dsp;
extern crate rms;

use dsp::{Event, Settings, SoundStream};
use rms::Rms;

fn main() {
    const CHANNELS: u16 = 2;
    let mut stream = SoundStream::<f32, f32>::new()
        .settings(Settings { sample_hz: 44_100, frames: 512, channels: CHANNELS })
        .run().unwrap();
    let mut buffer = Vec::new();
    let mut rms = Rms::new(CHANNELS as usize);
    for event in stream.by_ref() {
        match event {
            Event::In(input) => { ::std::mem::replace(&mut buffer, input); },
            Event::Out(output, settings) => {
                rms.update_rms(&buffer[..], settings);
                println!("Input RMS avg: {:?}, RMS per channel: {:?}", rms.avg(), rms.per_channel());
                output.clone_from_slice(&buffer[..]);
            },
            _ => (),
        }
    }
}

