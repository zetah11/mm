use std::fmt;

use midly::num::u7;
use mm_eval::note::Note;

#[derive(Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub struct Pitch(isize);

impl Pitch {
    pub const fn to_midi_key(self) -> Option<u7> {
        match self.0 + 69 {
            value @ 0..=127 => Some(u7::new(value as u8)),
            _ => None,
        }
    }

    pub const fn to_midi_key_saturating(self) -> u7 {
        match self.0 + 69 {
            value @ 0..=127 => u7::new(value as u8),
            128.. => u7::max_value(),
            _ => u7::new(0),
        }
    }
}

impl Note for Pitch {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "A" => Pitch(0),
            "B" => Pitch(2),
            "C" => Pitch(-9),
            "D" => Pitch(-7),
            "E" => Pitch(-5),
            "F" => Pitch(-4),
            "G" => Pitch(-2),

            _ => return None,
        })
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let in_octave = (self.0 + 9).rem_euclid(12) - 9;

        let name = match in_octave {
            -9 => "C",
            -8 => "C#",
            -7 => "D",
            -6 => "D#",
            -5 => "E",
            -4 => "F",
            -3 => "F#",
            -2 => "G",
            -1 => "G#",
            0 => "A",
            1 => "A#",
            2 => "B",

            _ => unreachable!(),
        };

        let octave = {
            let v = self.0 + 9;
            let off = if v < 0 { -1 } else { 0 };
            v / 12 + off + 4
        };

        write!(f, "{name}{octave}")
    }
}
