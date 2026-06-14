use crate::app::rendering::RenderItems;
use crate::app::scene::{SceneLayout, SceneObject};
use crate::app::shader_modules::fragment_shader_module::FragmentData;
use crate::app::shader_modules::vertex_shader_module::VertexData;
use crate::app::timing::TimingItems;
use glam::{Mat4, Quat, Vec3};
use std::collections::BTreeSet;
use std::f32::consts::FRAC_PI_2;
use winit::event::KeyEvent;
use winit::keyboard::KeyCode::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, KeyT, PageDown, PageUp};
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::app::util::radians_from_degrees;

pub struct LogicItems {
    // public

    // configuration

    // access through methods
    frame_id: u32,
    vertex_shader_uniforms:Vec<VertexData>,
    fragment_shader_uniforms: Vec<FragmentData>,

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

    pub fn get_vertex_shader_uniform(&self) -> &VertexData {
        &self.vertex_shader_uniform
    }

    pub fn get_fragment_shader_uniform(&self) -> &FragmentData {
        &self.fragment_shader_uniform
    }

    pub fn new() -> Self {
        LogicItems {
            frame_id: 0,
            keys_pressed: BTreeSet::new(),
            keys_down: BTreeSet::new(),
            vertex_shader_uniform: Default::default(),
            fragment_shader_uniform: Default::default(),
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
        
        let camera = &mut scene_layout.camera;

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

    fn make_view_proj_matrix(&self,
                             render_items: &RenderItems,
                             scene_layout: &mut SceneLayout
    ) -> Mat4 {
        let image_extent = render_items.swapchain.image_extent();
        let aspect_ratio = image_extent[0] as f32 / image_extent[1] as f32;
        let projection = Mat4::perspective_lh(
            FRAC_PI_2,
            aspect_ratio,
            0.1,
            1000.0
        );

        let view = Mat4::look_at_lh(
            scene_layout.camera.position,
            Vec3::ZERO,
            Vec3::NEG_Y
        );

        projection * view
    }

    fn make_model_matrix(&self, scene_object: &SceneObject) -> Mat4 {
        let rotation_quaternion =
            Quat::from_rotation_x(radians_from_degrees(scene_object.rotation.x))
            * Quat::from_rotation_y(radians_from_degrees(scene_object.rotation.y))
            * Quat::from_rotation_z(radians_from_degrees(scene_object.rotation.z));

        Mat4::from_scale_rotation_translation(scene_object.translation, rotation_quaternion, scene_object.translation)
    }

    

    pub fn base_logic(&mut self,
                      timing_items: &mut TimingItems,
                      render_items: &RenderItems,
                      scene_layout: &mut SceneLayout
    ) {
        let frame_duration = timing_items.get_frame_duration();
        let view_proj_matrix = self.make_view_proj_matrix(render_items, scene_layout);

        self.handle_input(frame_duration, timing_items, scene_layout);


        
        self.vertex_shader_uniform = VertexData {
            mvp: view_proj_matrix.to_cols_array_2d(),
        };
        
        self.fragment_shader_uniform = FragmentData {
            light_pos: self.light_pos.to_array().into(),
            eye_pos: camera.position.to_array(),
        };

        self.keys_pressed.clear();
    }
}