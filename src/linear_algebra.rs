use itertools::izip;

pub trait VectorOps<const D: usize> {
    /// Vector [dot product](https://en.wikipedia.org/wiki/Dot_product).
    fn dot_product(&self, other: &[f32; D]) -> f32;
}

impl<const D: usize> VectorOps<D> for [f32; D] {
    /// Vector [dot product](https://en.wikipedia.org/wiki/Dot_product).
    fn dot_product(&self, other: &[f32; D]) -> f32 {
        izip!(self, other).fold(0.0, |acc, (first, second)| acc + first * second)
    }
}

pub trait VectorRotate<const D: usize> {
    fn rotate(&self, angle: f32) -> [f32; D];
}

impl VectorRotate<2> for [f32; 2] {
    fn rotate(&self, angle: f32) -> [f32; 2] {
        let (sin_a, cos_a) = angle.sin_cos();
        [
            self[0] * cos_a - self[1] * sin_a,
            self[0] * sin_a + self[1] * cos_a,
        ]
    }
}
