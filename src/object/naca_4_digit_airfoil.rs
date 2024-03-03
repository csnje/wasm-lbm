use super::Object;
use crate::linear_algebra::VectorRotate;

/// A type describing a [4-digit NACA airfoil](https://en.wikipedia.org/wiki/NACA_airfoil).
pub struct Naca4DigitAirfoil {
    /// Position.
    pos: [f32; 2],
    /// Chord length.
    c: f32,
    /// Maximum camber.
    m: f32,
    /// Location of maximum camber (fraction of chord).
    p: f32,
    /// Maximum thickness (fraction of chord).
    t: f32,
    /// [Angle of attack](https://en.wikipedia.org/wiki/Angle_of_attack) (radians).
    a: f32,
}

impl Naca4DigitAirfoil {
    /// Creates a new `Naca4DigitAirfoil`.
    pub fn new(pos: [f32; 2], c: f32, m: f32, p: f32, t: f32, a: f32) -> Self {
        Self { pos, c, m, p, t, a }
    }
}

impl Object<2> for Naca4DigitAirfoil {
    fn characteristic_length(&self) -> f32 {
        self.c
    }

    fn contains(&self, pos: &[f32; 2]) -> bool {
        // 1. translate position
        // 2. scale to chord
        let [x, y] = [pos[0] - self.pos[0], pos[1] - self.pos[1]].map(|val| val / self.c);

        if x * x + y * y > 1.0 {
            return false;
        }

        // rotate to axis
        let [x, y] = [x, y].rotate(self.a);

        // calculate mean camber at the x-position
        let y_c = if self.m == 0.0 {
            0.0
        } else if x <= self.p {
            self.m / (self.p * self.p) * (self.p + self.p - x) * x
        } else {
            let (tmp1, tmp2) = (1.0 - self.p, self.p + self.p);
            self.m / (tmp1 * tmp1) * (1.0 - tmp2 + (tmp2 - x) * x)
        };

        // calculate offset from the mean camber at the x-position
        let y_t = 5.0
            * self.t
            * (0.2969 * x.sqrt() + (-0.126 + (-0.3516 + (0.2843 - 0.1015 * x) * x) * x) * x);

        (y_c - y_t..=y_c + y_t).contains(&(y))
    }
}
