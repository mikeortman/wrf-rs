use std::cmp::Ordering;
use std::ops::{Add, Sub};

use crate::{TimeError, TimeResult};

#[derive(Clone, Copy, Debug, Eq)]
pub(super) struct Rational {
    numerator: i128,
    denominator: i128,
}

impl Rational {
    pub(super) fn try_new(numerator: i128, denominator: i128) -> TimeResult<Self> {
        if denominator == 0 {
            return Err(TimeError::ZeroDenominator);
        }

        let (positive_denominator_numerator, positive_denominator) = if denominator < 0 {
            (-numerator, -denominator)
        } else {
            (numerator, denominator)
        };
        let greatest_common_divisor = Self::calculate_greatest_common_divisor(
            positive_denominator_numerator.unsigned_abs(),
            positive_denominator as u128,
        ) as i128;

        Ok(Self {
            numerator: positive_denominator_numerator / greatest_common_divisor,
            denominator: positive_denominator / greatest_common_divisor,
        })
    }

    pub(super) const fn from_integer(value: i128) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    pub(super) const fn numerator(self) -> i128 {
        self.numerator
    }

    pub(super) const fn denominator(self) -> i128 {
        self.denominator
    }

    pub(super) fn split_floor(self) -> (i128, i128, i128) {
        let whole = self.numerator.div_euclid(self.denominator);
        let remainder = self.numerator.rem_euclid(self.denominator);
        (whole, remainder, self.denominator)
    }

    pub(super) fn calculate_truncating_ratio(self, other: Self) -> TimeResult<i64> {
        if other.numerator == 0 {
            return Err(TimeError::DivisionByZero);
        }

        let numerator = self.numerator * other.denominator;
        let denominator = self.denominator * other.numerator;
        Ok((numerator / denominator) as i64)
    }

    pub(super) fn try_multiply(self, multiplier: i64) -> TimeResult<Self> {
        Self::try_new(self.numerator * i128::from(multiplier), self.denominator)
    }

    pub(super) fn try_divide(self, divisor: i64) -> TimeResult<Self> {
        if divisor == 0 {
            return Err(TimeError::DivisionByZero);
        }

        Self::try_new(self.numerator, self.denominator * i128::from(divisor))
    }

    const fn calculate_greatest_common_divisor(mut left: u128, mut right: u128) -> u128 {
        while right != 0 {
            let remainder = left % right;
            left = right;
            right = remainder;
        }

        if left == 0 { 1 } else { left }
    }
}

impl PartialEq for Rational {
    fn eq(&self, other: &Self) -> bool {
        self.numerator == other.numerator && self.denominator == other.denominator
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.numerator * other.denominator).cmp(&(other.numerator * self.denominator))
    }
}

impl Add for Rational {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let numerator = self.numerator * other.denominator + other.numerator * self.denominator;
        let denominator = self.denominator * other.denominator;

        Self::try_new(numerator, denominator)
            .unwrap_or_else(|_| unreachable!("valid rational addition preserves denominator"))
    }
}

impl Sub for Rational {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let numerator = self.numerator * other.denominator - other.numerator * self.denominator;
        let denominator = self.denominator * other.denominator;

        Self::try_new(numerator, denominator)
            .unwrap_or_else(|_| unreachable!("valid rational subtraction preserves denominator"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_reduces_fraction_and_normalizes_denominator_sign() {
        let rational = Rational::try_new(6, -8).unwrap();

        assert_eq!(rational.numerator(), -3);
        assert_eq!(rational.denominator(), 4);
    }

    #[test]
    fn split_floor_keeps_remainder_positive_for_negative_value() {
        let rational = Rational::try_new(-1, 3).unwrap();

        assert_eq!(rational.split_floor(), (-1, 2, 3));
    }
}
