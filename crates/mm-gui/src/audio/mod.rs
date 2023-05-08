pub mod event;

mod synth;

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, Stream};

use self::event::EventList;
use crate::structures::Latest;

const BPM_FRACTION_BITS: u32 = 12;
const BPM_DEFAULT: u32 = 120 << BPM_FRACTION_BITS;

pub struct AudioState {
    play: Arc<AtomicBool>,
    time: Arc<AtomicU64>,
    bpm: Arc<AtomicU32>,

    rate: u32,
}

impl AudioState {
    /// Create a shallow copy of this audio state which refers to the same
    /// underlying state.
    pub fn shallow_copy(&self) -> Self {
        Self {
            play: Arc::clone(&self.play),
            time: Arc::clone(&self.time),
            bpm: Arc::clone(&self.bpm),

            rate: self.rate,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.play.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.play.store(false, Ordering::Relaxed);
        self.time.store(0, Ordering::Relaxed);
    }

    pub fn toggle_playing(&self) {
        self.play.fetch_xor(true, Ordering::Relaxed);
    }

    pub fn bpm(&self) -> f64 {
        self.bpm.load(Ordering::Relaxed) as f64 / (1 << BPM_FRACTION_BITS) as f64
    }

    pub fn set_bpm(&self, value: f64) {
        let value = value * (1 << BPM_FRACTION_BITS) as f64;
        let value = value as u32;
        self.bpm.store(value, Ordering::Relaxed);
    }

    /// Get the time in the number of beats, given the current BPM and time
    /// in seconds.
    pub fn beat(&self) -> f64 {
        self.beat_delta().0
    }

    /// Get the time in the number of beats and the beat delta between
    /// successive samples. See also [`AudioState::beat`].
    pub fn beat_delta(&self) -> (f64, f64) {
        let t = self.time();
        let b = self.bpm();
        (b * t / 60.0, b / (60.0 * self.rate as f64))
    }

    pub fn time(&self) -> f64 {
        self.time.load(Ordering::Relaxed) as f64 / self.rate as f64
    }
}

pub struct AudioThread {
    state: AudioState,
    _stream: Stream,
}

impl AudioThread {
    pub fn state(&self) -> &AudioState {
        &self.state
    }
}

pub fn play() -> (AudioThread, Latest<EventList>) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no available output device");

    let config = device
        .supported_output_configs()
        .expect("error while querying configs")
        .next()
        .expect("no supported config")
        .with_max_sample_rate();

    let format = config.sample_format();
    let rate = config.sample_rate().0;
    let config = config.into();

    let state = AudioState {
        play: Arc::new(AtomicBool::new(false)),
        time: Arc::new(AtomicU64::new(0)),
        bpm: Arc::new(AtomicU32::new(BPM_DEFAULT)),
        rate,
    };

    let events = Latest::new();

    let err_fn = |e| eprintln!("An error occured on the output stream: {e}");
    let stream = match format {
        SampleFormat::F32 => device.build_output_stream(
            &config,
            audio_fn::<f32>(state.shallow_copy(), events.clone()),
            err_fn,
            None,
        ),

        SampleFormat::F64 => device.build_output_stream(
            &config,
            audio_fn::<f64>(state.shallow_copy(), events.clone()),
            err_fn,
            None,
        ),

        SampleFormat::I16 => device.build_output_stream(
            &config,
            audio_fn::<i16>(state.shallow_copy(), events.clone()),
            err_fn,
            None,
        ),

        SampleFormat::U16 => device.build_output_stream(
            &config,
            audio_fn::<u16>(state.shallow_copy(), events.clone()),
            err_fn,
            None,
        ),

        fmt => panic!("Unsupported sample format {fmt:?}"),
    };

    let stream = stream.expect("Unable to build output stream");
    stream.play().unwrap();

    let audio = AudioThread {
        state,
        _stream: stream,
    };

    (audio, events)
}

fn audio_fn<T: Sample>(
    state: AudioState,
    new_events: Latest<EventList>,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo)
where
    T: FromSample<f64>,
{
    let mut synth = synth::synth::<4>(state.shallow_copy(), new_events);
    let mut buffer = Buffer::new();

    move |data, _| {
        let buffer = buffer.with_len(data.len());
        synth(buffer);

        for (d, s) in data.iter_mut().zip(buffer.iter().copied()) {
            *d = T::from_sample(s);
        }

        if state.is_playing() {
            state.time.fetch_add(data.len() as u64, Ordering::Relaxed);
        }
    }
}

/// An audio buffer holds a bunch of samples.
struct Buffer<T> {
    data: Vec<T>,
}

impl<T: Default> Buffer<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Get a reference to a mutable slice `len` samples long.
    pub fn with_len(&mut self, len: usize) -> &mut [T] {
        if self.data.len() < len {
            self.data.resize_with(len, Default::default);
        }

        &mut self.data[0..len]
    }
}

impl<T> Default for Buffer<T> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}
