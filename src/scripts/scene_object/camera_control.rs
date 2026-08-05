use std::f32::consts::FRAC_PI_2;
use egui::Ui;
use glam::{Quat, Vec3};
use crate::app::scene::{SceneApi, SceneObject};
use crate::app::AppApi;
use crate::scripts::{convert_args, SceneObjectScript};
use serde::Deserialize;
use winit::keyboard::KeyCode;
use crate::app::ui::ControlUi;
use crate::app::util::GlobalTransformData;

#[derive(Deserialize, Default)]
struct Args {

}

pub struct CameraControl {
    args: Args,
    center: Vec3,
}

impl CameraControl {
    pub fn new(args: serde_json::Value) -> Self {
        CameraControl {
            args: convert_args(args),
            center: Vec3::ZERO,
        }
    }
}

impl SceneObjectScript for CameraControl {

    fn frame_update(&mut self, cur_object: &mut SceneObject, global_transform_data: &GlobalTransformData, app_api: &mut AppApi)
    {
        // camera controls
        // rotate 90 degrees (pi/2) in 1 sec
        // zoom 1m in 1 sec

        let mut vertical_angle_diff = FRAC_PI_2 * app_api.timing_api.frame_duration;
        let mut horizontal_angle_diff = FRAC_PI_2 * app_api.timing_api.frame_duration;

        let keys_down = app_api.logic_api.keys_down;

        if keys_down.contains(&KeyCode::ArrowDown) {
            vertical_angle_diff *= -1.0;
        }
        if keys_down.contains(&KeyCode::ArrowRight) {
            horizontal_angle_diff *= -1.0;
        }

        let transform = &mut cur_object.transform;

        if keys_down.contains(&KeyCode::ArrowUp) || keys_down.contains(&KeyCode::ArrowDown) {
            transform.translation = transform.translation.rotate_axis(global_transform_data.right, vertical_angle_diff);
        }
        if keys_down.contains(&KeyCode::ArrowLeft) || keys_down.contains(&KeyCode::ArrowRight) {
            transform.translation = transform.translation.rotate_y(horizontal_angle_diff);
        }

        let mut distance_diff = 1.0 * app_api.timing_api.frame_duration;
        if keys_down.contains(&KeyCode::PageDown) {
            distance_diff *= -1.0;
        }
        if keys_down.contains(&KeyCode::PageUp) || keys_down.contains(&KeyCode::PageDown) {
            transform.translation += (Vec3::ZERO - transform.translation).normalize() * distance_diff;
        }
        
        transform.rotation = Quat::look_at_lh(transform.translation, self.center, Vec3::Y).inverse();
    }
}

impl ControlUi for CameraControl {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
        ui.label("-");
    }
}