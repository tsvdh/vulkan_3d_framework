use crate::app::scene::SceneObject;
use crate::app::AppApi;
use crate::scripts::{convert_args, Script};
use serde::Deserialize;

#[derive(Deserialize)]
enum Axis {
    X, Y, Z
}

#[derive(Deserialize)]
struct Args {
    speed: f32,
    axis: Axis,
}

pub struct Rotate {
    args: Args,
}

impl Rotate {
    pub fn new(args: serde_json::Value) -> Self {
        Rotate {
            args: convert_args(args),
        }
    }
}

impl Script for Rotate {

    fn frame_update(&mut self, cur_object: &mut SceneObject, app_api: &mut AppApi) {
        let mut cur_rotation = match self.args.axis {
            Axis::X => { cur_object.rotation.x  }
            Axis::Y => { cur_object.rotation.y }
            Axis::Z => { cur_object.rotation.z }
        };

        cur_rotation += self.args.speed * app_api.timing_api.frame_duration;
        cur_rotation = cur_rotation % 360.0;

        match self.args.axis {
            Axis::X => { cur_object.rotation.x = cur_rotation }
            Axis::Y => { cur_object.rotation.y = cur_rotation }
            Axis::Z => { cur_object.rotation.z = cur_rotation }
        }
    }
}