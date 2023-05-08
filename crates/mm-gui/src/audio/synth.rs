use std::f64::consts::TAU;

use super::event::{EventKind, EventList};
use super::{AudioState, Buffer};
use crate::structures::Latest;

pub fn synth<const VOICES: usize>(
    state: AudioState,
    new_events: Latest<EventList>,
) -> impl FnMut(&mut [f64]) {
    let gain: f64 = 1.0 / VOICES as f64;

    let mut factors = [0.0; VOICES];
    let mut oscillators: [_; VOICES] =
        std::array::from_fn(|i| Voice::new(i as f64 / VOICES as f64, state.rate));

    let mut buffers: [_; VOICES] = std::array::from_fn(|_| Buffer::default());

    let mut events = EventList::new(Vec::new());

    move |data| {
        data.fill(0.0);

        if state.is_playing() {
            if let Some(new) = new_events.take() {
                events = new;

                // Reset envelopes
                for factor in factors.iter_mut() {
                    *factor = 0.0;
                }
            }

            let (start, end) = {
                let (start, delta) = state.beat_delta();
                (start, start + delta * data.len() as f64)
            };

            for event in events.events_from(start) {
                if event.beat >= end {
                    break;
                }

                let i = event.id as usize % VOICES;
                match event.kind {
                    EventKind::Start { frequency } => {
                        factors[i] = gain;
                        oscillators[i].frequency = frequency;
                    }

                    EventKind::Stop => {
                        factors[i] = 0.0;
                    }
                }
            }

            let from = state.time();

            for ((oscillator, buffer), factor) in oscillators
                .iter_mut()
                .zip(buffers.iter_mut())
                .zip(factors.iter())
            {
                let buffer = buffer.with_len(data.len());
                oscillator.evaluate(from, buffer);

                for (v, s) in data.iter_mut().zip(buffer.iter().copied()) {
                    *v += *factor * s;
                }
            }
        }
    }
}

struct Voice {
    frequency: f64,
    phase: f64,
    rate: u32,
}

impl Voice {
    pub fn new(phase: f64, rate: u32) -> Self {
        Self {
            frequency: 0.0,
            phase,
            rate,
        }
    }

    pub fn evaluate(&mut self, mut from: f64, data: &mut [f64]) {
        let delta = 1.0 / self.rate as f64;
        data.fill_with(|| {
            let v = (TAU * self.frequency * from + self.phase).sin();
            from += delta;
            v
        });
    }
}
