use egui::Ui;
use crate::app::scene::{SceneApi, SceneObject};
use crate::app::AppApi;
use crate::scripts::{convert_args, SceneObjectScript};
use serde::Deserialize;
use crate::app::ui::ControlUi;

#[derive(Deserialize)]
struct Args {

}

pub struct CameraControl {
    args: Args,
}

impl CameraControl {
    pub fn new(args: serde_json::Value) -> Self {
        CameraControl {
            args: convert_args(args)
        }
    }
}

impl SceneObjectScript for CameraControl {

    fn frame_update(&mut self, cur_object: &mut SceneObject, app_api: &mut AppApi)
    {
        // let cur_object = app_api.scene_api.scene_objects.get_mut(cur_object_id);
        //
        // // camera controls
        // // rotate 90 degrees (pi/2) in 1 sec
        // // zoom 1m in 1 sec
        //
        // let mut vertical_angle_diff = FRAC_PI_2 * app_api.timing_api.frame_duration;
        // let mut horizontal_angle_diff = FRAC_PI_2 * app_api.timing_api.frame_duration;
        //
        // let keys_down = app_api.logic_api.keys_down;
        //
        // if keys_down.contains(&ArrowDown) {
        //     vertical_angle_diff *= -1.0;
        // }
        // if keys_down.contains(&ArrowLeft) {
        //     horizontal_angle_diff *= -1.0;
        // }
        //
        // let camera = &mut cur_object.transform;
        //
        // if keys_down.contains(&ArrowUp) || keys_down.contains(&ArrowDown) {
        //     camera.translation = camera.translation.rotate_axis(camera.horizon, vertical_angle_diff);
        // }
        // if keys_down.contains(&ArrowLeft) || keys_down.contains(&ArrowRight) {
        //     camera.translation = camera.translation.rotate_y(horizontal_angle_diff);
        //     camera.horizon = camera.horizon.rotate_y(horizontal_angle_diff);
        // }
        //
        // let mut distance_diff = 1.0 * app_api.timing_api.frame_duration;
        // if keys_down.contains(&PageDown) {
        //     distance_diff *= -1.0;
        // }
        // if keys_down.contains(&PageUp) || keys_down.contains(&PageDown) {
        //     camera.translation += (Vec3::ZERO - camera.translation).normalize() * distance_diff;
        // }
    }
}

impl ControlUi for CameraControl {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
        ui.label("-");
    }
}