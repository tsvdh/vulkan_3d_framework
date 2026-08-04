use crate::app::shader_modules::fs_mod_render::{PhongMaterial, RenderFragmentData};
use crate::app::shader_modules::vs_mod_render::RenderVertexData;
use crate::app::shader_modules::vs_mod_shadow::ShadowVertexData;
use crate::app::util::{CommonItems, MeshHolder, ObjectHolder, rad_from_deg};
use crate::scripts::{InstanceScript, SceneObjectScript, get_instance_script, get_scene_object_script};
use glam::{EulerRot, Mat4, Quat, Vec3};
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::path::PathBuf;
//
// ----- Data holders -----

#[derive(Deserialize)]
pub struct SceneLayoutConfig {
    pub scene_objects: Vec<SceneObjectConfig>,
}

pub struct SceneItems {
    pub scene_objects: ObjectHolder<SceneObject>,
    pub scene_tree_root_id: u32,
    pub mesh_holder: MeshHolder,
    pub scene_object_scripts: ObjectHolder<Box<dyn SceneObjectScript>>,
    pub instance_scripts: ObjectHolder<Box<dyn InstanceScript>>,
}

#[derive(Deserialize, Clone)]
pub struct TransformConfig {
    #[serde(default)]
    pub translation: Vec3,
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(default = "TransformConfig::default_scale")]
    pub scale: Vec3,
}

pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Deserialize, Clone)]
pub struct Camera {
    pub active: bool,
    pub fov: f32,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Light {
    Point {
        active: bool,
    },
    Directional {
        active: bool,
    },
}

#[derive(Deserialize)]
pub struct SceneObjectConfig {
    pub name: String,
    #[serde(default)]
    pub transform: TransformConfig,

    #[serde(default)]
    pub mesh_path: Option<String>,
    #[serde(default)]
    pub material_path: Option<String>,
    #[serde(default)]
    pub scene_object_script: Option<ScriptConfig>,
    #[serde(default)]
    pub instance_script: Option<ScriptConfig>,

    #[serde(default)]
    pub camera: Option<Camera>,
    #[serde(default)]
    pub light: Option<Light>,

    #[serde(default)]
    pub children: Option<Vec<SceneObjectConfig>>
}

#[derive(Default)]
pub struct SceneObject {
    pub id: u32,

    pub name: String,
    pub transform: Transform,

    pub mesh_id: Option<u32>,
    pub material: Option<PhongMaterial>,
    pub uniforms: Option<(ShadowVertexData, RenderVertexData, RenderFragmentData)>,

    pub scene_object_script_id: Option<u32>,
    pub instance_script_id: Option<u32>,

    pub camera: Option<Camera>,
    pub light: Option<Light>,

    pub children: Vec<u32>
}

#[derive(Deserialize)]
pub struct ScriptConfig {
    name: String,
    #[serde(default)]
    args: serde_json::Value,
}

pub struct SceneApi<'api> {
    pub scene_objects: &'api mut ObjectHolder<SceneObject>,
    pub mesh_holder: &'api MeshHolder,
    pub scene_object_scripts: &'api mut ObjectHolder<Box<dyn SceneObjectScript>>,
    pub instance_scripts: &'api mut ObjectHolder<Box<dyn InstanceScript>>,
}

// ----- Functionality -----

impl SceneItems {
    pub fn new(scene_layout_config: SceneLayoutConfig, common_items: &CommonItems) -> Self {
        let mut scene_tree_root = SceneObject {
            name: "root".to_string(),
            ..Default::default()
        };

        let mut scene_objects = ObjectHolder::new();
        let mut mesh_holder = MeshHolder::new();
        let mut scene_object_scripts = ObjectHolder::new();
        let mut instance_scripts = ObjectHolder::new();

        let working_dir = env::current_dir().unwrap();
        Self::walk_through_scene_tree_config(common_items, &working_dir, &mut mesh_holder,
                                             &mut scene_object_scripts, &mut instance_scripts,
                                             &mut scene_objects, &scene_layout_config.scene_objects, &mut scene_tree_root);

        let scene_tree_root_id = scene_objects.set_id_and_add(scene_tree_root);

        SceneItems {
            scene_objects,
            scene_tree_root_id,
            mesh_holder,
            scene_object_scripts,
            instance_scripts
        }
    }

    fn walk_through_scene_tree_config(common_items: &CommonItems,
                                      working_dir: &PathBuf,
                                      mesh_holder: &mut MeshHolder,
                                      scene_object_scripts: &mut ObjectHolder<Box<dyn SceneObjectScript>>,
                                      instance_scripts: &mut ObjectHolder<Box<dyn InstanceScript>>,
                                      scene_objects: &mut ObjectHolder<SceneObject>,
                                      scene_object_configs: &Vec<SceneObjectConfig>,
                                      parent_scene_object: &mut SceneObject,
    ) {
        for scene_object_config in scene_object_configs {
            if scene_object_config.mesh_path.is_some() && scene_object_config.material_path.is_none() {
                panic!("Material is required if mesh is present")
            }
            if scene_object_config.scene_object_script.is_some() && scene_object_config.instance_script.is_some() {
                panic!("Only one type of script allowed")
            }

            let mut scene_object = SceneObject {
                name: scene_object_config.name.clone(),
                transform: scene_object_config.transform.clone().into(),
                camera: scene_object_config.camera.clone(),
                light: scene_object_config.light.clone(),
                ..Default::default()
            };

            if let Some(mesh_name) = scene_object_config.mesh_path.as_ref() {
                let mesh_path = working_dir.join("resources/meshes").join(mesh_name);

                if !mesh_holder.has_name(&mesh_name) {
                    let mesh_id = mesh_holder.load_and_add_mesh(mesh_name.clone(), &mesh_path, common_items);
                    scene_object.mesh_id = Some(mesh_id);
                } else {
                    scene_object.mesh_id = Some(mesh_holder.get_id(mesh_name));
                }
            }

            if let Some(material_name) = scene_object_config.material_path.as_ref() {
                let material_path = working_dir.join("resources/materials").join(material_name);
                scene_object.material = serde_json::from_reader(File::open(material_path).unwrap())
                    .expect("Incorrect material file");
            }

            if let Some(script_config) = scene_object_config.scene_object_script.as_ref() {
                let script = get_scene_object_script(script_config.name.as_str(), script_config.args.clone());
                let script_id = scene_object_scripts.add(script);
                scene_object.scene_object_script_id = Some(script_id);
            }
            if let Some(script_config) = scene_object_config.instance_script.as_ref() {
                let script = get_instance_script(script_config.name.as_str(), script_config.args.clone());
                let script_id = instance_scripts.add(script);
                scene_object.instance_script_id = Some(script_id);
            }

            if let Some(children) = scene_object_config.children.as_ref() {
                Self::walk_through_scene_tree_config(common_items, working_dir, mesh_holder,
                                                     scene_object_scripts, instance_scripts,
                                                     scene_objects, children, &mut scene_object);
            }

            let scene_object_id = scene_objects.set_id_and_add(scene_object);
            parent_scene_object.children.push(scene_object_id);
        }
    }

    // todo! temporary methods until deferred rendering
    pub fn get_camera(&self) -> &SceneObject {
        for (_, scene_object) in self.scene_objects.get_iter() {
            if let Some(camera) = scene_object.camera.as_ref() {
                if camera.active {
                    return scene_object;
                }
            }
        }
        panic!("No camera found")
    }
    pub fn get_camera_mut(&mut self) -> &mut SceneObject {
        for (_, scene_object) in self.scene_objects.get_iter_mut() {
            if let Some(camera) = scene_object.camera.as_ref() {
                if camera.active {
                    return scene_object;
                }
            }
        }
        panic!("No camera found")
    }
    pub fn get_light(&self) -> &SceneObject {
        for (_, scene_object) in self.scene_objects.get_iter() {
            if let Some(light) = scene_object.light.as_ref() {
                match light {
                    Light::Point { active } => {
                        if *active {
                            return scene_object
                        }
                    }
                    Light::Directional { active } => {
                        if *active {
                            return scene_object
                        }
                    }
                }
            }
        }
        panic!("No light found")
    }
    pub fn get_light_mut(&mut self) -> &mut SceneObject {
        for (_, scene_object) in self.scene_objects.get_iter_mut() {
            if let Some(light) = scene_object.light.as_ref() {
                match light {
                    Light::Point { active } => {
                        if *active {
                            return scene_object
                        }
                    }
                    Light::Directional { active } => {
                        if *active {
                            return scene_object
                        }
                    }
                }
            }
        }
        panic!("No light found")
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl From<TransformConfig> for Transform {
    fn from(value: TransformConfig) -> Self {
        Transform {
            translation: value.translation,
            rotation: Quat::from_rotation_x(rad_from_deg(value.rotation.x))
                    * Quat::from_rotation_y(rad_from_deg(value.rotation.y))
                    * Quat::from_rotation_z(rad_from_deg(value.rotation.z)),
            scale: value.scale
        }
    }
}

impl Transform {
    pub fn new_from(mat: &Mat4) -> Self {
        Transform {
            translation: mat.transform_point3(Vec3::ZERO),
            rotation: Quat::from_mat4(mat),
            scale: mat.transform_vector3(Vec3::ONE)
        }
    }
}

impl Default for TransformConfig {
    fn default() -> Self {
        TransformConfig {
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}
impl TransformConfig {
    fn default_scale() -> Vec3 { Vec3::ONE }
}

impl SceneApi<'_> {
    
    pub fn new(scene_items: &'_ mut SceneItems) -> SceneApi<'_> {
        SceneApi {
            scene_objects: &mut scene_items.scene_objects,
            mesh_holder: &scene_items.mesh_holder,
            scene_object_scripts: &mut scene_items.scene_object_scripts,
            instance_scripts: &mut scene_items.instance_scripts,
        }
    }
}
