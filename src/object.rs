pub mod circular;
pub mod naca_4_digit_airfoil;

pub trait Object<const D: usize> {
    /// The [characteristic length](https://en.wikipedia.org/wiki/Characteristic_length) of the object.
    fn characteristic_length(&self) -> f32;

    /// Calculate whether the object contains a position.
    fn contains(&self, pos: &[f32; D]) -> bool;
}
