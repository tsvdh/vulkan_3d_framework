use std::ops::DerefMut;
use crate::app::rendering::RenderItems;
use crate::app::scene::{Camera, Light, SceneApi, SceneItems, SceneObject};
use crate::app::util::{deg_from_rad, CommonItems, WithId, rad_from_deg};
use crate::scripts::{InstanceScript, SceneObjectScript};
use egui::{collapsing_header, Align, Atoms, Context, DragValue, Frame, Layout, MenuBar, Panel, RichText, TextStyle, Ui, UiBuilder};
use egui_winit_vulkano::{Gui, GuiConfig};
use glam::{EulerRot, Quat};
use vulkano::image::SampleCount;
use winit::event_loop::ActiveEventLoop;

struct State {
    selected_object_id: Option<u32>,
    show_tree_panel: bool,
    show_control_panel: bool,
}

impl State {
    fn new() -> Self {
        State {
            selected_object_id: None,
            show_tree_panel: true,
            show_control_panel: false,
        }
    }
}

const TEXT_SIZE: f32 = 15.0;
const HEADER_SIZE: f32 = 22.0;

const LEFT_FOLD_ICON_HEX: &str = "23F4";
const RIGHT_FOLD_ICON_HEX: &str = "23F5";
const CAMERA_ICON_HEX: &str = "1F3A5";
const LIGHT_ICON_HEX: &str = "2600";
const BOX_ICON_HEX: &str = "1F4E6";
const OBJECT_ICON_HEX: &str = "1F34E";
const ALT_OBJECT_ICON_HEX: &str = "1F310";

pub struct GuiItems {
    // public
    pub gui: Gui,

    // configuration

    // access through methods

    // private
    state: State,
}

impl GuiItems {

    pub fn new(event_loop: &ActiveEventLoop,
               vulkan_items: &CommonItems,
               render_items: &RenderItems,
    ) -> GuiItems
    {
        let swapchain = render_items.swapchain.clone();
        let egui_config = GuiConfig {
            allow_srgb_render_target: true,
            is_overlay: true,
            samples: SampleCount::Sample1,
        };
        
        let gui = Gui::new(
            event_loop,
            swapchain.surface().clone(),
            vulkan_items.queue.clone(),
            swapchain.image_format(),
            egui_config
        );
        
        GuiItems {
            gui,
            state: State::new(),
        }
    }

    fn set_font_sizes(context: &Context) {
        context.global_style_mut(|style| {
            for (text_style, font_id) in style.text_styles.iter_mut() {
                match text_style {
                    TextStyle::Body => { font_id.size = TEXT_SIZE }
                    TextStyle::Button => { font_id.size = TEXT_SIZE }
                    TextStyle::Heading => { font_id.size = HEADER_SIZE }
                    _ => {}
                }
            }
        });
    }

    pub fn build_ui(&mut self,
                    scene_items: &mut SceneItems,
    ) {
        self.gui.begin_frame();
        let context = self.gui.context();
        Self::set_font_sizes(&context);

        let mut ui = Ui::new(context.clone(), "ui".into(), UiBuilder::new());

        Panel::top("menu")
            .show_inside(&mut ui, |ui| {
                ui.menu_button("file", |_ui| {

                });
            });

        Panel::left("treePanel")
            .resizable(false)
            .show_animated_inside(&mut ui, self.state.show_tree_panel, |ui| {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Entity tree");
                });
                ui.separator();

                self.walk_through_tree(scene_items, scene_items.scene_tree_root_id, &context, ui);
            });

        Panel::right("controlPanel")
            .resizable(false)
            .show_animated_inside(&mut ui, self.state.show_control_panel, |ui| {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Entity control");
                });
                ui.separator();

                if let Some(selected_object_id) = self.state.selected_object_id
                {
                    let mut selected_object = scene_items.scene_objects.remove(selected_object_id);

                    let mut scene_api = SceneApi::new(scene_items);
                    selected_object.control_ui(ui, &mut scene_api);

                    scene_items.scene_objects.insert(selected_object);
                } else {
                    ui.label("Nothing selected");
                }
            });

        Panel::top("foldBar")
            .show_separator_line(false)
            .frame(Frame::new().inner_margin(4))
            .show_inside(&mut ui, |ui| {
                MenuBar::new().ui(ui, |ui| {

                    let tree_panel_toggle_text = if self.state.show_tree_panel {
                        hex_to_emoji(LEFT_FOLD_ICON_HEX, 20.0)
                    } else {
                        hex_to_emoji(RIGHT_FOLD_ICON_HEX, 20.0)
                    };
                    if ui.button(tree_panel_toggle_text).clicked() {
                        self.state.show_tree_panel = !self.state.show_tree_panel;
                    }

                    let control_panel_toggle_text = if self.state.show_control_panel {
                        hex_to_emoji(RIGHT_FOLD_ICON_HEX, 20.0)
                    } else {
                        hex_to_emoji(LEFT_FOLD_ICON_HEX, 20.0)
                    };
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(control_panel_toggle_text).clicked() {
                            self.state.show_control_panel = !self.state.show_control_panel;
                            if !self.state.show_control_panel {
                                self.state.selected_object_id = None;
                            }
                        }
                    });
                });
            });
    }

    fn walk_through_tree(&mut self,
                         scene_items: &SceneItems, cur_object_id: u32,
                         context: &Context, ui: &mut Ui)
    {
        let cur_object = scene_items.scene_objects.get(cur_object_id);
        let header_name = format!("{}_header", cur_object.name);

        let show_item_label = |this: &mut GuiItems, ui: &mut Ui| {
            let object_selected = this.state.selected_object_id.is_some_and(
                |id| { id == cur_object.get_id() });

            if ui.selectable_label(object_selected, cur_object.tree_heading_atoms()).clicked() {
                if this.state.selected_object_id.is_some_and(|id| {id == cur_object.get_id()}) {
                    this.state.selected_object_id = None;
                    this.state.show_control_panel = false;
                } else {
                    this.state.selected_object_id = Some(cur_object.get_id());
                    this.state.show_control_panel = true;
                }
            }
        };

        if cur_object.name == "root" {
            for child_id in cur_object.children.iter() {
                self.walk_through_tree(scene_items, *child_id, context, ui);
            }
            return
        }

        if cur_object.children.is_empty() {
            show_item_label(self, ui);
        } else {
            collapsing_header::CollapsingState::load_with_default_open(&context, header_name.into(), false)
                .show_header(ui, |ui| {
                    show_item_label(self, ui);
                })
                .body(|ui| {
                    for child_id in cur_object.children.iter() {
                        self.walk_through_tree(scene_items, *child_id, context, ui);
                    }
                });
        }
    }
}

pub fn vec3_drag_values_float_range(ui: &mut Ui, vec3: &mut [f32; 3], speed: f32, min: f32, max: f32)
{
    ui.horizontal(|ui| {
        ui.label("x: ");
        ui.add(DragValue::new(&mut vec3[0]).speed(speed).range(min..=max));
        ui.label("y: ");
        ui.add(DragValue::new(&mut vec3[1]).speed(speed).range(min..=max));
        ui.label("z: ");
        ui.add(DragValue::new(&mut vec3[2]).speed(speed).range(min..=max));
    });
}
pub fn vec3_drag_values_int_range(ui: &mut Ui, vec3: &mut [f32; 3], speed: f32, min: i32, max: i32) {
    vec3_drag_values_float_range(ui, vec3, speed, min as f32, max as f32);
}
pub fn vec3_drag_values(ui: &mut Ui, vec3: &mut [f32; 3], speed: f32) {
    vec3_drag_values_int_range(ui, vec3, speed, i32::MIN, i32::MAX);
}

pub trait ControlUi {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi);
}
impl ControlUi for Light {
    fn control_ui(&mut self, ui: &mut Ui, _scene_api: &mut SceneApi) {
        ui.label("Type");
        match self {
            Light::Point { .. } => { ui.label("Point"); }
            Light::Directional { .. } => { ui.label("Directional"); }
        }
    }
}
impl ControlUi for Camera {
    fn control_ui(&mut self, ui: &mut Ui, _scene_api: &mut SceneApi) {
        ui.horizontal(|ui| {
            ui.label("FOV: ");
            ui.add(DragValue::new(&mut self.fov,).speed(0.1).range(10..=100));
        });
    }
}
impl ControlUi for Box<dyn SceneObjectScript> {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
        self.deref_mut().control_ui(ui, scene_api);
    }
}
impl ControlUi for Box<dyn InstanceScript> {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
        self.deref_mut().control_ui(ui, scene_api)
    }
}
impl ControlUi for SceneObject {
    fn control_ui(&mut self, ui: &mut Ui, scene_api: &mut SceneApi) {
        ui.horizontal(|ui| {
            ui.label(hex_to_emoji(OBJECT_ICON_HEX, 20.0));
            ui.label(format!("{}", self.name))
        });

        ui.separator();
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Transform").size(TEXT_SIZE + 2.0))
        });

        ui.label("Translation");
        vec3_drag_values(ui, self.transform.translation.as_mut(), 0.1);

        let mut rotation_euler: [f32; 3] = self.transform.rotation.to_euler(EulerRot::XYZ).into();
        for i in 0..3 {
            rotation_euler[i] = deg_from_rad(rotation_euler[i]);
        }
        ui.add_space(8.0);
        ui.label("Rotation");
        vec3_drag_values(ui, &mut rotation_euler, 1.0);
        self.transform.rotation = Quat::from_euler(EulerRot::XYZ,
                                                   rad_from_deg(rotation_euler[0]),
                                                   rad_from_deg(rotation_euler[1]),
                                                   rad_from_deg(rotation_euler[2]));

        ui.add_space(8.0);
        ui.label("Scale");
        vec3_drag_values(ui, self.transform.scale.as_mut(), 0.02);

        fn attribute_control_ui(name: &str, opt_attribute: Option<&mut impl ControlUi>, ui: &mut Ui, scene_api: &mut SceneApi) {
            if let Some(attribute) = opt_attribute {
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(name).size(TEXT_SIZE + 2.0))
                });
                attribute.control_ui(ui, scene_api);
            }
        }

        if self.mesh_id.is_some() {
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Mesh").size(TEXT_SIZE + 2.0))
            });
            ui.label("-");
        }

        attribute_control_ui("Material", self.material.as_mut(), ui, scene_api);
        attribute_control_ui("Camera", self.camera.as_mut(), ui, scene_api);
        attribute_control_ui("Light", self.light.as_mut(), ui, scene_api);

        {
            let mut script = self.scene_object_script_id.map(|script_id| { scene_api.scene_object_scripts.remove(script_id) });
            attribute_control_ui("Scene Object Script", script.as_mut(), ui, scene_api);
            if let Some(script_id) = self.scene_object_script_id {
                scene_api.scene_object_scripts.insert_at_id(script_id, script.unwrap());
            }
        } {
            let mut script = self.instance_script_id.map(|script_id| { scene_api.instance_scripts.remove(script_id) });
            attribute_control_ui("Instance Script", script.as_mut(), ui, scene_api);
            if let Some(script_id) = self.instance_script_id {
                scene_api.instance_scripts.insert_at_id(script_id, script.unwrap());
            }
        }
    }
}

fn hex_to_emoji(hex: &str, size: f32) -> RichText {
    if let Ok(number) = u32::from_str_radix(hex, 16) {
        if let Some(character) = std::char::from_u32(number) {
            return RichText::new(character).size(size)
        }
    }
    RichText::new("?").size(size)
}

pub trait TreeHeadingUi {
    fn tree_heading_atoms(&'_ self) -> Atoms<'_>;
}
impl TreeHeadingUi for SceneObject {
    fn tree_heading_atoms(&'_ self) -> Atoms<'_> {
        let mut icon_hex = BOX_ICON_HEX;
        if self.mesh_id.is_some() {
            icon_hex = OBJECT_ICON_HEX;
        }
        if self.light.is_some() {
            icon_hex = LIGHT_ICON_HEX;
        }
        if self.camera.is_some() {
            icon_hex = CAMERA_ICON_HEX;
        }
        Atoms::new((hex_to_emoji(icon_hex, TEXT_SIZE), self.name.clone()))
    }
}
