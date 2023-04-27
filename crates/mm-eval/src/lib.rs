pub mod check;
pub mod eval;
pub mod implicit;
pub mod melody;

mod dependency;
mod topology;

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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Length(pub Rational);

impl Length {
    pub fn one() -> Self {
        Self(Rational::one())
    }

    pub fn zero() -> Self {
        Self(Rational::zero())
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
        Self(self.0 + rhs.0)
    }
}

impl Add for Length {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
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
        Self(rhs.0 * self.0)
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
