use egui::Ui;
use glam::Quat;
use crate::app::AppApi;
use crate::scripts::{convert_args, SceneObjectScript};
use serde::Deserialize;
use crate::app::scene::{SceneApi, SceneObject};
use crate::app::ui::ControlUi;
use crate::app::util::{rad_from_deg, GlobalTransformData};

#[derive(Deserialize)]
enum Axis {
    X, Y, Z
}

#[derive(Deserialize)]
struct Args {
    speed: f32,
    axis: Axis,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            speed: 0.0,
            axis: Axis::X,
        }
    }
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

impl SceneObjectScript for Rotate {

    fn frame_update(&mut self, cur_object: &mut SceneObject, global_transform_data: &GlobalTransformData, app_api: &mut AppApi) {
        let rad_diff = rad_from_deg(self.args.speed * app_api.timing_api.frame_duration);

        cur_object.transform.rotation *= match self.args.axis {
            Axis::X => { Quat::from_rotation_x(rad_diff) }
            Axis::Y => { Quat::from_rotation_y(rad_diff) }
            Axis::Z => { Quat::from_rotation_z(rad_diff) }
        }
    }
}

impl ControlUi for Rotate {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
        ui.label("-");
    }
}
