use std::fmt;
use std::ops::Add;

use midly::num::u7;
use mm_eval::note::Note;

const TWELFTH_ROOT_TWO: f64 = 1.059_463_094_359_295_3;

#[derive(Clone, Debug, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pitch(isize);

impl Pitch {
    pub const A4: Self = Self(0);

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

    /// Get the offset from this pitch to the given pitch.
    pub const fn offset(&self, to: &Self) -> isize {
        self.0 - to.0
    }

    /// Get the closest A below this pitch. If this pitch is itself
    pub const fn a_below(&self) -> Self {
        let off = if self.0 < 0 { 1 } else { 0 };
        Self(((self.0 + off) / 12 - off) * 12)
    }

    pub fn to_frequency(&self, a4: f64) -> f64 {
        a4 * TWELFTH_ROOT_TWO.powi(self.0 as i32)
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

    fn add_sharp(&self, by: usize) -> Self {
        Self(self.0 + by as isize)
    }

    fn add_octave(&self, by: isize) -> Self {
        Self(self.0 + 12 * by)
    }
}

impl Add<Interval> for Pitch {
    type Output = Self;

    fn add(self, rhs: Interval) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Add<Pitch> for Interval {
    type Output = Pitch;

    fn add(self, rhs: Pitch) -> Pitch {
        rhs + self
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

pub struct Interval(isize);

impl Interval {
    pub const SEMITONE: Self = Self(1);
    pub const WHOLETONE: Self = Self(2);
}
