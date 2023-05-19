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
        let clips = &mut self
            .tracks
            .get_mut(&track)
            .expect("no dangling track id")
            .clips;

        clips.push(Clip {
            name: name.clone(),
            start,
            length: Beat::ONE,
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TrackId(usize);

#[derive(Debug, Default)]
pub struct Track {
    clips: Vec<Clip>,
}

#[derive(Debug)]
struct Clip {
    name: String,
    start: Beat,
    length: Beat,
}
