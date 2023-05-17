use std::array;

use super::event::{EventKind, EventList};
use super::{AudioState, Hz, Second, StereoOut};
use crate::structures::Latest;

pub fn synth<const VOICES: usize>(
    state: AudioState,
    new_events: Latest<EventList>,
) -> impl FnMut(StereoOut) {
    let mut oscillators: [_; VOICES] = array::from_fn(|i| Voice::new(i as f64 / VOICES as f64));
    let mut events = EventList::new(Vec::new());

    move |(left, right)| {
        left.fill(0.0);
        right.fill(0.0);

        if state.is_playing() {
            if let Some(new) = new_events.take() {
                events = new;
                for oscilattor in oscillators.iter_mut() {
                    oscilattor.env.stop();
                }
            }

            let mut time = state.time();
            let delta_time = 1.0 / state.rate;

            let (mut beat, delta_beat) = state.beat_delta();

            let mut events = events.events_from(beat);
            for (left, right) in left.iter_mut().zip(right.iter_mut()) {
                while let Some(event) = events.first() {
                    if event.beat > beat {
                        break;
                    }

                    let i = event.id as usize % VOICES;
                    match event.kind {
                        EventKind::Start { frequency } => {
                            oscillators[i].env.start();
                            oscillators[i].frequency = frequency;
                        }

                        EventKind::Stop => {
                            oscillators[i].env.stop();
                        }
                    }

                    events = &events[1..];
                }

                for oscillator in oscillators.iter_mut() {
                    let (l, r) = oscillator.tick(time, delta_time);
                    *left += l;
                    *right += r;
                }

                beat += delta_beat;
                time += delta_time;
            }
        }
    }
}

struct Voice {
    frequency: Hz,
    phase: f64,

    env: Adsr,
}

impl Voice {
    pub fn new(phase: f64) -> Self {
        Self {
            frequency: 0.0.into(),
            phase,

            env: Adsr::new(0.01.into(), 0.1.into(), 0.4, 0.02.into()),
        }
    }

    pub fn tick(&mut self, from: Second, delta: Second) -> (f64, f64) {
        let rx = self.frequency * from + self.phase;
        let lx = rx - 0.25;

        let r = -4.0 * (rx - (rx + 0.5).floor()).abs() + 1.0;
        let l = -4.0 * (lx - (lx + 0.5).floor()).abs() + 1.0;

        let e = self.env.attn();
        self.env.step(delta);

        (l * e, r * e)
    }
}

struct Adsr {
    attack: Second,
    decay: Second,
    sustain: f64,
    release: Second,

    state: EnvelopeState,
    time: Second,
    last: f64,
}

impl Adsr {
    pub fn new(a: Second, d: Second, s: f64, r: Second) -> Self {
        Self {
            attack: a.max(Second::ZERO),
            decay: d.max(Second::ZERO),
            sustain: s.clamp(0.0, 1.0),
            release: r.max(Second::ZERO),

            state: EnvelopeState::Quiet,
            time: Second::ZERO,
            last: 0.0,
        }
    }

    pub fn attn(&mut self) -> f64 {
        match self.state {
            EnvelopeState::Playing => {
                let v = if self.time < self.attack {
                    self.time / self.attack
                } else if self.time < self.attack + self.decay {
                    let t = (self.time - self.attack) / self.decay;
                    (1.0 - t) + t * self.sustain
                } else {
                    self.sustain
                };

                self.last = v;
                v
            }

            EnvelopeState::Released => {
                let t = self.time / self.release;
                (1.0 - t) * self.last
            }

            EnvelopeState::Quiet => 0.0,
        }
    }

    pub fn step(&mut self, delta: Second) {
        match self.state {
            EnvelopeState::Playing => {
                if self.time <= self.attack + self.decay {
                    self.time += delta;
                }
            }

            EnvelopeState::Released => {
                self.time += delta;
                if self.time > self.release {
                    self.state = EnvelopeState::Quiet;
                    self.time = Second::ZERO;
                }
            }

            EnvelopeState::Quiet => {}
        }
    }

    pub fn start(&mut self) {
        self.state = EnvelopeState::Playing;
        self.time = Second::ZERO;
    }

    pub fn stop(&mut self) {
        if matches!(self.state, EnvelopeState::Playing) {
            self.state = EnvelopeState::Released;
            self.time = Second::ZERO;
        }
    }
}

#[derive(Clone, Copy)]
enum EnvelopeState {
    Playing,
    Released,
    Quiet,
}
