# rms [![Build Status](https://travis-ci.org/RustAudio/rms.svg?branch=master)](https://travis-ci.org/RustAudio/rms)

A simple type for calculating and storing the RMS given some buffer of interleaved audio samples.

```Rust
let mut rms = Rms::new(num_channels);
rms.udpate_rms(&sample_buffer[..], dsp_settings);
println!("Average RMS across channels: {:?}", rms.avg());
println!("RMS for each channel: {:?}", rms.per_channel());
```

The `Rms` type also implements `dsp-chain`'s `Dsp` trait, meaning it can be used as a node within a DspGraph.

