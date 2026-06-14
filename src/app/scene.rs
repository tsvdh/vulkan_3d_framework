use std::collections::{BTreeMap};
use rand::random;
use glam::{U8Vec3, Vec3};
use serde::Deserialize;
use crate::app::util::generate_scene_tree;

#[derive(Deserialize)]
pub struct SceneLayoutConfig {
    pub camera: Camera,
    pub light: Light,
    pub scene_objects: Vec<SceneObjectConfig>,
}

pub struct SceneLayout {
    pub camera: Camera,
    pub light: Light,
    pub scene_root: SceneObject,
}

impl From<SceneLayoutConfig> for SceneLayout {
    fn from(value: SceneLayoutConfig) -> Self {
        let mut scene_root = SceneObject {
            id: random(),
            name: "root".to_string(),
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            obj_id: 0,
            material: Default::default(),
            children: BTreeMap::new(),
        };

        generate_scene_tree(&value.scene_objects, &mut scene_root);

        SceneLayout {
            camera: value.camera,
            light: value.light,
            scene_root
        }
    }
}

#[derive(Deserialize)]
pub struct SceneObjectConfig {
    pub name: String,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,

    pub obj_path: String,
    pub material_path: String,

    pub children: Vec<SceneObjectConfig>
}



pub struct SceneObject {
    pub id: u64,

    pub name: String,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,

    pub obj_id: u32,
    pub material: PhongMaterial,

    pub children: BTreeMap<u64, SceneObject>
}

#[derive(Deserialize)]
pub struct Camera {
    pub position: Vec3,
    pub look_at: Vec3,
    pub horizon: Vec3,
}

#[derive(Deserialize)]
pub struct Light {
    pub name: String,
    pub direction: Vec3,
}

#[derive(Deserialize, Default)]
pub struct PhongMaterial {
    pub ambient: PhongComponent,
    pub diffuse: PhongComponent,
    pub specular: PhongComponent,
    pub shininess: u32,
}

#[derive(Deserialize, Default)]
pub struct PhongComponent {
    pub color: U8Vec3,
    pub coefficient: f32,
}

