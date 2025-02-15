mod animation;

use serde::{Deserialize, Serialize};

use crate::{fractal::Fractal, F};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Render {
    Frame {
        zoom: F,
        center_x: F,
        center_y: F,
        fractal: Fractal,
    },
    Animation {
        zoom: Vec<RenderStep>,
        center_x: Vec<RenderStep>,
        center_y: Vec<RenderStep>,
        fractal: animation::Fractal,
        duration: f32,
        fps: f32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RenderStep {
    /// (start_time, end_time, value)
    Const(f32, f32, F),
    /// (start_time, end_time, start_value, end_value)
    Linear(f32, f32, F, F),
    /// (start_time, end_time, start_value, end_value)
    Smooth(f32, f32, F, F),
}

impl RenderStep {
    pub fn get_current_step_index(steps: &[RenderStep], t: f32) -> usize {
        steps
            .iter()
            .enumerate()
            .find_map(|(i, &step)| match step {
                RenderStep::Const(start_time, end_time, _)
                | RenderStep::Linear(start_time, end_time, _, _)
                | RenderStep::Smooth(start_time, end_time, _, _) => {
                    (start_time <= t && t <= end_time).then_some(i)
                }
            })
            .unwrap()
    }

    pub fn get_value(&self, t: f32) -> F {
        // see https://www.desmos.com/calculator/a1ddmg7pxk
        match *self {
            RenderStep::Const(_, _, value) => value,
            RenderStep::Linear(start_time, end_time, start_value, end_value) => {
                let w = ((t - start_time) / (end_time - start_time)) as F;
                start_value * (1. - w) + end_value * w
            }
            RenderStep::Smooth(start_time, end_time, start_value, end_value) => {
                let w = ((t - start_time) / (end_time - start_time)) as F;
                let smooth_w = w * w * (3. - 2. * w);
                start_value * (1. - smooth_w) + end_value * smooth_w
            }
        }
    }
}
