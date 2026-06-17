use crate::app::util::{CommonItems, MeshHolder, ObjectHolder, SettableId};
use glam::{U8Vec3, Vec3};
use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

// ----- Data holders -----

#[derive(Deserialize)]
pub struct SceneLayoutConfig {
    pub camera: Camera,
    pub light: Light,
    pub scene_objects: Vec<SceneObjectConfig>,
}

pub struct SceneLayout {
    pub camera: Camera,
    pub light: Light,
    pub scene_objects: ObjectHolder<SceneObject>,
    pub scene_tree: SceneTree,
}

pub struct SceneTree {
    pub object_id: u32,
    pub children: HashSet<SceneTree>
}

#[derive(Deserialize)]
pub struct SceneObjectConfig {
    pub name: String,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,

    #[serde(default)]
    pub mesh_path: Option<String>,
    #[serde(default)]
    pub material_path: Option<String>,

    pub children: Vec<SceneObjectConfig>
}

pub struct SceneObject {
    pub id: u32,

    pub name: String,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,

    pub mesh_id: Option<u32>,
    pub material: Option<PhongMaterial>,
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
    pub position: Vec3,
}

#[derive(Deserialize, Default)]
pub struct PhongMaterial {
    pub name: String,
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

// ----- Functionality -----

impl SettableId for SceneObject {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}

impl PartialEq for SceneTree {
    fn eq(&self, other: &Self) -> bool {
        self.object_id == other.object_id
    }
}

impl Eq for SceneTree {}

impl Hash for SceneTree {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.object_id.hash(state);
    }
}

impl SceneLayoutConfig {

    pub fn parse(self, common_items: &CommonItems) -> (SceneLayout, MeshHolder) {
        let scene_root = SceneObject {
            id: Default::default(),
            name: "root".to_string(),
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            mesh_id: None,
            material: None,
        };

        let mut scene_objects: ObjectHolder<SceneObject> = ObjectHolder::new();
        let root_id = scene_objects.add_with_id(scene_root);

        let mut scene_tree_root = SceneTree {
            object_id: root_id,
            children: HashSet::new()
        };

        let mut mesh_holder = MeshHolder::new();

        let working_dir = env::current_dir().unwrap();
        Self::walk_through_tree(&self.scene_objects, common_items, &mut scene_objects,
                                &mut scene_tree_root, &mut mesh_holder, &working_dir);

        let scene_layout = SceneLayout {
            camera: self.camera,
            light: self.light,
            scene_objects,
            scene_tree: scene_tree_root
        };

        (scene_layout, mesh_holder)
    }

    fn walk_through_tree(scene_object_configs: &Vec<SceneObjectConfig>,
                         common_items: &CommonItems,
                         scene_objects: &mut ObjectHolder<SceneObject>, scene_tree: &mut SceneTree,
                         mesh_holder: &mut MeshHolder,
                         working_dir: &PathBuf)
    {
        for scene_object_config in scene_object_configs {

            let scene_object = SceneObject {
                id: Default::default(),
                name: scene_object_config.name.clone(),
                translation: scene_object_config.translation,
                rotation: scene_object_config.rotation,
                scale: scene_object_config.scale,
                mesh_id: None,
                material: None,
            };
            let object_id = scene_objects.add_with_id(scene_object);
            let scene_object = scene_objects.get_mut(object_id);

            if scene_object_config.mesh_path.is_some() {
                let mesh_name = scene_object_config.mesh_path.clone().unwrap();
                let mesh_path = working_dir.join("resources/meshes").join(mesh_name.clone());

                if !mesh_holder.has_name(&mesh_name) {
                    let mesh_id = mesh_holder.load_and_add_mesh(mesh_name, &mesh_path, common_items);
                    scene_object.mesh_id = Some(mesh_id);
                } else {
                    scene_object.mesh_id = Some(mesh_holder.get_id(&mesh_name));
                }
            }

            if scene_object_config.material_path.is_some() {
                let material_name = scene_object_config.material_path.as_ref().unwrap();
                let material_path = working_dir.join("resources/materials").join(material_name);
                scene_object.material = serde_json::from_reader(File::open(material_path).unwrap())
                    .expect("Incorrect material file");
            }

            let mut child_tree = SceneTree {
                object_id,
                children: HashSet::new()
            };

            Self::walk_through_tree(&scene_object_config.children, common_items, scene_objects,
                                    &mut child_tree, mesh_holder, working_dir);

            scene_tree.children.insert(child_tree);
        }
    }
}