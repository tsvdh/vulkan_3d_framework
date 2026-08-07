use crate::app::AppApi;
use crate::app::scene::{SceneApi, SceneObject};
use crate::app::ui::ControlUi;
use crate::app::util::GlobalTransformData;
use crate::scripts::{SceneObjectScript, convert_args};
use egui::{ComboBox, Ui};
use glam::{Quat, Vec3};
use serde::Deserialize;
use std::f32::consts::FRAC_PI_2;
use std::fmt::{Display, Formatter};
use std::fs::write;
use winit::keyboard::KeyCode;

#[derive(Deserialize, Default)]
struct Args {

}

#[derive(PartialEq)]
enum Mode {
    AroundPoint,
    FirstPerson
}

struct State {
    mode: Mode
}

pub struct CameraControl {
    args: Args,
    state: State,
    center: Vec3,
}

impl CameraControl {
    pub fn new(args: serde_json::Value) -> Self {
        CameraControl {
            args: convert_args(args),
            state: State { mode: Mode::AroundPoint },
            center: Vec3::ZERO,
        }
    }
}

impl SceneObjectScript for CameraControl {

    fn frame_update(&mut self, cur_object: &mut SceneObject, global_transform_data: &GlobalTransformData, app_api: &mut AppApi)
    {
        let keys_down = app_api.logic_api.keys_down;

        let transform = &mut cur_object.transform;

        match self.state.mode {
            Mode::AroundPoint => {
                // camera controls
                // rotate 90 degrees (pi/2) in 1 sec
                // zoom 2m in 1 sec
                // move 2m in 1 sec

                // --- determine changes ---

                let rotation_change = FRAC_PI_2 * app_api.timing_api.frame_duration;
                let mut vertical_angle_diff = 0.0;
                let mut horizontal_angle_diff = 0.0;

                if keys_down.contains(&KeyCode::ArrowUp) {
                    vertical_angle_diff += rotation_change
                }
                if keys_down.contains(&KeyCode::ArrowDown) {
                    vertical_angle_diff -= rotation_change;
                }
                if keys_down.contains(&KeyCode::ArrowLeft) {
                    horizontal_angle_diff += rotation_change;
                }
                if keys_down.contains(&KeyCode::ArrowRight) {
                    horizontal_angle_diff -= rotation_change;
                }

                let zoom_change = 2.0 * app_api.timing_api.frame_duration;
                let mut zoom_diff = 0.0;

                if keys_down.contains(&KeyCode::PageDown) {
                    zoom_diff += zoom_change;
                }
                if keys_down.contains(&KeyCode::PageUp) {
                    zoom_diff -= zoom_change;
                }

                let mut center_diff = Vec3::ZERO;
                let forward_vec = global_transform_data.right.cross(Vec3::Y).normalize();

                if keys_down.contains(&KeyCode::KeyW) {
                    center_diff += forward_vec;
                }
                if keys_down.contains(&KeyCode::KeyS) {
                    center_diff -= forward_vec;
                }
                if keys_down.contains(&KeyCode::KeyA) {
                    center_diff -= global_transform_data.right;
                }
                if keys_down.contains(&KeyCode::KeyD) {
                    center_diff += global_transform_data.right;
                }
                if keys_down.contains(&KeyCode::KeyQ) {
                    center_diff -= Vec3::Y;
                }
                if keys_down.contains(&KeyCode::KeyE) {
                    center_diff += Vec3::Y;
                }
                center_diff *= 2.0 * app_api.timing_api.frame_duration;

                // --- apply changes ---

                transform.translation -= self.center;

                transform.translation = transform.translation.rotate_axis(global_transform_data.right, vertical_angle_diff);
                transform.translation = transform.translation.rotate_y(horizontal_angle_diff);

                transform.translation += transform.translation.normalize() * zoom_diff;

                self.center += center_diff;

                transform.translation += self.center;

                transform.rotation = Quat::look_at_lh(transform.translation, self.center, Vec3::Y).inverse();
            }
            Mode::FirstPerson => {
                // camera controls
                // rotate 90 degrees in 1 sec
                // move 2m in 1 sec

                let y_aligned_up = if global_transform_data.up.y >= 0.0 { Vec3::Y } else { Vec3::NEG_Y };

                // --- determine changes ---

                let rotation_change = FRAC_PI_2 * app_api.timing_api.frame_duration;
                let mut yaw_diff = 0.0;
                let mut pitch_diff = 0.0;

                if keys_down.contains(&KeyCode::ArrowUp) {
                    pitch_diff -= rotation_change
                }
                if keys_down.contains(&KeyCode::ArrowDown) {
                    pitch_diff += rotation_change;
                }
                if keys_down.contains(&KeyCode::ArrowLeft) {
                    yaw_diff -= rotation_change;
                }
                if keys_down.contains(&KeyCode::ArrowRight) {
                    yaw_diff += rotation_change;
                }

                let mut position_diff = Vec3::ZERO;
                let forward_vec = global_transform_data.right.cross(y_aligned_up).normalize();

                if keys_down.contains(&KeyCode::KeyW) {
                    position_diff += forward_vec;
                }
                if keys_down.contains(&KeyCode::KeyS) {
                    position_diff -= forward_vec;
                }
                if keys_down.contains(&KeyCode::KeyA) {
                    position_diff -= global_transform_data.right;
                }
                if keys_down.contains(&KeyCode::KeyD) {
                    position_diff += global_transform_data.right;
                }
                if keys_down.contains(&KeyCode::KeyQ) {
                    position_diff -= Vec3::Y;
                }
                if keys_down.contains(&KeyCode::KeyE) {
                    position_diff += Vec3::Y;
                }
                position_diff *= 2.0 * app_api.timing_api.frame_duration;

                // --- apply changes ---

                transform.translation += position_diff;

                let mut new_forward = global_transform_data.forward;
                let mut new_up = global_transform_data.up;

                let yaw_rotation_axis = y_aligned_up;
                let pitch_rotation_axis = global_transform_data.right;

                new_forward = new_forward.rotate_axis(yaw_rotation_axis, yaw_diff);
                new_up = new_up.rotate_axis(yaw_rotation_axis, yaw_diff);

                new_forward = new_forward.rotate_axis(pitch_rotation_axis, pitch_diff);
                new_up = new_up.rotate_axis(pitch_rotation_axis, pitch_diff);

                transform.rotation = Quat::look_to_lh(new_forward, new_up).inverse();
            }
        }
    }
}

impl ControlUi for CameraControl {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
        ComboBox::from_label("Camera mode")
            .selected_text(format!("{}", self.state.mode))
            .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.state.mode, Mode::AroundPoint, format!("{}", Mode::AroundPoint));
            ui.selectable_value(&mut self.state.mode, Mode::FirstPerson, format!("{}", Mode::FirstPerson));
        });
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                Mode::AroundPoint => { "Around point" }
                Mode::FirstPerson => { "First person" }
            }
        )
    }
}
