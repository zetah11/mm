use std::cmp::Ordering;
use std::collections::BinaryHeap;

use midly::num::{u28, u4, u7};
use midly::{MidiMessage, TrackEvent, TrackEventKind};
use mm_eval::span::Span;
use mm_eval::{Length, Time};
use num_rational::BigRational;
use num_traits::{FromPrimitive, ToPrimitive};

use super::Pitch;

/// Write the notes from the given iterator to the given track on a specific
/// channel.
///
/// `ticks_per_beat` determines how many ticks a note of length `1` should last.
pub fn write_channel<'src>(
    notes: impl Iterator<Item = (Pitch, Span<'src>, Time, Length)>,
    ticks_per_beat: usize,
    channel: u4,
    track: &mut Vec<TrackEvent>,
) {
    let mut events = BinaryHeap::new();

    for (note, _, start, length) in notes {
        let off = PitchEvent {
            at: &start + &length,
            kind: PitchEventKind::Off(note),
        };

        let on = PitchEvent {
            at: start,
            kind: PitchEventKind::On(note),
        };

        events.push(on);
        events.push(off);
    }

    let mut at = 0;
    while let Some(event) = events.pop() {
        let now = (event.at.0 * BigRational::from_usize(ticks_per_beat).unwrap())
            .to_usize()
            .expect("start times are reasonably small");

        let delta = now - at;
        let delta = u28::new(u32::try_from(delta).unwrap());
        at = now;

        let kind = match event.kind {
            PitchEventKind::On(note) => TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn {
                    key: note.to_midi_key_saturating(),
                    vel: u7::new(100),
                },
            },

            PitchEventKind::Off(note) => TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOff {
                    key: note.to_midi_key_saturating(),
                    vel: u7::new(100),
                },
            },
        };

        track.push(TrackEvent { delta, kind });
    }
}

struct PitchEvent {
    at: Time,
    kind: PitchEventKind,
}

enum PitchEventKind {
    On(Pitch),
    Off(Pitch),
}

impl Eq for PitchEvent {}

impl PartialEq for PitchEvent {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for PitchEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PitchEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.at
            .0
            .cmp(&other.at.0)
            .then(self.kind.cmp(&other.kind))
            .reverse()
    }
}

impl Eq for PitchEventKind {}

impl PartialEq for PitchEventKind {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for PitchEventKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PitchEventKind {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Off(_), Self::On(_)) => Ordering::Less,
            (Self::On(_), Self::Off(_)) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}
