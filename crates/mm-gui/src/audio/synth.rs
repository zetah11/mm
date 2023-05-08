use std::array;

use super::event::{EventKind, EventList};
use super::AudioState;
use crate::structures::Latest;

pub fn synth<const VOICES: usize>(
    state: AudioState,
    new_events: Latest<EventList>,
) -> impl FnMut(&mut [f64]) {
    let mut oscillators: [_; VOICES] = array::from_fn(|i| Voice::new(i as f64 / VOICES as f64));
    let mut events = EventList::new(Vec::new());

    move |data| {
        data.fill(0.0);

        if state.is_playing() {
            if let Some(new) = new_events.take() {
                events = new;
                for oscilattor in oscillators.iter_mut() {
                    oscilattor.env.stop();
                }
            }

            let mut time = state.time();
            let delta_time = 1.0 / state.rate as f64;

            let (mut beat, delta_beat) = state.beat_delta();

            let mut events = events.events_from(beat);
            for v in data.iter_mut() {
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
                    *v += oscillator.tick(time, delta_time);
                }

                beat += delta_beat;
                time += delta_time;
            }
        }
    }
}

struct Voice {
    frequency: f64,
    phase: f64,

    env: Adsr,
}

impl Voice {
    pub fn new(phase: f64) -> Self {
        Self {
            frequency: 0.0,
            phase,

            env: Adsr::new(0.5, 0.1, 0.5, 0.2),
        }
    }

    pub fn tick(&mut self, from: f64, delta: f64) -> f64 {
        let fx = self.frequency * from + self.phase;
        let v = 4.0 * (fx - (fx + 0.5).floor()).abs() - 1.0;
        let v = v * self.env.attn();
        self.env.step(delta);
        v
    }
}

struct Adsr {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,

    state: EnvelopeState,
    last: f64,
    time: f64,
}

impl Adsr {
    pub fn new(a: f64, d: f64, s: f64, r: f64) -> Self {
        Self {
            attack: a.max(0.0),
            decay: d.max(0.0),
            sustain: s.clamp(0.0, 1.0),
            release: r.max(0.0),

            state: EnvelopeState::Quiet,
            last: 0.0,
            time: 0.0,
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

    pub fn step(&mut self, delta: f64) {
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
                    self.time = 0.0;
                }
            }

            EnvelopeState::Quiet => {}
        }
    }

    pub fn start(&mut self) {
        self.state = EnvelopeState::Playing;
        self.time = 0.0;
    }

    pub fn stop(&mut self) {
        if matches!(self.state, EnvelopeState::Playing) {
            self.state = EnvelopeState::Released;
            self.time = 0.0;
        }
    }
}

#[derive(Clone, Copy)]
enum EnvelopeState {
    Playing,
    Released,
    Quiet,
}
