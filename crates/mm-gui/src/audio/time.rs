use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

const BPM_FRACTION_BITS: u32 = 12;

#[derive(Clone, Copy, Debug)]
pub struct Beat(f64);

impl Beat {
    pub const ONE: Self = Self(1.0);

    pub fn to_f32(self) -> f32 {
        self.0 as f32
    }
}

impl Add for Beat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Beat {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl From<f64> for Beat {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Eq for Beat {}

impl Ord for Beat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for Beat {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for Beat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bpm(f64);

impl Bpm {
    /// Get the number of beats per sample for a specific sample rate.
    pub fn beats_per_sample(&self, sample_rate: Hz) -> Beat {
        Beat((self.0 / 60.0) / sample_rate.0)
    }
}

impl Default for Bpm {
    fn default() -> Self {
        Self(120.0)
    }
}

impl From<f64> for Bpm {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<Bpm> for f64 {
    fn from(value: Bpm) -> Self {
        value.0
    }
}

impl From<u32> for Bpm {
    fn from(value: u32) -> Self {
        Self(value as f64 / (1 << BPM_FRACTION_BITS) as f64)
    }
}

impl From<Bpm> for u32 {
    fn from(value: Bpm) -> Self {
        (value.0 * (1 << BPM_FRACTION_BITS) as f64) as u32
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hz(f64);

impl Eq for Hz {}

impl Ord for Hz {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for Hz {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for Hz {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<u32> for Hz {
    fn from(value: u32) -> Self {
        Self(value as f64)
    }
}

impl From<f64> for Hz {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Second(f64);

impl Second {
    pub const ZERO: Self = Self(0.0);
}

impl fmt::Display for Second {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let minutes = self.0.div_euclid(60.0);
        let seconds = self.0.rem_euclid(60.0);

        if let Some(precision) = f.precision() {
            if minutes == 0.0 {
                write!(f, "{seconds:.*} s", precision)
            } else {
                write!(f, "{minutes:.0} min {seconds:.*} s", precision)
            }
        } else if minutes == 0.0 {
            write!(f, "{seconds} s")
        } else {
            write!(f, "{minutes:.0} min {seconds} s")
        }
    }
}

impl Eq for Second {}

impl Ord for Second {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for Second {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for Second {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<f64> for Second {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Add for Second {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Second {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

/// `beat / (beat / time) = time`
impl Div<Bpm> for Beat {
    type Output = Second;

    fn div(self, rhs: Bpm) -> Self::Output {
        Second(60.0 * self.0 / rhs.0)
    }
}

/// `1 / (1 / time) = time`
impl Div<Hz> for f64 {
    type Output = Second;

    fn div(self, rhs: Hz) -> Self::Output {
        Second(self / rhs.0)
    }
}

/// `1 / time`
impl Div<Second> for f64 {
    type Output = Hz;

    fn div(self, rhs: Second) -> Self::Output {
        Hz(self / rhs.0)
    }
}

/// `time / time`
impl Div for Second {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

/// `time * 1 / time = 1`
impl Mul<Hz> for Second {
    type Output = f64;

    fn mul(self, rhs: Hz) -> Self::Output {
        self.0 * rhs.0
    }
}

/// `(1 / time) * time = 1`
impl Mul<Second> for Hz {
    type Output = f64;

    fn mul(self, rhs: Second) -> Self::Output {
        rhs * self
    }
}

/// `time * beat / time = beat`
impl Mul<Second> for Bpm {
    type Output = Beat;

    fn mul(self, rhs: Second) -> Self::Output {
        Beat(self.0 * rhs.0 / 60.0)
    }
}

/// `(beat / time) * time = beat`
impl Mul<Bpm> for Second {
    type Output = Beat;

    fn mul(self, rhs: Bpm) -> Self::Output {
        rhs * self
    }
}

impl Sub for Second {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Second {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
