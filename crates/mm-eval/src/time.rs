use std::cmp::Ordering;
use std::fmt;
use std::iter::Sum;
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

impl Add<Time> for Length {
    type Output = Time;

    fn add(self, rhs: Time) -> Self::Output {
        rhs + self
    }
}

impl Add<Length> for Time {
    type Output = Self;

    fn add(self, rhs: Length) -> Self {
        let Length::Bounded(length) = rhs else { panic!("add unbounded length to time") };
        Self(self.0 + length)
    }
}

impl Add for Length {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Bounded(left), Self::Bounded(right)) => Self::Bounded(left + right),
            _ => Self::Unbounded,
        }
    }
}

impl Mul for Factor {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Mul<Length> for Factor {
    type Output = Length;

    fn mul(self, rhs: Length) -> Length {
        rhs * self
    }
}

impl Mul<Factor> for Length {
    type Output = Self;

    fn mul(self, rhs: Factor) -> Self {
        match self {
            Self::Bounded(length) => Self::Bounded(length * rhs.0),
            Self::Unbounded => Self::Unbounded,
        }
    }
}

impl Sum for Length {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Self::zero();
        for item in iter {
            result = result + item;
        }
        result
    }
}
