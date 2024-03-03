/// Convert from HSV colour to RGB colour
/// ([reference](https://en.wikipedia.org/wiki/HSL_and_HSV)).
///
/// Input HSV range is ([0,360], [0,1], [0,1]).
/// Output RGB range is ([0,1], [0,1], [0,1]).
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h = h % 360.0 / 60.0;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h {
        _ if h < 1.0 => (c, x, 0.0),
        _ if h < 2.0 => (x, c, 0.0),
        _ if h < 3.0 => (0.0, c, x),
        _ if h < 4.0 => (0.0, x, c),
        _ if h < 5.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}
