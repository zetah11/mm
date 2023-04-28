pub mod check;
pub mod eval;
pub mod implicit;
pub mod melody;
pub mod note;
pub mod parse;

pub use crate::state::CompilerState;

mod state;

mod dependency;
mod topology;

use std::cmp::Ordering;
use std::iter::Sum;
use std::ops::{Add, Mul};

use rational::Rational;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Name(pub String);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Factor(pub Rational);

impl Factor {
    pub fn one() -> Self {
        Self(Rational::one())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Length {
    Bounded(Rational),
    Unbounded,
}

impl Length {
    pub fn one() -> Self {
        Self::Bounded(Rational::one())
    }

    pub fn zero() -> Self {
        Self::Bounded(Rational::zero())
    }

    pub fn is_unbounded(&self) -> bool {
        matches!(self, Length::Unbounded)
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Time(pub Rational);

impl Time {
    pub fn zero() -> Self {
        Self(Rational::zero())
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
