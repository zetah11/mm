use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Mul};

use num_bigint::BigInt;
use num_rational::BigRational;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Factor(pub BigRational);

impl Factor {
    pub fn one() -> Self {
        Self(BigRational::from_integer(BigInt::from(1)))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Length {
    Bounded(BigRational),
    Unbounded,
}

impl Length {
    pub fn one() -> Self {
        Self::Bounded(BigRational::from_integer(BigInt::from(1)))
    }

    pub fn zero() -> Self {
        Self::Bounded(BigRational::from_integer(BigInt::from(0)))
    }

    pub fn is_unbounded(&self) -> bool {
        matches!(self, Length::Unbounded)
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bounded(length) => write!(f, "{length}"),
            Self::Unbounded => write!(f, "oo"),
        }
    }
}

impl PartialOrd for Length {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Length {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Unbounded, Self::Unbounded) => Ordering::Equal,
            (Self::Unbounded, _) => Ordering::Greater,
            (_, Self::Unbounded) => Ordering::Less,
            (Self::Bounded(left), Self::Bounded(right)) => left.cmp(right),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Time(pub BigRational);

impl Time {
    pub fn zero() -> Self {
        Self(BigRational::from_integer(BigInt::from(0)))
    }
}

impl Add<&'_ Time> for &'_ Length {
    type Output = Time;

    fn add(self, rhs: &Time) -> Time {
        rhs + self
    }
}

impl Add<&'_ Length> for &'_ Time {
    type Output = Time;

    fn add(self, rhs: &Length) -> Time {
        let Length::Bounded(length) = rhs else { panic!("add unbounded length to time") };
        Time(&self.0 + length)
    }
}

impl Add for &'_ Length {
    type Output = Length;

    fn add(self, rhs: Self) -> Length {
        match (self, rhs) {
            (Length::Bounded(left), Length::Bounded(right)) => Length::Bounded(left + right),
            _ => Length::Unbounded,
        }
    }
}

impl Mul for &'_ Factor {
    type Output = Factor;

    fn mul(self, rhs: Self) -> Factor {
        Factor(&self.0 * &rhs.0)
    }
}

impl Mul<&'_ Length> for &'_ Factor {
    type Output = Length;

    fn mul(self, rhs: &Length) -> Length {
        rhs * self
    }
}

impl Mul<&'_ Factor> for &'_ Length {
    type Output = Length;

    fn mul(self, rhs: &'_ Factor) -> Length {
        match self {
            Length::Bounded(length) => Length::Bounded(length * &rhs.0),
            Length::Unbounded => Length::Unbounded,
        }
    }
}
