//! Three-dimensional vectors.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A three-dimensional real-valued vector.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3(pub [f64; 3]);

impl Vec3 {
    /// Constructs a new zero vector.
    pub const fn new() -> Self {
        Self([0.0; 3])
    }

    /// Gets the x component of this vector.
    pub const fn x(&self) -> f64 {
        self.0[0]
    }

    /// Gets the y component of this vector.
    pub const fn y(&self) -> f64 {
        self.0[1]
    }

    /// Gets the z component of this vector.
    pub const fn z(&self) -> f64 {
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
    pub const fn square_length(&self) -> f64 {
        self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]
    }

    /// Computes the dot product with another vector.
    pub const fn dot(&self, rhs: &Self) -> f64 {
        self.0[0] * rhs.0[0] + self.0[1] * rhs.0[1] + self.0[2] * rhs.0[2]
    }

    /// Computes the cross product with another vector.
    pub const fn cross(&self, rhs: &Self) -> Self {
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

impl Default for Vec3 {
    /// Constructs a new zero vector.
    fn default() -> Self {
        Self::new()
    }
}

impl From<[f64; 3]> for Vec3 {
    /// Constructs a new vector with the given components.
    fn from(value: [f64; 3]) -> Self {
        Self(value)
    }
}

impl From<(f64, f64, f64)> for Vec3 {
    /// Constructs a new vector with the given components.
    fn from(value: (f64, f64, f64)) -> Self {
        Self(value.into())
    }
}

impl From<Vec3> for [f64; 3] {
    /// Extracts the components of the vector as an array.
    fn from(value: Vec3) -> Self {
        value.0
    }
}

impl From<Vec3> for (f64, f64, f64) {
    /// Extracts the components of the vector as a tuple.
    fn from(value: Vec3) -> Self {
        value.0.into()
    }
}

impl Neg for Vec3 {
    type Output = Self;

    /// Performs vector negation.
    ///
    /// This is equivalent to multiplying this vector by `-1.0`.
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
        for component in self.0.each_mut() {
            *component *= rhs;
        }
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

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        for component in self.0.each_mut() {
            *component /= rhs;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_vector_math() {
        let v1 = Vec3([1.0, 2.0, 3.0]);
        let v2 = Vec3([5.0, 7.0, 1.0]);
        assert_eq!(v1 + v2, Vec3([6.0, 9.0, 4.0]));
        assert_eq!(v2 + v1, Vec3([6.0, 9.0, 4.0]));
        assert_eq!(v1 - v2, Vec3([-4.0, -5.0, 2.0]));
        assert_eq!(v2 - v1, Vec3([4.0, 5.0, -2.0]));

        assert_eq!(v1.dot(&v2), 22.0);
        assert_eq!(v2.dot(&v1), 22.0);
        assert_eq!(v1.cross(&v2), Vec3([-19.0, 14.0, -3.0]));
        assert_eq!(v2.cross(&v1), Vec3([19.0, -14.0, 3.0]));
    }

    #[test]
    fn vector_scalar_math() {
        let v = Vec3([9.0, 4.0, -3.0]);
        assert_eq!(2.0 * v, Vec3([18.0, 8.0, -6.0]));
        assert_eq!(v * 2.0, Vec3([18.0, 8.0, -6.0]));
        assert_eq!(v / 2.0, Vec3([4.5, 2.0, -1.5]));
    }

    #[test]
    fn unary_vector_math() {
        let v = Vec3([2.0, -5.0, 1.0]);
        assert_eq!(-v, Vec3([-2.0, 5.0, -1.0]));

        let v = Vec3([12.0, 16.0, -21.0]);
        assert_eq!(v.square_length(), 841.0);
        assert_eq!(v.length(), 29.0);

        let v = Vec3([4.0, 0.0, 0.0]);
        assert_eq!(v.length(), 4.0);
        assert_eq!(v.normalized(), Vec3([1.0, 0.0, 0.0]));
    }

    #[test]
    fn vector_assign_math() {
        let mut v = Vec3([3.0, -6.0, 5.0]);

        v += Vec3([1.0, 2.0, -3.0]);
        assert_eq!(v, Vec3([4.0, -4.0, 2.0]));

        v -= Vec3([6.0, 7.0, -1.0]);
        assert_eq!(v, Vec3([-2.0, -11.0, 3.0]));

        v *= 2.0;
        assert_eq!(v, Vec3([-4.0, -22.0, 6.0]));

        v /= 2.0;
        assert_eq!(v, Vec3([-2.0, -11.0, 3.0]));
    }
}
