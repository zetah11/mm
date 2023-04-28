use mm_eval::span::Span;
use mm_eval::{Length, Time};
pub use pitch::Pitch;

mod channel;
mod pitch;

use std::io;
use std::path::Path;

use midly::num::{u15, u24, u28, u4};
use midly::{Format, Header, MetaMessage, Smf, Timing, TrackEvent, TrackEventKind};

use self::channel::write_channel;

const TICKS_PER_BEAT: usize = 128;

/// Write the given notes to a MIDI file at the given path.
pub fn write<'src>(
    notes: impl Iterator<Item = (Pitch, Span<'src>, Time, Length)>,
    to: impl AsRef<Path>,
) -> Result<(), io::Error> {
    let mut track = vec![TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(500_000))),
    }];

    write_channel(notes, TICKS_PER_BEAT, u4::new(0), &mut track);

    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let header = Header::new(
        Format::Parallel,
        Timing::Metrical(u15::new(u16::try_from(TICKS_PER_BEAT).unwrap())),
    );

    let mut smf = Smf::new(header);
    smf.tracks.push(track);

    smf.save(to)
}
