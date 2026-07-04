use crate::app::rendering::RenderItems;
use crate::app::scene::{Light, SceneLayout, SceneObject, SceneTree};
use crate::app::shader_modules::fs_mod_render::{RenderFragmentData, Lights};
use crate::app::shader_modules::vs_mod_render::RenderVertexData;
use crate::app::timing::TimingItems;
use glam::{Mat4, Quat, Vec3};
use std::collections::BTreeSet;
use std::f32::consts::FRAC_PI_2;
use winit::event::KeyEvent;
use winit::keyboard::KeyCode::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, KeyT, PageDown, PageUp};
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::app::shader_modules::vs_mod_shadow::ShadowVertexData;
use crate::app::UniformHolder;
use crate::app::util::{radians_from_degrees};

pub struct LogicItems {
    // public

    // configuration

    // access through methods
    frame_id: u32,

    // private
    keys_pressed: BTreeSet<KeyCode>,
    keys_down: BTreeSet<KeyCode>,
}

impl LogicItems {
    pub fn get_frame_id(&self) -> u32 {
        self.frame_id
    }

    pub fn increment_frame_id(&mut self) {
        self.frame_id += 1;
    }

    pub fn new() -> Self {
        LogicItems {
            frame_id: 0,
            keys_pressed: BTreeSet::new(),
            keys_down: BTreeSet::new(),
        }
    }

    pub fn process_keyboard_input(&mut self, event: KeyEvent) {
        if event.repeat == true {
            return;
        }

        match event.physical_key {
            PhysicalKey::Code(key_code) => {
                if event.state.is_pressed() {
                    self.keys_pressed.insert(key_code);
                    self.keys_down.insert(key_code);
                } else {
                    self.keys_down.remove(&key_code);
                }
            }
            PhysicalKey::Unidentified(_) => {}
        }
    }

    fn handle_input(&mut self, frame_duration: f32,
                    timing_items: &mut TimingItems,
                    scene_layout: &mut SceneLayout)
    {
        let keys_pressed = &self.keys_pressed;
        let keys_down = &self.keys_down;

        if keys_pressed.contains(&KeyT) {
            timing_items.show_frame_times = !timing_items.show_frame_times;
        }

        // camera controls
        // rotate 90 degrees (pi/2) in 1 sec
        // zoom 1m in 1 sec

        let mut vertical_angle_diff = FRAC_PI_2 * frame_duration;
        let mut horizontal_angle_diff = FRAC_PI_2 * frame_duration;
        if keys_down.contains(&ArrowDown) {
            vertical_angle_diff *= -1.0;
        }
        if keys_down.contains(&ArrowLeft) {
            horizontal_angle_diff *= -1.0;
        }

        let camera = scene_layout.get_camera_mut();

        if keys_down.contains(&ArrowUp) || keys_down.contains(&ArrowDown) {
            camera.position = camera.position.rotate_axis(camera.horizon, vertical_angle_diff);
        }
        if keys_down.contains(&ArrowLeft) || keys_down.contains(&ArrowRight) {
            camera.position = camera.position.rotate_y(horizontal_angle_diff);
            camera.horizon = camera.horizon.rotate_y(horizontal_angle_diff);
        }

        let mut distance_diff = 1.0 * frame_duration;
        if keys_down.contains(&PageDown) {
            distance_diff *= -1.0;
        }
        if keys_down.contains(&PageUp) || keys_down.contains(&PageDown) {
            camera.position += (Vec3::ZERO - camera.position).normalize() * distance_diff;
        }
    }

    pub fn base_logic(&mut self,
                      timing_items: &mut TimingItems,
                      render_items: &RenderItems,
                      scene_layout: &mut SceneLayout,
                      uniform_holder: &mut UniformHolder
    ) {
        let frame_duration = timing_items.get_frame_duration();
        self.handle_input(frame_duration, timing_items, scene_layout);

        let view_proj_camera_matrix = make_view_proj_camera_matrix(render_items, scene_layout);
        let view_proj_light_matrix = make_view_proj_light_matrix(scene_layout);
        let f_lights = Lights {
            point_light: scene_layout.get_light().get_point_light(),
            directional_light: scene_layout.get_light().get_directional_light(),
        };
        Self::walk_through_tree(&scene_layout.scene_tree, scene_layout,
                                &view_proj_camera_matrix, &view_proj_light_matrix, &Mat4::IDENTITY,
                                uniform_holder,
                                &f_lights, &scene_layout.get_camera().position.to_array());

        self.keys_pressed.clear();
    }

    fn walk_through_tree(scene_tree: &SceneTree, scene_layout: &SceneLayout,
                         view_proj_camera_matrix: &Mat4, view_proj_light_matrix: &Mat4, prev_model_matrix: &Mat4,
                         uniform_holder: &mut UniformHolder,
                         f_lights: &Lights, f_camera_pos: &[f32; 3])
    {
        let cur_entity = scene_layout.scene_entities.get(scene_tree.entity_id);
        if cur_entity.downcast_ref::<SceneObject>().is_none() {
            if !scene_tree.children.is_empty() {
                panic!("Only scene objects can have children")
            }
            return;
        }

        let cur_object = cur_entity.downcast_ref::<SceneObject>().unwrap();

        let cur_model_matrix = prev_model_matrix * make_model_matrix(cur_object);
        let cur_model_normals_matrix = cur_model_matrix.inverse().transpose();
        let cur_mvp_light_matrix = view_proj_light_matrix * cur_model_matrix;

        let shadow_vertex_data = ShadowVertexData {
            mvp_light: cur_mvp_light_matrix.to_cols_array_2d(),
        };
        let render_vertex_data = RenderVertexData {
            model: cur_model_matrix.to_cols_array_2d(),
            model_normals: cur_model_normals_matrix.to_cols_array_2d(),
            view_proj_camera: view_proj_camera_matrix.to_cols_array_2d(),
            view_proj_light: view_proj_light_matrix.to_cols_array_2d(),
        };
        let render_fragment_data = RenderFragmentData {
            material: cur_object.material.unwrap_or_default().into(),
            lights: *f_lights,
            camera_pos: *f_camera_pos,
        };

        uniform_holder.insert(cur_object.id, (shadow_vertex_data, render_vertex_data, render_fragment_data));

        for child in scene_tree.children.iter() {
            Self::walk_through_tree(child, scene_layout, view_proj_camera_matrix, view_proj_light_matrix,
                                    &cur_model_matrix, uniform_holder, f_lights, f_camera_pos);
        }
    }
}

fn make_view_proj_camera_matrix(render_items: &RenderItems, scene_layout: &SceneLayout) -> Mat4 {
    let image_extent = render_items.swapchain.image_extent();
    let aspect_ratio = image_extent[0] as f32 / image_extent[1] as f32;
    let projection = Mat4::perspective_lh(
        radians_from_degrees(65.0),
        aspect_ratio,
        0.1,
        100.0
    );

    let view = Mat4::look_at_lh(
        scene_layout.get_camera().position,
        Vec3::ZERO,
        Vec3::NEG_Y
    );

    projection * view
}

fn make_view_proj_light_matrix(scene_layout: &SceneLayout) -> Mat4 {
    match scene_layout.get_light() {
        Light::Point { .. } => {
            panic!("Point light not implemented yet")
        }
        Light::Directional { direction, .. } => {
            let box_size = 10f32;
            let projection = Mat4::orthographic_lh(-box_size, box_size, -box_size, box_size, -box_size, box_size);
            let view = Mat4::look_to_lh(Vec3::ZERO, *direction, direction.any_orthonormal_vector());
            projection * view
        }
    }
}

fn make_model_matrix(scene_object: &SceneObject) -> Mat4 {
    let rotation_quaternion =
        Quat::from_rotation_x(radians_from_degrees(scene_object.rotation.x))
            * Quat::from_rotation_y(radians_from_degrees(scene_object.rotation.y))
            * Quat::from_rotation_z(radians_from_degrees(scene_object.rotation.z));

    Mat4::from_scale_rotation_translation(scene_object.scale, rotation_quaternion, scene_object.translation)
}
