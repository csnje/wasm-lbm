use itertools::izip;

use super::Object;

/// A type describing an object that is circular in each dimension (e.g. circle, sphere).
pub struct Circular<const D: usize> {
    /// Position
    pos: [f32; D],
    /// Radius squared.
    rxr: f32,
    /// Characteristic length.
    characteristic_length: f32,
}

impl<const D: usize> Circular<D> {
    /// Create a new `Circular`.
    pub fn new(pos: [f32; D], r: f32) -> Self {
        Self {
            pos,
            rxr: r * r,
            characteristic_length: r + r,
        }
    }
}

impl<const D: usize> Object<D> for Circular<D> {
    fn characteristic_length(&self) -> f32 {
        self.characteristic_length
    }

    fn contains(&self, pos: &[f32; D]) -> bool {
        izip!(pos, self.pos).fold(0.0, |acc, (first, second)| {
            let d = first - second;
            acc + d * d
        }) <= self.rxr
    }
}
