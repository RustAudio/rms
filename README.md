# rms [![Build Status](https://travis-ci.org/RustAudio/rms.svg?branch=master)](https://travis-ci.org/RustAudio/rms)

A simple type for calculating and storing the RMS given some buffer of interleaved audio samples.


Usage
-----

```Rust
const WINDOW_SIZE_MS: f64 = 10.0;
let mut rms = Rms::new(WINDOW_SIZE_MS);
rms.udpate(&sample_buffer[..], dsp_settings);
println!("Average RMS across channels at the last frame: {:?}", rms.avg_at_last_frame());
println!("RMS for each channel at the last frame: {:?}", rms.per_channel_at_last_frame());
```

The `Rms` type also implements `dsp-chain`'s `Dsp` trait, meaning it can be updated as a node within a DspGraph.

Add the `rms` crate to your dependencies like so:

```
[dependencies]
rms = "<version>"
```
