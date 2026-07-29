use crate::app::rendering::RenderItems;
use crate::app::scene::{Light, SceneItems, SceneObject};
use crate::app::shader_modules::fs_mod_render::RenderFragmentData;
use crate::app::shader_modules::vs_mod_render::RenderVertexData;
use crate::app::shader_modules::vs_mod_shadow::ShadowVertexData;
use crate::app::timing::TimingItems;
use crate::app::util::{radians_from_degrees, ObjectHolder};
use crate::app::AppApi;
use glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles};
use std::collections::{BTreeMap, BTreeSet};
use winit::event::KeyEvent;
use winit::keyboard::KeyCode::KeyT;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct LogicItems {
    // public

    // configuration

    // access through methods
    frame_id: u32,

    // private
    keys_pressed: BTreeSet<KeyCode>,
    keys_down: BTreeSet<KeyCode>,
}

pub struct LogicApi<'api> {
    pub keys_pressed: &'api BTreeSet<KeyCode>,
    pub keys_down: &'api BTreeSet<KeyCode>,
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

    fn handle_input(&mut self,
                    timing_items: &mut TimingItems,)
    {
        let keys_pressed = &self.keys_pressed;
        let keys_down = &self.keys_down;

        if keys_pressed.contains(&KeyT) {
            timing_items.show_frame_times = !timing_items.show_frame_times;
        }
    }

    pub fn base_logic(&mut self,
                      timing_items: &mut TimingItems,
                      render_items: &RenderItems,
                      scene_items: &mut SceneItems,
    ) {
        self.handle_input(timing_items);

        let mut model_matrices = BTreeMap::new();
        Self::make_model_matrices(&mut scene_items.scene_objects, &mut model_matrices, Mat4::IDENTITY,
                                  scene_items.scene_tree_root_id);

        Self::set_uniforms(scene_items, render_items, &mut model_matrices);

        let mut app_api = AppApi::new(self, scene_items, timing_items);
        Self::execute_scripts(&mut app_api);

        self.keys_pressed.clear();
    }

    fn make_model_matrices(scene_objects: &ObjectHolder<SceneObject>,
                           model_matrices: &mut BTreeMap<u32, Mat4>, prev_model_matrix: Mat4,
                           cur_scene_object_id: u32
    ) {
        let cur_scene_object = scene_objects.get(cur_scene_object_id);
        let cur_model_matrix = prev_model_matrix * make_model_matrix(cur_scene_object);
        model_matrices.insert(cur_scene_object.id, cur_model_matrix);

        for child_id in cur_scene_object.children.iter() {
            Self::make_model_matrices(scene_objects, model_matrices, cur_model_matrix, *child_id);
        }
    }

    fn set_uniforms(scene_items: &mut SceneItems, render_items: &RenderItems,
                    model_matrices: &mut BTreeMap<u32, Mat4>,
    ) {
        let light_id = scene_items.get_light().id;
        let camera_id = scene_items.get_camera().id;

        let model_light_matrix = model_matrices.get(&light_id).unwrap();
        let model_camera_matrix = model_matrices.get(&camera_id).unwrap();

        let view_proj_light_matrix = make_proj_light_matrix(scene_items) * model_light_matrix.inverse();
        let view_proj_camera_matrix = make_view_proj_camera_matrix(scene_items, render_items) * model_camera_matrix.inverse();

        for (_, cur_scene_object) in scene_items.scene_objects.get_iter_mut()
        {
            if cur_scene_object.mesh_id.is_none() {
                continue;
            }

            let cur_model_matrix = model_matrices.get(&cur_scene_object.id).unwrap();
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
                material: cur_scene_object.material.unwrap_or_default().into(),
                light_dir: (model_light_matrix.inverse().transpose() * Vec4::new(0.0, 0.0, 1.0, 1.0)).xyz().to_array().into(),
                camera_pos: (model_camera_matrix * Vec4::new(0.0, 0.0, 0.0, 1.0)).xyz().to_array(),
            };

            cur_scene_object.uniforms = Some((shadow_vertex_data, render_vertex_data, render_fragment_data));
        }
    }

    fn execute_scripts(app_api: &mut AppApi)
    {
        for scene_object_id in app_api.scene_api.scene_objects.get_ids()
        {
            let mut scene_object = app_api.scene_api.scene_objects.remove(scene_object_id);

            if let Some(script_id) = scene_object.scene_object_script_id {
                let mut script = app_api.scene_api.scene_object_scripts.remove(script_id);
                script.frame_update(&mut scene_object, app_api);
                app_api.scene_api.scene_object_scripts.insert_at_id(script_id, script);
            }
            if let Some(script_id) = scene_object.instance_script_id {
                let mut script = app_api.scene_api.instance_scripts.remove(script_id);
                script.frame_update(&mut scene_object, app_api);
                script.test();
                app_api.scene_api.instance_scripts.insert_at_id(script_id, script);
            }

            app_api.scene_api.scene_objects.insert(scene_object);
        }
    }
}

fn make_view_proj_camera_matrix(scene_items: &SceneItems, render_items: &RenderItems) -> Mat4 {
    let camera =  scene_items.get_camera().camera.as_ref().unwrap();

    let image_extent = render_items.swapchain.image_extent();
    let aspect_ratio = image_extent[0] as f32 / image_extent[1] as f32;
    let projection = Mat4::perspective_lh(
        radians_from_degrees(camera.fov),
        aspect_ratio,
        0.1,
        100.0
    );

    let view = Mat4::look_to_lh(
        Vec3::ZERO,
        Vec3::Z,
        Vec3::NEG_Y
    );

    projection * view
}

fn make_proj_light_matrix(scene_items: &SceneItems) -> Mat4 {
    match scene_items.get_light().light.as_ref().unwrap() {
        Light::Point { .. } => {
            panic!("Point light not implemented yet")
        }
        Light::Directional { .. } => {
            let box_size = 10f32;
            let projection = Mat4::orthographic_lh(-box_size, box_size, -box_size, box_size, -box_size, box_size);
            projection
        }
    }
}

fn make_model_matrix(scene_object: &SceneObject) -> Mat4 {
    let rotation_quaternion =
              Quat::from_rotation_x(radians_from_degrees(scene_object.transform.rotation.x))
            * Quat::from_rotation_y(radians_from_degrees(scene_object.transform.rotation.y))
            * Quat::from_rotation_z(radians_from_degrees(scene_object.transform.rotation.z));

    Mat4::from_scale_rotation_translation(
        scene_object.transform.scale,
        rotation_quaternion,
        scene_object.transform.translation
    )
}

impl LogicApi<'_> {

    pub fn new(logic_items: &'_ mut LogicItems) -> LogicApi<'_> {
        LogicApi {
            keys_pressed: &logic_items.keys_pressed,
            keys_down: &logic_items.keys_down,
        }
    }
}
