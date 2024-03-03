/// Lattice vectors for the LBGK D2Q9 model.
/// Index for vectors:
///     6   2   5
///       \ | /
///     3 — 0 — 1
///       / | \
///     7   4   8
pub const C: [[isize; 2]; 9] = [
    [0, 0],
    [1, 0],
    [0, 1],
    [-1, 0],
    [0, -1],
    [1, 1],
    [-1, 1],
    [-1, -1],
    [1, -1],
];

/// Weights corresponding to the lattice vectors for the LBGK D2Q9 model.
pub const W: [f32; 9] = [
    4.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 36.0,
    1.0 / 36.0,
    1.0 / 36.0,
    1.0 / 36.0,
];

/// Sound speed squared for the LBGK D2Q9 model.
pub const CS2: f32 = 1.0 / 3.0;
