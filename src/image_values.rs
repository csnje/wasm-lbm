use crate::colour::hsv_to_rgb;

use itertools::iproduct;
use wasm_bindgen::{prelude::*, Clamped};

const HUE_RANGE: [f32; 2] = [180.0, 360.0];

/// Image data.
pub struct ImageValues {
    size: [usize; 2], // size of the image
    data: Vec<u8>,    // RGBA data for the image
    // note: flat vectors reduce cache loads
    values: Vec<Option<f32>>,
    standard_value: f32,
    minimum_value: f32,
    maximum_value: f32,
}

impl ImageValues {
    /// Create a new `Image`.
    pub fn new(size: &[usize; 2]) -> Self {
        Self {
            size: *size,
            data: vec![u8::MAX; size[0] * size[1] * 4],
            values: vec![None; size[0] * size[1]],
            standard_value: 0.0,
            minimum_value: 0.0,
            maximum_value: 0.0,
        }
    }

    /// Set value at image position.
    pub fn set_value(&mut self, pos: &[usize; 2], value: Option<f32>) {
        self.values[self.size[0] * pos[1] + pos[0]] = value;
    }

    /// Set standard value.
    pub fn set_standard_value(&mut self, value: f32) {
        self.standard_value = value;
    }

    /// Set minimum value.
    pub fn set_minimum_value(&mut self, value: f32) {
        self.minimum_value = value;
    }

    /// Set maximum value.
    pub fn set_maximum_value(&mut self, value: f32) {
        self.maximum_value = value;
    }

    /// Draw values.
    pub fn draw(
        &mut self,
        amplify: bool,
        canvas_rendering_context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<(), JsValue> {
        let val_divisor = (self.maximum_value - self.standard_value)
            .abs()
            .max((self.minimum_value - self.standard_value).abs());

        for (x, y) in iproduct!(0..self.size[0], 0..self.size[1]) {
            let data_idx = (self.size[0] * (self.size[1] - 1 - y) + x) * 4;
            match self.values[self.size[0] * y + x] {
                None => {
                    self.data[data_idx] = u8::MAX;
                    self.data[data_idx + 1] = u8::MAX;
                    self.data[data_idx + 2] = u8::MAX;
                }
                Some(value) => {
                    let (r, g, b) = hsv_to_rgb(
                        match value < self.standard_value {
                            true => HUE_RANGE[0],
                            false => HUE_RANGE[1],
                        },
                        1.0,
                        {
                            let v = (value - self.standard_value).abs() / val_divisor;
                            match amplify {
                                true => v.sqrt(),
                                false => v,
                            }
                        },
                    );
                    self.data[data_idx] = (r * u8::MAX as f32) as u8;
                    self.data[data_idx + 1] = (g * u8::MAX as f32) as u8;
                    self.data[data_idx + 2] = (b * u8::MAX as f32) as u8;
                }
            }
        }

        canvas_rendering_context.put_image_data(
            &web_sys::ImageData::new_with_u8_clamped_array(
                Clamped(&self.data),
                self.size[0] as u32,
            )
            .unwrap(),
            0.0,
            0.0,
        )
    }
}
