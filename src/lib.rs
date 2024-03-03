pub mod colour;
pub mod image_values;
pub mod lbgk;
pub mod linear_algebra;
pub mod object;

use image_values::ImageValues;
use lbgk::Lbgk;
use linear_algebra::VectorOps;
use object::Object;

use itertools::iproduct;
use js_sys::Date;
use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const SIZE: [usize; 2] = [401, 201];
const BOUNDARY_SCHEMES: [[lbgk::BoundaryScheme; 2]; 2] = [
    [lbgk::BoundaryScheme::Inflow, lbgk::BoundaryScheme::Outflow],
    [lbgk::BoundaryScheme::SpecularReflection; 2],
];

const DENSITY: f32 = 1.0;
const VELOCITY_VECTOR: [f32; 2] = [0.1, 0.0];

// Reynolds number (https://en.wikipedia.org/wiki/Reynolds_number)
const RE: f32 = 200.0;

const RATE_MOVING_AVERAGE_PERIOD_SECS: f64 = 2.0;
const DRAW_ITERATION_STEPS: usize = 10;

fn window() -> web_sys::Window {
    web_sys::window().expect("should have window")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register request animation frame callback");
}

struct UserInterfaceElements {
    canvas_rendering_contexts: [web_sys::CanvasRenderingContext2d; 3],
    iteration_element: web_sys::Element,
    rate_element: web_sys::Element,
}

impl UserInterfaceElements {
    fn new(
        paused: Rc<RefCell<bool>>,
        velocity: f32,
        relaxation_time: f32,
    ) -> Result<Self, JsValue> {
        let document = window().document().ok_or("should have document")?;
        let body = document.body().ok_or("should have document body")?;

        let canvas_rendering_contexts = ["Density", "Velocity", "Vorticity"].map(|name| {
            let div = document.create_element("div").unwrap();
            div.set_text_content(Some(name));
            body.append_child(&div).unwrap();

            let canvas = document
                .create_element("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            canvas.set_width(SIZE[0] as u32);
            canvas.set_height(SIZE[1] as u32);
            body.append_child(&canvas).unwrap();

            canvas
                .get_context("2d")
                .unwrap()
                .expect("should have 2d context")
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap()
        });

        let iteration_element = {
            let iteration_element = document.create_element("div")?;
            body.append_child(&iteration_element)?;
            iteration_element
        };
        let rate_element = {
            let frames_element = document.create_element("div")?;
            body.append_child(&frames_element)?;
            frames_element
        };
        {
            let div = document.create_element("div")?;
            div.set_text_content(Some(&format!("Magnitude velocity {velocity}")));
            body.append_child(&div)?;
        }
        {
            let div = document.create_element("div")?;
            div.set_text_content(Some(&format!("Reynolds number {RE}")));
            body.append_child(&div)?;
        }
        {
            let div = document.create_element("div")?;
            div.set_text_content(Some(&format!("=> Relaxation time {relaxation_time}")));
            body.append_child(&div)?;
        }
        {
            let button_pause = document
                .create_element("button")
                .unwrap()
                .dyn_into::<web_sys::HtmlButtonElement>()?;
            button_pause.set_text_content(Some(if *paused.borrow() { "Start" } else { "Pause" }));

            let div = document.create_element("div").unwrap();
            div.append_child(&button_pause)?;
            body.append_child(&div)?;

            let button_pause = Rc::new(RefCell::new(button_pause));

            let paused_clone = paused.clone();
            let button_pause_clone = button_pause.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                let paused = !*paused_clone.borrow();
                *paused_clone.borrow_mut() = paused;
                button_pause_clone
                    .borrow()
                    .set_text_content(Some(if paused { "Start" } else { "Pause" }));
            });
            button_pause
                .borrow()
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        };

        Ok(Self {
            canvas_rendering_contexts,
            iteration_element,
            rate_element,
        })
    }
}

/// Entry point of the application.
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let objects = vec![object::circular::Circular::new(
        [SIZE[0] as f32 / 4.0, SIZE[1] as f32 / 2.0],
        SIZE[1] as f32 / 10.0,
    )];
    // let objects = vec![
    //     object::circular::Circular::new([SIZE[0] as f32 / 3.0, 0.0], SIZE[1] as f32 / 4.0),
    //     object::circular::Circular::new(
    //         [SIZE[0] as f32 / 3.0, (SIZE[1] - 1) as f32],
    //         SIZE[1] as f32 / 4.0,
    //     ),
    // ];
    // NACA 2412
    // let objects = vec![object::naca_4_digit_airfoil::Naca4DigitAirfoil::new(
    //     [SIZE[0] as f32 / 5.0, SIZE[1] as f32 / 2.0],
    //     SIZE[1] as f32 / 2.0,
    //     0.02,
    //     0.4,
    //     0.12,
    //     8.0f32.to_radians(),
    // )];
    // NACA 2415
    // let objects = vec![object::naca_4_digit_airfoil::Naca4DigitAirfoil::new(
    //     [SIZE[0] as f32 / 5.0, SIZE[1] as f32 / 2.0],
    //     SIZE[1] as f32 / 2.0,
    //     0.02,
    //     0.4,
    //     0.15,
    //     8.0f32.to_radians(),
    // )];
    // NACA 6412
    // let objects = vec![object::naca_4_digit_airfoil::Naca4DigitAirfoil::new(
    //     [SIZE[0] as f32 / 5.0, SIZE[1] as f32 / 2.0],
    //     SIZE[1] as f32 / 2.0,
    //     0.06,
    //     0.4,
    //     0.12,
    //     8.0f32.to_radians(),
    // )];

    let mut lbgk = Lbgk::new_d2q9(&SIZE, &BOUNDARY_SCHEMES, DENSITY, &VELOCITY_VECTOR);
    for pos in iproduct!(0..SIZE[0], 0..SIZE[1]).map(|(x, y)| [x, y]) {
        lbgk.set_object(
            &pos,
            objects
                .iter()
                .any(|object| object.contains(&[pos[0] as f32, pos[1] as f32])),
        );
    }

    let velocity = VELOCITY_VECTOR.dot_product(&VELOCITY_VECTOR).sqrt();
    let relaxation_time = lbgk.relaxation_time(velocity, objects[0].characteristic_length(), RE);

    let paused = Rc::new(RefCell::new(false));
    let ui = UserInterfaceElements::new(paused.clone(), velocity, relaxation_time)?;

    let mut iteration = 0usize;
    let mut rate_dates = VecDeque::new();
    let mut image_values = ImageValues::new(&SIZE);

    let ff = Rc::new(RefCell::new(None));
    let ff_cloned = ff.clone();
    *ff.borrow_mut() = Some(Closure::new(move || {
        if !*paused.borrow() {
            iteration += 1;
            ui.iteration_element
                .set_text_content(Some(&format!("Iteration {iteration}")));

            let now = Date::now();
            rate_dates.push_back(now);
            while let Some(front) = rate_dates.front() {
                if front + (RATE_MOVING_AVERAGE_PERIOD_SECS * 1.0e3) > now {
                    break;
                }
                rate_dates.pop_front();
            }
            let rate = rate_dates.len() as f64 / RATE_MOVING_AVERAGE_PERIOD_SECS;
            ui.rate_element
                .set_text_content(Some(&format!("Iteration rate {rate}")));

            // iterate the algorithm
            lbgk.iterate(relaxation_time);

            if iteration % DRAW_ITERATION_STEPS == 0 {
                // draw density image
                let (mut min, mut max) = (f32::MAX, f32::MIN);
                for pos in iproduct!(0..SIZE[0], 0..SIZE[1]).map(|(x, y)| [x, y]) {
                    match lbgk.object(&pos) {
                        true => image_values.set_value(&pos, None),
                        false => {
                            let val = lbgk.density(&pos);
                            image_values.set_value(&pos, Some(val));
                            (min, max) = (min.min(val), max.max(val));
                        }
                    }
                }
                image_values.set_standard_value(DENSITY);
                image_values.set_minimum_value(min);
                image_values.set_maximum_value(max);
                let _ = image_values.draw(false, &ui.canvas_rendering_contexts[0]);

                // draw velocity image
                let (mut min, mut max) = (f32::MAX, f32::MIN);
                for pos in iproduct!(0..SIZE[0], 0..SIZE[1]).map(|(x, y)| [x, y]) {
                    match lbgk.object(&pos) {
                        true => image_values.set_value(&pos, None),
                        false => {
                            let val = lbgk.velocity(&pos);
                            image_values.set_value(&pos, Some(val));
                            (min, max) = (min.min(val), max.max(val));
                        }
                    }
                }
                image_values.set_standard_value(velocity);
                image_values.set_minimum_value(min);
                image_values.set_maximum_value(max);
                let _ = image_values.draw(false, &ui.canvas_rendering_contexts[1]);

                // draw vorticity image
                let (mut min, mut max) = (f32::MAX, f32::MIN);
                for pos in iproduct!(0..SIZE[0], 0..SIZE[1]).map(|(x, y)| [x, y]) {
                    match lbgk.object(&pos) {
                        true => image_values.set_value(&pos, None),
                        false => {
                            let val = lbgk.vorticity(&pos);
                            image_values.set_value(&pos, Some(val));
                            (min, max) = (min.min(val), max.max(val));
                        }
                    }
                }
                image_values.set_standard_value(0.0);
                image_values.set_minimum_value(min);
                image_values.set_maximum_value(max);
                let _ = image_values.draw(true, &ui.canvas_rendering_contexts[2]);
            }
        }

        request_animation_frame(ff_cloned.borrow().as_ref().unwrap());
    }));
    request_animation_frame(ff.borrow().as_ref().unwrap());

    Ok(())
}
