use std::collections::HashMap;

use crate::audio::Beat;

#[derive(Debug, Default)]
pub struct Tracks {
    tracks: HashMap<TrackId, Track>,
    order: Vec<TrackId>,

    track_names: HashMap<TrackId, String>,
    name_tracks: HashMap<String, TrackId>,

    counter: usize,
}

impl Tracks {
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
            order: Vec::new(),

            track_names: HashMap::new(),
            name_tracks: HashMap::new(),

            counter: 0,
        }
    }

    /// Create a new track with the given name. Returns `None` if the name is
    /// used by another track.
    pub fn add_track(&mut self, name: String) -> Option<TrackId> {
        if self.name_tracks.get(&name).is_some() {
            None
        } else {
            let id = TrackId(self.counter);
            self.counter += 1;

            let prev = self.tracks.insert(id, Track::default());
            debug_assert!(prev.is_none());

            self.order.push(id);

            let prev = self.track_names.insert(id, name.clone());
            debug_assert!(prev.is_none());

            let prev = self.name_tracks.insert(name, id);
            debug_assert!(prev.is_none());

            Some(id)
        }
    }

    /// Remove the track with the given id. Returns `None` if and only if it has
    /// already been removed.
    pub fn remove_track(&mut self, id: TrackId) -> Option<Track> {
        let track = self.tracks.remove(&id)?;

        let index = self
            .order
            .iter()
            .enumerate()
            .find_map(|(index, track)| (*track == id).then_some(index))
            .expect("order is in sync with tracks");

        self.order.remove(index);

        let name = self
            .track_names
            .remove(&id)
            .expect("track_names is in sync with tracks");

        self.name_tracks
            .remove(&name)
            .expect("name_tracks is in sync with tracks");

        Some(track)
    }

    /// Add a clip to the given track. Panics if the track with the given id
    /// does not exist.
    pub fn add_clip(&mut self, track: TrackId, start: Beat) {
        let name = self.track_names.get(&track).expect("no dangling track id");
        let track = self.tracks.get_mut(&track).expect("no dangling track id");

        track.clips.push(Clip {
            name: name.clone(),
            start,
            length: Beat::ONE,
        });
    }

    pub fn tracks(&self) -> impl Iterator<Item = (TrackId, &Track)> {
        self.order.iter().map(|id| {
            (
                *id,
                self.tracks.get(id).expect("order is in sync with tracks"),
            )
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TrackId(usize);

#[derive(Debug, Default)]
pub struct Track {
    clips: Vec<Clip>,
    generation: usize,
}

impl Track {
    pub(super) fn clips(&self, id: TrackId) -> impl Iterator<Item = (ClipId, &Clip)> {
        self.clips
            .iter()
            .enumerate()
            .map(move |(index, clip)| (ClipId(self.generation, id, index), clip))
    }
}

/// A `ClipId` represents a clip in terms of a track and its index in the track.
/// This is invalidated whenever that track is removed or a clip in that track
/// is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct ClipId(usize, TrackId, usize);

#[derive(Debug)]
pub struct Clip {
    pub name: String,
    pub start: Beat,
    pub length: Beat,
}
