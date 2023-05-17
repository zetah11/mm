pub mod event;

pub use self::time::{Beat, Bpm, Hz, Second};

mod delay;
mod synth;
mod time;

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, Stream};

use self::event::EventList;
use crate::structures::Latest;

pub type StereoIn<'a> = (&'a [f64], &'a [f64]);
pub type StereoOut<'a> = (&'a mut [f64], &'a mut [f64]);

pub struct AudioState {
    play: Arc<AtomicBool>,
    time: Arc<AtomicU64>,
    bpm: Arc<AtomicU32>,

    rate: Hz,
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

    pub fn bpm(&self) -> Bpm {
        Bpm::from(self.bpm.load(Ordering::Relaxed))
    }

    pub fn set_bpm(&self, value: Bpm) {
        self.bpm.store(value.into(), Ordering::Relaxed);
    }

    /// Get the time in the number of beats, given the current BPM and time
    /// in seconds.
    pub fn beat(&self) -> Beat {
        self.beat_delta().0
    }

    /// Get the current time in number of beats as well as the length of a
    /// sample in terms of a beat. See also [`AudioState::beat`].
    pub fn beat_delta(&self) -> (Beat, Beat) {
        let t = self.time();
        let b = self.bpm();
        (b * t, b.beats_per_sample(self.rate))
    }

    pub fn time(&self) -> Second {
        self.time.load(Ordering::Relaxed) as f64 / self.rate
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
    let rate = config.sample_rate().0.into();
    let channels = config.channels() as usize;
    let config = config.into();

    let state = AudioState {
        play: Arc::new(AtomicBool::new(false)),
        time: Arc::new(AtomicU64::new(0)),
        bpm: Arc::new(AtomicU32::new(Bpm::default().into())),
        rate,
    };

    let events = Latest::new();

    let err_fn = |e| eprintln!("An error occured on the output stream: {e}");
    let stream = match format {
        SampleFormat::F32 => device.build_output_stream(
            &config,
            audio_fn::<f32>(channels, state.shallow_copy(), events.clone()),
            err_fn,
            None,
        ),

        SampleFormat::F64 => device.build_output_stream(
            &config,
            audio_fn::<f64>(channels, state.shallow_copy(), events.clone()),
            err_fn,
            None,
        ),

        SampleFormat::I16 => device.build_output_stream(
            &config,
            audio_fn::<i16>(channels, state.shallow_copy(), events.clone()),
            err_fn,
            None,
        ),

        SampleFormat::U16 => device.build_output_stream(
            &config,
            audio_fn::<u16>(channels, state.shallow_copy(), events.clone()),
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
    channels: usize,
    state: AudioState,
    new_events: Latest<EventList>,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo)
where
    T: FromSample<f64>,
{
    let mut synth = synth::synth::<16>(state.shallow_copy(), new_events);
    let mut delay = delay::rotating(
        state.rate,
        std::f64::consts::FRAC_PI_3,
        0.5.into(),
        0.3.into(),
        0.4,
        0.6,
    );

    let mut left = Buffer::new();
    let mut right = Buffer::new();

    let mut final_left = Buffer::new();
    let mut final_right = Buffer::new();

    move |data, _| {
        let sample_count = (data.len() + (channels - 1)) / channels;

        let left = left.with_len(sample_count);
        let right = right.with_len(sample_count);

        let final_left = final_left.with_len(sample_count);
        let final_right = final_right.with_len(sample_count);

        synth((left, right));
        delay((left, right), (final_left, final_right));

        let samples = final_left.iter().copied().zip(final_right.iter().copied());
        for (mut d, (l, r)) in data.chunks_exact_mut(channels).zip(samples) {
            let l = (0.3 * l).tanh();
            let r = (0.3 * r).tanh();

            if channels >= 2 {
                d[0] = T::from_sample(l);
                d[1] = T::from_sample(r);
                d = &mut d[2..];
            }

            if channels <= 1 || channels > 2 {
                let avg = (l + r) / 2.0;
                d.fill_with(|| T::from_sample(avg));
            }
        }

        if state.is_playing() {
            state.time.fetch_add(sample_count as u64, Ordering::Relaxed);
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
