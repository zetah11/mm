use std::cmp::Ordering;

use super::{Beat, Hz};

#[derive(Debug)]
pub struct EventList {
    events: Vec<Event>,
}

impl EventList {
    pub fn new(events: Vec<Event>) -> Self {
        debug_assert!(is_sorted(&events));
        Self { events }
    }

    /// Get a slice of the events after and including this beat.
    pub fn events_from(&self, beat: Beat) -> &[Event] {
        let i = self.events.partition_point(|event| event.beat < beat);
        &self.events[i..]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub beat: Beat,

    /// An id which locally uniquely identifies events related to each other.
    /// For example, a start and a stop event for the same note will have the
    /// same id. Events which are unrelated but close in time will have
    /// different ids. The same id may be used several times later on for
    /// unrelated events.
    pub id: u32,
}

impl Eq for Event {}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.beat.cmp(&other.beat).then(self.kind.cmp(&other.kind))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EventKind {
    Start { frequency: Hz },
    Stop,
}

impl Eq for EventKind {}

impl Ord for EventKind {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Start { frequency: f1 }, Self::Start { frequency: f2 }) => f1.cmp(f2),
            (Self::Stop, Self::Stop) => Ordering::Equal,

            (Self::Start { .. }, Self::Stop) => Ordering::Greater,
            (Self::Stop, Self::Start { .. }) => Ordering::Less,
        }
    }
}

impl PartialEq for EventKind {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for EventKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Returns true if the given slice is sorted from least to greatest.
fn is_sorted<T: Ord>(ts: &[T]) -> bool {
    let mut prev = None;

    for t in ts {
        if let Some(prev) = prev.as_mut() {
            if *prev > t {
                return false;
            } else {
                *prev = t;
            }
        } else {
            prev = Some(t);
        }
    }

    true
}
