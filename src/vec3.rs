//! Three-dimensional vectors.

use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

/// A three-dimensional real-valued vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3(pub [f64; 3]);

impl Vec3 {
    /// Constructs a new zero vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the x component of this vector.
    pub fn x(&self) -> f64 {
        self.0[0]
    }

    /// Gets the y component of this vector.
    pub fn y(&self) -> f64 {
        self.0[1]
    }

    /// Gets the z component of this vector.
    pub fn z(&self) -> f64 {
        self.0[2]
    }

    /// Computes the length of this vector.
    ///
    /// If the square of the length is needed, use [`Self::square_length()`]
    /// instead.
    pub fn length(&self) -> f64 {
        self.square_length().sqrt()
    }

    /// Computes the square of the length of this vector.
    ///
    /// This is more efficient than getting the length and squaring it
    /// afterwards, and should be preferred over [`Self::length()`] when the
    /// square of the length is needed.
    pub fn square_length(&self) -> f64 {
        self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]
    }

    /// Computes the dot product with another vector.
    pub fn dot(&self, rhs: &Self) -> f64 {
        self.0[0] * rhs.0[0] + self.0[1] * rhs.0[1] + self.0[2] * rhs.0[2]
    }

    /// Computes the cross product with another vector.
    pub fn cross(&self, rhs: &Self) -> Self {
        Self([
            self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
            self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
        ])
    }

    /// Produces the unit vector with the same direction as this vector.
    pub fn normalized(&self) -> Self {
        *self / self.length()
    }
}

impl Neg for Vec3 {
    type Output = Self;

    /// Negate this vector.
    ///
    /// Equivalent to scaling this vector by a factor of -1.
    fn neg(self) -> Self::Output {
        Self(self.0.map(Neg::neg))
    }
}

impl Add for Vec3 {
    type Output = Self;

    /// Performs vector addition.
    fn add(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        let components = self.0.each_mut();
        *components[0] += rhs.0[0];
        *components[1] += rhs.0[1];
        *components[2] += rhs.0[2];
    }
}

impl Sub for Vec3 {
    type Output = Self;

    /// Performs vector subtraction.
    fn sub(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ])
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        let components = self.0.each_mut();
        *components[0] -= rhs.0[0];
        *components[1] -= rhs.0[1];
        *components[2] -= rhs.0[2];
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    /// Performs scalar multiplication.
    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0.map(|x| x * rhs))
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        let components = self.0.each_mut();
        *components[0] *= rhs;
        *components[1] *= rhs;
        *components[2] *= rhs;
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    /// Performs scalar multiplication.
    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    /// Performs scalar division.
    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0.map(|x| x / rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: tests
}
