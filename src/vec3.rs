//! Three-dimensional vectors.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use rand::{self, RngExt};

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

    /// Computes the reflection of this vector from the surface with the given
    /// normal vector.
    ///
    /// Assumes that `unit_normal` is a unit normal vector.
    pub fn reflected_over(&self, unit_normal: Self) -> Self {
        // Compute the component of this vector that is parallel to the
        // normal, using the dot product (project v onto u).
        // This will be negative, since the incident vector and the normal are
        // assumed to point in opposite directions.
        let parallel_comp_len = self.dot(&unit_normal);
        // Invert the component parallel to the normal to perform the
        // reflection. This is done by subtracting double the parallel
        // component from the original vector: once to cancel its existing
        // inwards component, and once to give it an outwards component of
        // equal magnitude.
        *self - ((2.0 * parallel_comp_len) * unit_normal)
    }

    /// Produces the unit vector with the same direction as this vector.
    pub fn normalized(&self) -> Self {
        *self / self.length()
    }

    /// Produces a vector whose components are in the range [0.0, 1.0).
    pub fn random() -> Self {
        let mut rng = rand::rng();
        Self(rng.random())
    }

    /// Produces a vector whose components are in the range [`min`, `max`).
    pub fn random_range(min: f64, max: f64) -> Self {
        let mut rng = rand::rng();
        Self([
            rng.random_range(min..max),
            rng.random_range(min..max),
            rng.random_range(min..max),
        ])
    }

    /// Produces a unit vector with a random direction.
    pub fn random_unit() -> Self {
        loop {
            // Sample random vectors with components in the interval
            // [-1.0, 1.0) until one lies within the unit sphere, rejecting
            // other vectors. This is to avoid biasing towards the corners of
            // the cube-shaped state space, with vertices (+/-1, +/-1).
            // We also reject vectors whose lengths are extremely small, to
            // avoid errors due to floating point rounding.
            let v = Self::random_range(-1.0, 1.0);
            let v_len_squared = v.square_length();
            if (1.0e-160..=1.0).contains(&v_len_squared) {
                return v / v_len_squared.sqrt();
            }
        }
    }

    /// Produces a unit vector with a random direction, constrained to the
    /// hemisphere surrounding the given normal vector.
    pub fn random_hemisphere_unit(normal: &Self) -> Self {
        let v = Self::random_unit();
        if v.dot(normal) >= 0.0 { v } else { -v }
    }

    /// Produces a random vector within the XY-plane's unit disk.
    pub fn random_in_disk() -> Self {
        let mut rng = rand::rng();
        loop {
            // Sample random vectors in the XY-plane until one lies inside the
            // unit disk.
            let v = Self([
                rng.random_range(-1.0..=1.0),
                rng.random_range(-1.0..=1.0),
                0.0,
            ]);
            if v.square_length() < 1.0 {
                return v;
            }
        }
    }

    /// Determines whether this vector is very close to the zero vector.
    ///
    /// This is useful for preventing undesirable behavior where vectors with a
    /// very small (or zero) length could cause rounding errors or division by
    /// zero.
    pub fn is_near_zero(&self) -> bool {
        const EPSILON: f64 = 1e-8;
        self.0.into_iter().all(|component| component < EPSILON)
    }

    /// Computes the refraction of a unit vector through a refracting surface.
    ///
    /// `refraction_ratio` is the current medium's index of refraction divided
    /// by the entered medium's index of refraction.
    ///
    /// Assumes that this vector and `unit_normal` are both unit vectors.
    pub fn refract(&self, unit_normal: Self, refraction_ratio: f64) -> Self {
        // Snell's law: for angles theta and theta' from the normal, and
        // refraction indices eta and eta', then:
        //  eta * sin(theta) = eta' * sin(theta')
        // Therefore:
        //  sin(theta') = (eta / eta') * sin(theta')
        //
        // For an incident vector r, we want to find the resulting vector r'.
        // Finding the components of r' that are perpendicular and parallel to
        // the surface normal n:
        //  r_parallel = -|r| * cos(theta) * n
        //  r_perp = r - r_parallel
        //  r_perp = r + |r| * cos(theta) * n
        //  r'_perp = (eta / eta') * (r + |r| * cos(theta) * n)
        // The dot product can be expressed in terms of the cosine:
        //  a . b = |a| * |b| * cos(angle_ab)
        // Since r and n point in opposite directions, and n is a unit vector:
        //  -(r . n) = |r| * |n| * cos(theta)
        //  -(r . n) = |r| * cos(theta)
        let r_cos_theta = -self.dot(&unit_normal);
        let r_out_perp = refraction_ratio * (*self + (r_cos_theta * unit_normal));
        // Since the resulting vector will also be a unit vector, then by using
        // the Pythagorean theorem, we see:
        // |r'|^2 = |r'_perp|^2 + |r'_parallel|^2
        // 1 = |r'_perp|^2 + |r'_parallel|^2
        // |r'_parallel| = sqrt(1 - |r'_perp|^2)
        // r'_parallel = sqrt(1 - |r'_perp|^2) * (-n)
        let r_out_parallel = (1.0 - r_out_perp.square_length()).sqrt() * (-unit_normal);
        // Finally, r' is just the vector sum of its parallel and perpendicular
        // components.
        r_out_perp + r_out_parallel
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

impl Mul for Vec3 {
    type Output = Self;

    /// Computes the Hadamard product, which is elementwise multiplication of
    /// the vectors.
    fn mul(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
        ])
    }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
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

/// A `Vec3` used to store a real-valued color instead of a position.
pub type ColorVec3 = Vec3;

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

    #[test]
    fn random_vector_sampling() {
        for _ in 0..10_000 {
            let v = Vec3::random_unit();
            assert!((v.length() - 1.0).abs() < 1.0e-10);
        }

        let normal = Vec3([1.0, -1.0, 0.0]);
        for _ in 0..10_000 {
            let v = Vec3::random_hemisphere_unit(&normal);
            assert!((v.length() - 1.0).abs() < 1.0e-10);
            assert!(v.dot(&normal) >= 0.0);
        }
    }
}
