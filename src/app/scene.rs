use crate::app::shader_modules::fragment_shader_module::PhongMaterial;
use crate::app::ui::{ControlUi, TreeHeadingUi};
use crate::app::util::{CommonItems, MeshHolder, ObjectHolder};
use downcast_rs::{impl_downcast, Downcast};
use glam::Vec3;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::path::PathBuf;

// ----- Data holders -----

#[derive(Deserialize)]
pub struct SceneLayoutConfig {
    pub camera: Camera,
    pub light: Light,
    pub scene_objects: Vec<SceneObjectConfig>,
}

pub struct SceneLayout {
    pub camera_id: u32,
    pub light_id: u32,
    pub scene_entities: ObjectHolder<Box<dyn SceneEntity>>,
    pub scene_tree: SceneTree,
}

pub struct SceneTree {
    pub entity_id: u32,
    pub children: Vec<SceneTree>
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
    #[serde(skip)]
    pub id: u32,

    pub position: Vec3,
    // pub look_at: Vec3,
    pub horizon: Vec3,
}

#[derive(Deserialize)]
pub struct Light {
    #[serde(skip)]
    pub id: u32,

    pub position: Vec3,
}

// ----- Functionality -----

pub trait SceneEntity : ControlUi + TreeHeadingUi + Downcast {
    fn get_id(&self) -> u32;
    fn set_id(&mut self, id: u32);
    fn get_name(&self) -> &str;
}
impl_downcast!(SceneEntity);

impl SceneEntity for SceneObject {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn set_id(&mut self, id: u32) {
        self.id = id
    }
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}
impl SceneEntity for Camera {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn set_id(&mut self, id: u32) {
        self.id = id
    }
    fn get_name(&self) -> &str {
        "camera"
    }
}
impl SceneEntity for Light {
    fn get_id(&self) -> u32 {
        self.id
    }
    fn set_id(&mut self, id: u32) {
        self.id = id
    }
    fn get_name(&self) -> &str {
        "light"
    }
}
impl SceneEntity for Box<dyn SceneEntity> {
    fn get_id(&self) -> u32 {
        self.as_ref().get_id()
    }
    fn set_id(&mut self, id: u32) {
        self.as_mut().set_id(id);
    }
    fn get_name(&self) -> &str {
        self.as_ref().get_name()
    }
}

impl SceneTree {
    fn new(entity_id: u32) -> Self {
        SceneTree {
            entity_id,
            children: Vec::new(),
        }
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

        let mut scene_entities: ObjectHolder<Box<dyn SceneEntity>> = ObjectHolder::new();

        let root_id = scene_entities.add_with_id(Box::new(scene_root));
        let mut scene_tree_root = SceneTree::new(root_id);

        let camera_id = scene_entities.add_with_id(Box::new(self.camera));
        let light_id = scene_entities.add_with_id(Box::new(self.light));
        scene_tree_root.children.push(SceneTree::new(camera_id));
        scene_tree_root.children.push(SceneTree::new(light_id));

        let mut mesh_holder = MeshHolder::new();

        let working_dir = env::current_dir().unwrap();
        Self::walk_through_tree(&self.scene_objects, common_items, &mut scene_entities,
                                &mut scene_tree_root, &mut mesh_holder, &working_dir);

        let scene_layout = SceneLayout {
            camera_id,
            light_id,
            scene_entities,
            scene_tree: scene_tree_root
        };

        (scene_layout, mesh_holder)
    }

    fn walk_through_tree(scene_object_configs: &Vec<SceneObjectConfig>,
                         common_items: &CommonItems,
                         scene_entities: &mut ObjectHolder<Box<dyn SceneEntity>>, scene_tree: &mut SceneTree,
                         mesh_holder: &mut MeshHolder,
                         working_dir: &PathBuf)
    {
        for scene_object_config in scene_object_configs
        {
            if scene_object_config.mesh_path.is_some() && scene_object_config.material_path.is_none() {
                panic!("Material is required if mesh is present")
            }

            let mut scene_object = SceneObject {
                id: Default::default(),
                name: scene_object_config.name.clone(),
                translation: scene_object_config.translation,
                rotation: scene_object_config.rotation,
                scale: scene_object_config.scale,
                mesh_id: None,
                material: None,
            };

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

            let entity_id = scene_entities.add_with_id(Box::new(scene_object));
            let mut child_tree = SceneTree::new(entity_id);

            Self::walk_through_tree(&scene_object_config.children, common_items, scene_entities,
                                    &mut child_tree, mesh_holder, working_dir);

            scene_tree.children.push(child_tree);
        }
    }
}

impl SceneLayout {

    pub fn get_camera(&self) -> &Camera {
        self.scene_entities.get(self.camera_id)
            .downcast_ref().unwrap()
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        self.scene_entities.get_mut(self.camera_id)
            .downcast_mut().unwrap()
    }

    pub fn get_light(&self) -> &Light {
        self.scene_entities.get(self.light_id)
            .downcast_ref().unwrap()
    }

    pub fn get_light_mut(&mut self) -> &mut Light {
        self.scene_entities.get_mut(self.light_id)
            .downcast_mut().unwrap()
    }
}
